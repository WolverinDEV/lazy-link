use proc_macro::TokenStream;
use syn::parse_macro_input;

mod macro_lazy_link;

/// Marks an external block to be resolved by the specified resolver.
///
/// The following attributes are supported:
///
/// - `resolver = "<function_name>"`  
///   Specify the resolver function that will be used to look up the external function by name.
///
/// - `module = "<any_value>"`  
///   Provide a value for the `module` parameter in the resolver function. This can be useful for
///   grouping or identifying external functions in different modules.
///
/// - `cache = "none" | "static" | "static-atomic"`  
///   Specify if and how the resolved values should be cached:
///   - `"none"`: No caching, the function is resolved every time it is called.
///   - `"static"`: Cache the resolved function using a static variable, ensuring the function is
///     only resolved once.
///   - `"static-atomic"`: Use atomic operations to cache the resolved function for thread safety
///     in multi-threaded environments.
#[proc_macro_attribute]
pub fn lazy_link(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let attr = parse_macro_input!(attr);
    macro_lazy_link::lazy_link(attr, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
