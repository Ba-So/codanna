//! Comprehensive Rust test file for parser maturity assessment
//! Tests all major Rust language features and constructs

use std::collections::HashMap;
use std::sync::Arc;
use std::marker::PhantomData;

// Module declaration
mod inner {
    pub struct InnerStruct;
}

// Re-exports
pub use inner::InnerStruct;

// Constants and statics
const MAX_SIZE: usize = 1024;
pub const DEFAULT_NAME: &str = "default";
static mut COUNTER: u32 = 0;
static INSTANCE: std::sync::OnceLock<Config> = std::sync::OnceLock::new();

// Type aliases
type Result<T> = std::result::Result<T, Error>;
type NodeId = u32;
pub type SharedData = Arc<Vec<u8>>;

// Generic type alias with bounds
type Handler<T> = Box<dyn Fn(T) -> Result<()> + Send + Sync>;

// Struct with various field types
#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    port: u16,
    #[deprecated]
    enabled: bool,
    phantom: PhantomData<()>,
}

// Tuple struct
pub struct Point(f64, f64, f64);

// Unit struct
pub struct Marker;

// Struct with lifetime parameters
pub struct BorrowedData<'a> {
    data: &'a str,
    mutable: &'a mut [u8],
}

// Enum with various variants
#[derive(Debug)]
pub enum Status {
    Active,
    Inactive { reason: String },
    Pending(std::time::Duration),
    Complex { id: u32, data: Vec<u8> },
}

// Generic enum
pub enum Option2<T> {
    Some(T),
    None,
}

// Trait with associated types and constants
pub trait Parser {
    type Input;
    type Output;
    type Error: std::error::Error;

    const MAX_DEPTH: usize = 100;

    fn parse(&self, input: Self::Input) -> Result<Self::Output>;

    fn validate(&self, input: &Self::Input) -> bool {
        true
    }

    // Associated function (no self)
    fn new() -> Self where Self: Sized;
}

// Trait with generic methods
pub trait Container<T> {
    fn add(&mut self, item: T);
    fn get(&self, index: usize) -> Option<&T>;
    fn iter(&self) -> impl Iterator<Item = &T>;
}

// Trait with lifetime bounds
pub trait Lifecycle<'a> {
    type Item: 'a;
    fn process(&'a self) -> Self::Item;
}

// Implementation block
impl Config {
    // Associated constant
    pub const DEFAULT_PORT: u16 = 8080;

    // Associated function (constructor)
    pub fn new(name: String) -> Self {
        Self {
            name,
            port: Self::DEFAULT_PORT,
            enabled: true,
            phantom: PhantomData,
        }
    }

    // Method with self
    pub fn port(&self) -> u16 {
        self.port
    }

    // Method with mut self
    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    // Method consuming self
    pub fn into_name(self) -> String {
        self.name
    }

    // Generic method
    pub fn with_data<T>(&self, data: T) -> (Self, T)
    where
        T: Clone,
    {
        (self.clone(), data)
    }

    // Async method
    pub async fn connect(&self) -> Result<()> {
        Ok(())
    }

    // Unsafe method
    pub unsafe fn get_raw_ptr(&self) -> *const u8 {
        &self.port as *const u16 as *const u8
    }
}

// Trait implementation
impl Parser for Config {
    type Input = String;
    type Output = Config;
    type Error = std::io::Error;

    fn parse(&self, input: Self::Input) -> Result<Self::Output> {
        Ok(Config::new(input))
    }

    fn new() -> Self {
        Config::new(String::new())
    }
}

// Generic struct
pub struct GenericContainer<T, U = String>
where
    T: Clone,
{
    items: Vec<T>,
    metadata: U,
}

// Implementation for generic struct
impl<T, U> GenericContainer<T, U>
where
    T: Clone + std::fmt::Debug,
    U: Default,
{
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            metadata: U::default(),
        }
    }

    pub fn add(&mut self, item: T) {
        self.items.push(item);
    }
}

// Implementation with trait bounds
impl<T> Container<T> for GenericContainer<T>
where
    T: Clone,
{
    fn add(&mut self, item: T) {
        self.items.push(item);
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }
}

// Function with various parameter types
pub fn complex_function<'a, T, U>(
    reference: &'a str,
    mutable: &mut Vec<T>,
    owned: String,
    generic: U,
    closure: impl Fn() -> T,
) -> Result<&'a str>
where
    T: Clone + 'a,
    U: std::fmt::Debug,
{
    mutable.push(closure());
    Ok(reference)
}

// Async function
pub async fn async_operation(url: &str) -> Result<String> {
    Ok(url.to_string())
}

// Const function
pub const fn const_function(x: u32) -> u32 {
    x * 2
}

// Unsafe function
pub unsafe fn unsafe_operation(ptr: *mut u8) {
    *ptr = 0;
}

// Function with impl Trait return
pub fn returns_impl_trait() -> impl std::fmt::Display {
    "hello"
}

// Function with dyn Trait parameter
pub fn takes_dyn_trait(parser: &dyn Parser<Input = String, Output = Config, Error = std::io::Error>) {
    // ...
}

// Higher-ranked trait bounds (HRTB)
pub fn higher_ranked<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    f("test");
}

// Macro definition
macro_rules! create_function {
    ($name:ident) => {
        fn $name() {
            println!("Function: {}", stringify!($name));
        }
    };
}

// Macro invocation
create_function!(generated_func);

// Union (unsafe)
#[repr(C)]
union MyUnion {
    f1: u32,
    f2: f32,
}

// Extern block
extern "C" {
    fn external_function(x: i32) -> i32;
}

// Error type
#[derive(Debug)]
pub struct Error {
    message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

// Test module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config = Config::new("test".to_string());
        assert_eq!(config.port(), Config::DEFAULT_PORT);
    }

    #[test]
    #[ignore]
    fn ignored_test() {
        // ...
    }
}

// Benchmark (nightly only)
#[cfg(all(test, feature = "unstable"))]
mod benches {
    use test::Bencher;

    #[bench]
    fn bench_create(b: &mut Bencher) {
        b.iter(|| Config::new("bench".to_string()));
    }
}

// Main function
fn main() {
    let config = Config::new("app".to_string());
    println!("Config: {:?}", config);
}