# Complex real-world Nix example
# Simulates a package definition with build configuration

{ lib, stdenv, fetchFromGitHub, rustPlatform, pkg-config, openssl, libressl, darwin }:

let
  # Version and source information
  pname = "codanna";
  version = "0.5.6";
  
  # Platform-specific dependencies
  isDarwin = stdenv.isDarwin;
  isx86_64 = stdenv.isx86_64;
  
  # SSL library selection
  sslLibrary = if isDarwin then libressl else openssl;
  
  # Build inputs based on platform
  buildInputs = [ sslLibrary ] ++ lib.optionals isDarwin [
    darwin.apple_sdk.frameworks.CoreFoundation
    darwin.apple_sdk.frameworks.Security
  ];
  
  nativeBuildInputs = [ pkg-config ];
  
  # Feature configuration
  features = {
    default = [ "cli" "server" ];
    all = [ "cli" "server" "benchmarks" "extra-parsers" ];
    minimal = [ "cli" ];
  };
  
  # Build configuration functions
  mkFeatureFlags = featureSet: builtins.concatStringsSep "," featureSet;
  
  # Cargo build configuration
  cargoFlags = selectedFeatures: [
    "--release"
    "--features"
    (mkFeatureFlags selectedFeatures)
  ];
  
  # Source configuration
  src = fetchFromGitHub {
    owner = "anthropics";
    repo = pname;
    rev = "v${version}";
    sha256 = lib.fakeSha256;
  };
  
  # Build phases
  preBuild = ''
    echo "Building ${pname} version ${version}"
    export CARGO_NET_OFFLINE=1
  '';
  
  buildPhase = ''
    runHook preBuild
    cargo build ${lib.concatStringsSep " " (cargoFlags features.default)}
    runHook postBuild
  '';
  
  installPhase = ''
    runHook preInstall
    mkdir -p $out/bin
    cp target/release/${pname} $out/bin/
    runHook postInstall
  '';
  
  # Test configuration
  checkPhase = ''
    runHook preCheck
    cargo test --release --features ${mkFeatureFlags features.default}
    runHook postCheck
  '';
  
  # Package variants
  variants = rec {
    # Default variant
    default = buildPackage features.default;
    
    # Minimal variant
    minimal = buildPackage features.minimal;
    
    # Full-featured variant
    full = buildPackage features.all;
    
    # Development variant with debug symbols
    dev = buildPackage features.all // {
      cargoFlags = [ "--features" (mkFeatureFlags features.all) ];
      dontStrip = true;
    };
  };
  
  # Main build function
  buildPackage = selectedFeatures: rustPlatform.buildRustPackage rec {
    inherit pname version src buildInputs nativeBuildInputs;
    inherit preBuild buildPhase installPhase checkPhase;
    
    cargoLock = {
      lockFile = "${src}/Cargo.lock";
      outputHashes = {
        "tree-sitter-nix-0.3.0" = lib.fakeSha256;
        "custom-parser-0.1.0" = lib.fakeSha256;
      };
    };
    
    # Feature-specific configuration
    cargoBuildFlags = cargoFlags selectedFeatures;
    cargoTestFlags = cargoFlags selectedFeatures;
    
    # Environment variables
    env = {
      RUSTFLAGS = "-C target-cpu=native";
      CARGO_PROFILE_RELEASE_LTO = "fat";
    } // lib.optionalAttrs isDarwin {
      MACOSX_DEPLOYMENT_TARGET = "10.12";
    };
    
    # Metadata
    meta = with lib; {
      description = "Advanced code intelligence and analysis tool";
      longDescription = ''
        Codanna is a powerful code intelligence system that provides
        advanced symbol extraction, relationship tracking, and semantic
        analysis across multiple programming languages including Rust,
        Go, Python, TypeScript, PHP, and Nix.
      '';
      
      homepage = "https://github.com/anthropics/codanna";
      license = licenses.mit;
      maintainers = with maintainers; [ ];
      platforms = platforms.unix;
      
      # Indicate this is a CLI tool
      mainProgram = pname;
    };
    
    # Post-installation checks
    doInstallCheck = true;
    installCheckPhase = ''
      runHook preInstallCheck
      $out/bin/${pname} --version
      $out/bin/${pname} --help
      runHook postInstallCheck
    '';
  };
  
  # Utility functions for package configuration
  utils = {
    # Generate feature combinations
    featureCombinations = lib.powerset features.all;
    
    # Validate feature set
    validateFeatures = featureSet:
      lib.all (feature: lib.elem feature features.all) featureSet;
    
    # Get package size estimate
    estimateSize = featureSet: 
      builtins.length featureSet * 1024 * 1024; # Rough estimate
  };
  
in variants.default // {
  # Export variants and utilities
  inherit variants utils;
  
  # Export configuration for external use
  config = {
    inherit pname version src features;
    inherit buildInputs nativeBuildInputs;
  };
}