# `nih-plug` example for `clap-wrapper-rs`

This example demonstrates how to use **[nih-plug](https://github.com/robbert-vdh/nih-plug)** with `clap-wrapper-rs` to make a VST3/AUv2 audio plugin. 
It is based on the [gain](https://github.com/robbert-vdh/nih-plug/tree/master/plugins/examples/gain) example from the `nih-plug` repository.

Note: make sure to not use `nih_plug::nih_export_vst3` macro alongside with `clap_wrapper::export`. This is because `clap-wrapper-rs` will export the necessary symbols for you if the VST3 SDK path is set, and doing so would result in a symbol name collision.


See [Info.auv2.plist](Info.auv2.plist) and [Info.vst3.plist](Info.vst3.plist) for the examples of required MacOS bundle metadata.

## License

Licensed under the ISC license ([LICENSE-ISC](LICENSE-ISC) or http://opensource.org/licenses/ISC).