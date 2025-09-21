# Function definitions and patterns in Nix
# Tests various function syntax and patterns

{
  # Simple function
  identity = x: x;
  
  # Function composition
  compose = f: g: x: f (g x);
  
  # Curried function
  add3 = a: b: c: a + b + c;
  
  # Function with pattern matching
  processUser = { name, age ? 25, email ? null }: {
    inherit name age email;
    isAdult = age >= 18;
    displayName = if email != null then "${name} <${email}>" else name;
  };
  
  # Function with ellipsis (rest parameters)
  buildPackage = { pname, version, src, ... }@args: {
    inherit pname version src;
    allArgs = args;
  };
  
  # Recursive function using 'rec'
  rec {
    factorial = n: if n <= 1 then 1 else n * factorial (n - 1);
    fibonacci = n: if n <= 1 then n else fibonacci (n - 1) + fibonacci (n - 2);
  };
  
  # Higher-order functions
  map = f: list: if list == [] then [] else [ (f (builtins.head list)) ] ++ map f (builtins.tail list);
  
  filter = pred: list: if list == [] then [] else
    let
      head = builtins.head list;
      tail = builtins.tail list;
      filtered_tail = filter pred tail;
    in
      if pred head then [ head ] ++ filtered_tail else filtered_tail;
  
  # Function returning function
  multiplier = factor: x: x * factor;
  
  # Complex function with let-in
  processConfig = config: let
    defaults = {
      timeout = 30;
      retries = 3;
      debug = false;
    };
    merged = defaults // config;
    validated = merged // {
      timeout = if merged.timeout < 1 then 1 else merged.timeout;
      retries = if merged.retries < 0 then 0 else merged.retries;
    };
  in validated;
}