# lazy_link &emsp; [![Latest Version]][crates.io] [![License: GPL v3]](./LICENSE) [![GitHub build status]][actions]

[License: GPL v3]: https://img.shields.io/badge/License-GPLv3-blue.svg
[Latest Version]: https://img.shields.io/crates/v/lazy_link.svg
[crates.io]: https://crates.io/crates/lazy_link
[GitHub build status]: https://github.com/WolverinDEV/lazy-link/workflows/Rust/badge.svg?branch=master
[actions]: https://github.com/WolverinDEV/lazy-link/actions?query=workflow%3ARust

`lazy_link` is a Rust procedural macro crate that allows you to dynamically look up external functions at runtime without any additional boilerplate code.
This crate is designed to simplify the process of dynamically linking to external functions as runtime, inclusing support for function name obfuscation and `no_std` environments.

## Features

- **Dynamic Function Lookup**: Use the `lazy_link` attribute to dynamically link external functions at runtime.
- **No Boilerplate Required**: No need for additional setup or boilerplate code—just annotate your external blocks.
- **Function Name Obfuscation**: Function names can be obfuscated using the `obfstr` crate, enhancing security by making the function identifiers less readable in binary form.
- **Platform and ABI Agnostic**: Works across different platforms and ABIs.
- **`no_std` Compatible**: Full support for `no_std` environments.

## Usage
To use `lazy_link`, simply add the attribute to your external block declarations:
```rust
use lazy_link::lazy_link;

#[lazy_link(resolver = "resolve_externals")]
extern "C" {
    fn external_add(v1: u8, v2: u8) -> u8;
}
```

Additionally, you have to implement the resolve function, which dynamically resolves the function at runtime. This function takes the function's name (and an optional module) and returns a non-null pointer to the function:
```rust
fn resolve_externals(module: Option<&str>, name: &str) -> NonNull<()> {
    // Your resolution logic here, typically using some form of dynamic lookup.
    unimplemented!("Function lookup logic for {}", name)
}
```
  
## Examples
Examples can be found within the examples directory of this repository. These examples demonstrate how to use lazy_link in various contexts, including platform-specific scenarios.

To run the examples, clone the repository and use the following command:
```bash
cargo run --bin <example_name>
```