{
  description = "Rust development environment";

  inputs = {
    # Reference to your isolated development shells flake
    devshells.url = "git+ssh://git@github.com/Ba-So/nixos-flakes?dir=devshells";
  };

  outputs = { self, devshells }:
    let
      # Get all systems that devshells supports
      systems = builtins.attrNames devshells.devShells;
      genAttrs = names: f: builtins.listToAttrs (map (n: { name = n; value = f n; }) names);
    in
    {
      devShells = genAttrs systems (system: {
        # Use the Rust development shell from your central configuration
        default = devshells.devShells.${system}.rust;
        rust = devshells.devShells.${system}.rust;
      });
    };
}
