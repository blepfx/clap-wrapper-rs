# clap-wrapper-rs

[![Validate](https://github.com/Quant1um/clap-wrapper-rs/actions/workflows/validate.yml/badge.svg)](https://github.com/Quant1um/clap-wrapper-rs/actions/workflows/validate.yml)

An easy way to use [clap-wrapper](https://github.com/https://github.com/free-audio/clap-wrapper) in your Rust plugins!

## Features
- Provides a simple way to export Rust-based CLAP plugins as VST3 and AUv2 plugins.
- Builds "fat", self-contained binaries for VST3 and AUv2 plugins.
- Does not use `cmake`. Instead it uses the `cc` crate to compile the `clap-wrapper` code.

## Limitations
- Currently only supports VST3 and AUv2 plugins. Standalone builds are not supported yet.
- Currently requires the minimum OSX version to be 10.15 or higher due to C++ `std::filesystem` shenanigans.
- AUv2 wrapper can only export a single plugin per binary. If `clap_entry` exports multiple plugins,
  only the first one will be exported.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
clap-wrapper = { git = "https://github.com/blepfx/clap-wrapper-rs" }
```
    
Then, in your `lib.rs`:
```rust
clap_wrapper::export!();
```

This will export VST3 and AUv2 entrypoints that use the `clap_entry` symbol exported from your crate (as an example, `nih_plug::nih_export_clap` exports it).

To build the plugin with VST3 or AUv2 capabilities, add `CLAP_WRAPPER_VST3_SDK` and/or `CLAP_WRAPPER_AUV2_SDK` environment variables to your build command. For example:

```bash
CLAP_WRAPPER_VST3_SDK=/path/to/vst3sdk cargo build -p example-clap
```

Keep in mind, that `clap-wrapper-rs` only adds the necessary entrypoints that reexport the CLAP plugin you already have. You'd still have to use a crate like `nih-plug` to actually create the plugin.


After building, you have to manually "bundle" your plugin. This means setting up the correct directory structure and copying the necessary files. See [VST 3 Developer Portal: Plug-in Format Structure](https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Plugin+Format.html) for more info about VST3 directory structure. For AUv2, the directory structure is similar. 
Note that when building for MacOS you have to add a `Info.plist` file yourself.
Check out [Info.vst3.plist](examples/example-clap/Info.vst3.plist) and [Info.auv2.plist](examples/example-clap/Info.auv2.plist) for an example of what `Info.plist` should look like.


See [validate.yml](.github/workflows/validate.yml) for a complete example of how to build, bundle and validate a plugin.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.