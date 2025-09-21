# Attribute set patterns and constructs
# Tests various attribute set syntaxes and features

{
  # Simple attribute set
  simple = {
    name = "test";
    value = 42;
  };
  
  # Recursive attribute set
  recursive = rec {
    a = 10;
    b = a * 2;
    c = b + a;
    total = a + b + c;
  };
  
  # Nested attribute sets
  nested = {
    server = {
      host = "localhost";
      port = 8080;
      ssl = {
        enabled = true;
        cert = "/path/to/cert.pem";
        key = "/path/to/key.pem";
      };
    };
    
    database = {
      host = "db.localhost";
      port = 5432;
      credentials = {
        username = "dbuser";
        password = "secret";
      };
    };
    
    cache = {
      redis = {
        host = "redis.localhost";
        port = 6379;
      };
    };
  };
  
  # Attribute set with inherit
  inherited = let
    name = "myapp";
    version = "2.1.0";
    description = "My application";
  in {
    inherit name version description;
    fullName = "${name}-${version}";
  };
  
  # Attribute set with inherit from
  inheritedFrom = let
    base = { x = 1; y = 2; z = 3; };
  in {
    inherit (base) x y;
    w = base.z * 2;
  };
  
  # Dynamic attribute names
  dynamic = let
    key1 = "dynamic-key";
    key2 = "another-key";
  in {
    ${key1} = "dynamic value 1";
    ${key2} = "dynamic value 2";
    "static-key" = "static value";
  };
  
  # Attribute set with functions
  withFunctions = {
    getValue = key: default: attrs:
      if builtins.hasAttr key attrs
      then builtins.getAttr key attrs
      else default;
    
    mapAttrs = f: attrs:
      builtins.listToAttrs (
        builtins.map (name: {
          inherit name;
          value = f name (builtins.getAttr name attrs);
        }) (builtins.attrNames attrs)
      );
  };
  
  # Conditional attributes
  conditional = {
    always = "present";
  } // (if true then { sometimes = "present too"; } else {});
  
  # Complex merge example
  merged = let
    base = {
      name = "base";
      features = [ "basic" ];
      config = { debug = false; };
    };
    
    override = {
      features = [ "advanced" "extra" ];
      config = { debug = true; logging = true; };
    };
  in base // override // {
    # Merge lists and nested attributes properly
    features = base.features ++ override.features;
    config = base.config // override.config;
  };
}