# clap-wrapper-rs

[![Validate](https://github.com/blepfx/clap-wrapper-rs/actions/workflows/validate.yml/badge.svg)](https://github.com/blepfx/clap-wrapper-rs/actions/workflows/validate.yml)
[![Crates](https://img.shields.io/crates/v/clap-wrapper)](https://crates.io/crates/clap-wrapper)

An easy way to use [clap-wrapper](https://github.com/free-audio/clap-wrapper) in your Rust plugins!

## Usecases
- Adding VST3 or AUv2 support to existing Rust plugin frameworks that do not support them (e.g. [clack](https://github.com/prokopyl/clack))
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
clap-wrapper = { version = "0.2.0", features = ["vst3", "auv2", "parallel"] } # these features are enabled by default
```
    
Then, in your `lib.rs`:
```rust
// exports `GetPluginFactoryAUV2` symbol.
clap_wrapper::export_auv2!(); 
// exports `GetPluginFactory` symbol and extra VST3 symbols.
clap_wrapper::export_vst3!(); 
```

This will export VST3 and AUv2 entrypoints that use the `clap_entry` symbol exported from your crate (as an example, `nih_plug::nih_export_clap` exports it).

Keep in mind, that `clap-wrapper-rs` only adds the necessary entrypoints that reexport the CLAP plugin you already have. You'd still have to use a crate like `nih-plug` to actually create the plugin.


After building, you have to manually "bundle" your plugin. This means setting up the correct directory structure and copying the necessary files. See [VST 3 Developer Portal: Plug-in Format Structure](https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Plugin+Format.html) for more info about VST3 directory structure. For AUv2, the directory structure is similar. 
Note that when building for MacOS you have to add a `Info.plist` file yourself.
Check out [Info.vst3.plist](examples/example-clack/Info.vst3.plist) and [Info.auv2.plist](examples/example-clack/Info.auv2.plist) for an example of what `Info.plist` should look like.


See [validate.yml](.github/workflows/validate.yml) for a complete example of how to build, bundle and validate a plugin.

## Changelog

- 0.2.0:
    - Embed VST3 and AUv2 SDKs directly into the crate, removing the need to download them separately. This is possible thanks to VST3 SDK's new MIT license. 
    - Added `vst3` and `auv2` features to enable/disable building those wrappers.
    - Simplified build.rs by a lot.
- 0.1.2:
    - Updated `clap-wrapper` to latest.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.