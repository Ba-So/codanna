# Basic Nix bindings and constructs for testing
# This file contains fundamental Nix language features

let
  # Simple variable bindings
  name = "test-package";
  version = "1.0.0";
  
  # Numeric values
  port = 8080;
  count = 42;
  
  # Boolean values
  enabled = true;
  debug = false;
  
  # List values
  dependencies = [ "lib1" "lib2" "lib3" ];
  
  # Simple attribute set
  config = {
    host = "localhost";
    port = 3000;
    ssl = false;
  };
  
  # Function definitions
  add = a: b: a + b;
  
  # Function with default parameters
  greet = { name ? "World" }: "Hello, ${name}!";
  
in {
  # Export all bindings
  inherit name version port count enabled debug dependencies config;
  inherit add greet;
  
  # Additional computed values
  displayName = "${name} v${version}";
  isProduction = !debug;
}