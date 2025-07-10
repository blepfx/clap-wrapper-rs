# clap-wrapper-rs

[![Validate](https://github.com/blepfx/clap-wrapper-rs/actions/workflows/validate.yml/badge.svg)](https://github.com/blepfx/clap-wrapper-rs/actions/workflows/validate.yml)
[![Crates](https://img.shields.io/crates/v/clap-wrapper)](https://crates.io/crates/clap-wrapper)

An easy way to use [clap-wrapper](https://github.com/free-audio/clap-wrapper) in your Rust plugins!

## Usecases
- Adding VST3 or AUv2 support to existing Rust plugin frameworks that do not support them
- Using [nih-plug](https://github.com/robbert-vdh/nih-plug) with non-GPLv3 licensed VST3 SDK
- Making your own audio plugin framework without dealing with VST3 and AUv2 directly

## Features
- Provides a simple way to export Rust-based CLAP plugins as VST3 and AUv2 plugins.
- Builds "fat", self-contained binaries for VST3 and AUv2 plugins.
- Does not use `cmake`. Instead it uses the `cc` crate to compile the `clap-wrapper` code.
- Tested on Linux (Ubuntu 22.04), MacOS (13.7) and Windows (10). In theory the minimum supported OSX version is 10.12, but I have no way to test that.

## Limitations
- Currently only supports VST3 and AUv2 plugins. Standalone builds are not supported yet.
- AUv2 wrapper can only export a single plugin per binary. If `clap_entry` exports multiple plugins,
  only the first one will be exported.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
clap-wrapper = "0.1.2"
```
    
Then, in your `lib.rs`:
```rust
// exports `GetPluginFactoryAUV2` symbol if CLAP_WRAPPER_AUV2_SDK env variable is present
clap_wrapper::export_auv2!(); 
// exports `GetPluginFactory` symbol and extra VST3 symbols if CLAP_WRAPPER_VST3_SDK env variable is present
clap_wrapper::export_vst3!(); 
```

This will export VST3 and AUv2 entrypoints that use the `clap_entry` symbol exported from your crate (as an example, `nih_plug::nih_export_clap` exports it).

To build the plugin with VST3 or AUv2 capabilities, add `CLAP_WRAPPER_VST3_SDK` and/or `CLAP_WRAPPER_AUV2_SDK` environment variables to your build command. For example:

```bash
CLAP_WRAPPER_VST3_SDK=/path/to/vst3sdk cargo build -p example-clap
```

Keep in mind, that `clap-wrapper-rs` only adds the necessary entrypoints that reexport the CLAP plugin you already have. You'd still have to use a crate like `nih-plug` to actually create the plugin.


After building, you have to manually "bundle" your plugin. This means setting up the correct directory structure and copying the necessary files. See [VST 3 Developer Portal: Plug-in Format Structure](https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Plugin+Format.html) for more info about VST3 directory structure. For AUv2, the directory structure is similar. 
Note that when building for MacOS you have to add a `Info.plist` file yourself.
Check out [Info.vst3.plist](examples/example-clack/Info.vst3.plist) and [Info.auv2.plist](examples/example-clack/Info.auv2.plist) for an example of what `Info.plist` should look like.


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