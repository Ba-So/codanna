{
  description = "Codanna - Code Intelligence for Large Language Models";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs { inherit system; };

          codanna = pkgs.rustPlatform.buildRustPackage {
            pname = "codanna";
            version = "0.9.14-nix";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            # Optimize build for faster compilation
            doCheck = false;
            auditable = false;

            # Build with HTTP server feature (default)
            buildFeatures = [ "http-server" ];

            nativeBuildInputs = with pkgs; [ pkg-config perl ];
            buildInputs = with pkgs; [ openssl onnxruntime ];

            # Configure ort-sys to use system ONNX Runtime instead of downloading
            env = {
              ORT_LIB_LOCATION = "${pkgs.onnxruntime}";
              ORT_SKIP_DOWNLOAD = "1";
            };

            postInstall = ''
                          if [ -f "$out/bin/codanna" ]; then
                            mv "$out/bin/codanna" "$out/bin/codanna-unwrapped"

                            cat > "$out/bin/codanna" << 'EOF'
              #!/usr/bin/env bash
              export CODANNA_DATA_DIR="''${XDG_DATA_HOME:-$HOME/.local/share}/codanna"
              export CODANNA_CONFIG_DIR="''${XDG_CONFIG_HOME:-$HOME/.config}/codanna"
              mkdir -p "$CODANNA_DATA_DIR" "$CODANNA_CONFIG_DIR"
              exec "$(dirname "$0")/codanna-unwrapped" "$@"
              EOF
                            chmod +x "$out/bin/codanna"
                          fi
            '';

            meta = with pkgs.lib; {
              description = "Code intelligence for Large Language Models - semantic search and navigation";
              homepage = "https://github.com/bartolli/codanna";
              license = licenses.asl20;
              mainProgram = "codanna";
              platforms = platforms.all;
            };
          };

        in
        {
          packages = {
            default = codanna;
            inherit codanna;
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              # Rust toolchain
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer

              # Build dependencies
              pkg-config
              openssl
              onnxruntime
              perl

              # Development tools
              cargo-watch
              cargo-edit
            ];

            env = {
              ORT_LIB_LOCATION = "${pkgs.onnxruntime}";
              ORT_SKIP_DOWNLOAD = "1";
              RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            };

            shellHook = ''
              echo "Codanna development environment"
              echo "Rust: $(rustc --version)"
            '';
          };

          apps.default = flake-utils.lib.mkApp {
            drv = codanna;
            name = "codanna";
          };
        }) // {
      overlays.default = final: prev: {
        codanna = self.packages.${final.system}.default;
      };
    };
}
