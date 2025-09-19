# Modern Nix flake example
# Tests flake syntax and structure

{
  description = "Codanna code intelligence system";

  inputs = {
    # Core inputs
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    
    # Development tools
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    # Optional crane for Rust builds
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        
        # Rust toolchain configuration
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
          targets = [ "x86_64-unknown-linux-gnu" "aarch64-apple-darwin" ];
        };
        
        # Crane library for Rust builds
        craneLib = crane.lib.${system}.overrideToolchain rustToolchain;
        
        # Common source filtering
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        
        # Common build arguments
        commonArgs = {
          inherit src;
          pname = "codanna";
          version = "0.5.6";
          
          buildInputs = with pkgs; [
            openssl
          ] ++ lib.optionals stdenv.isDarwin [
            libiconv
            darwin.apple_sdk.frameworks.CoreFoundation
            darwin.apple_sdk.frameworks.Security
          ];
          
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          
          # Environment variables
          env = {
            OPENSSL_NO_VENDOR = "1";
          };
        };
        
        # Build dependencies separately for caching
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        
        # The main package
        codanna = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          
          meta = with pkgs.lib; {
            description = "Advanced code intelligence and analysis tool";
            license = licenses.mit;
            maintainers = [ ];
          };
        });
        
        # Development shell packages
        devPackages = with pkgs; [
          # Rust toolchain
          rustToolchain
          
          # Development tools
          cargo-watch
          cargo-edit
          cargo-audit
          cargo-outdated
          cargo-udeps
          
          # System dependencies
          pkg-config
          openssl
          
          # Additional development tools
          git
          just
          fd
          ripgrep
          
          # Language servers and formatters
          rust-analyzer
          rustfmt
          clippy
          
          # Benchmarking and profiling
          hyperfine
          perf-tools
          
          # Documentation tools
          mdbook
        ];
        
      in {
        # Packages
        packages = {
          default = codanna;
          inherit codanna;
          
          # Additional variants
          codanna-minimal = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--no-default-features --features minimal";
          });
          
          codanna-full = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "--all-features";
          });
        };
        
        # Development shells
        devShells = {
          default = pkgs.mkShell {
            buildInputs = devPackages;
            
            # Shell environment
            env = {
              RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
              RUST_LOG = "debug";
            };
            
            shellHook = ''
              echo "Codanna development environment"
              echo "Rust version: $(rustc --version)"
              echo "Cargo version: $(cargo --version)"
              
              # Set up git hooks if needed
              if [ ! -f .git/hooks/pre-commit ]; then
                echo "Setting up git hooks..."
                # Setup pre-commit hooks here if desired
              fi
            '';
          };
          
          # Minimal shell for quick tasks
          minimal = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustToolchain
              pkg-config
              openssl
            ];
          };
        };
        
        # Checks (run with 'nix flake check')
        checks = {
          # Run tests
          test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
          });
          
          # Check formatting
          fmt = craneLib.cargoFmt {
            inherit src;
          };
          
          # Run clippy
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
          
          # Check documentation
          doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });
          
          # Security audit
          audit = craneLib.cargoAudit {
            inherit src;
          };
        };
        
        # Formatter for 'nix fmt'
        formatter = pkgs.nixpkgs-fmt;
        
        # Apps (run with 'nix run')
        apps = {
          default = flake-utils.lib.mkApp {
            drv = codanna;
            name = "codanna";
          };
          
          codanna = flake-utils.lib.mkApp {
            drv = codanna;
            name = "codanna";
          };
        };
      }) // {
        # System-independent outputs
        
        # Overlays for use in other flakes
        overlays = {
          default = final: prev: {
            codanna = self.packages.${final.system}.default;
          };
        };
        
        # NixOS module (if applicable)
        nixosModules = {
          default = { config, lib, pkgs, ... }: {
            options.services.codanna = with lib; {
              enable = mkEnableOption "Codanna code intelligence service";
              
              package = mkOption {
                type = types.package;
                default = self.packages.${pkgs.system}.default;
                description = "Codanna package to use";
              };
              
              port = mkOption {
                type = types.port;
                default = 8080;
                description = "Port for Codanna server";
              };
            };
            
            config = lib.mkIf config.services.codanna.enable {
              systemd.services.codanna = {
                description = "Codanna code intelligence service";
                wantedBy = [ "multi-user.target" ];
                after = [ "network.target" ];
                
                serviceConfig = {
                  ExecStart = "${config.services.codanna.package}/bin/codanna server --port ${toString config.services.codanna.port}";
                  Restart = "always";
                  User = "codanna";
                  Group = "codanna";
                };
              };
              
              users.users.codanna = {
                isSystemUser = true;
                group = "codanna";
              };
              
              users.groups.codanna = {};
              
              networking.firewall.allowedTCPPorts = [ config.services.codanna.port ];
            };
          };
        };
      };
}