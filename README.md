# pdl [![Travis Build Status](https://travis-ci.org/flier/rust-pdl.svg?branch=master)](https://travis-ci.org/flier/rust-pdl) [![Appveyor Build status](https://ci.appveyor.com/api/projects/status/h2tvdm5uiqtc4mh2?svg=true)](https://ci.appveyor.com/project/flier/rust-pdl) [![crate](https://img.shields.io/crates/v/pdl.svg)](https://crates.io/crates/pdl) [![docs](https://docs.rs/pdl/badge.svg)](https://docs.rs/crate/pdl/) [![dependency status](https://deps.rs/repo/github/flier/rust-pdl/status.svg)](https://deps.rs/repo/github/flier/rust-pdl)

Parse PDL file for [the Chrome DevTools Protocol](https://github.com/ChromeDevTools/devtools-protocol).

**NOTE:** `PDL` (pronounced as `ˈpo͞odl`) is a home-grown format to describe the  DevTools protocol. PDL support, such as Sublime syntax highlighting and the json converter, is available at https://github.com/pavelfeldman/pdl.

## Usage

To use `pdl` in your project, add the following to your Cargo.toml:

```toml
[dependencies]
pdl = "0.1"
```

## Example

Use `pdl::parse` to parse a PDL file as strongly typed data structures.

```rust
let mut f = File::open("browser_protoco.pdl")?;
let mut s = String::new();
f.read_to_string(&mut s)?;

let (rest, proto) = pdl::parse(&s)?;

println!("PDL: {}", proto);
println!("JSON: {}", proto.to_json_pretty());
```

For more detail, please check the `parser` example.

```sh
$ cargo run --example parser -- browser_protocol.pdl --json --output browser_protocol.json
```

## Resources

- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/) - Chrome DevTools Protocol Domain documentation
- [Awesome chrome-devtools](https://github.com/ChromeDevTools/awesome-chrome-devtools) - Awesome tooling and resources in the Chrome DevTools ecosystem
- [devtools-protocol repo](https://github.com/ChromeDevTools/devtools-protocol) - File issues at this repo if you have concerns or problems with the DevTools Protocol (aka Chrome Remote Debugging Protocol).
