#![doc = include_str!("../README.md")]
#![allow(non_snake_case)]
#![no_std]

#[cfg(clap_wrapper_vst3)]
#[doc(hidden)]
pub mod vst3;

#[cfg(not(clap_wrapper_vst3))]
#[doc(hidden)]
pub mod vst3 {}

#[cfg(clap_wrapper_auv2)]
#[doc(hidden)]
pub mod auv2;

#[cfg(not(clap_wrapper_auv2))]
#[doc(hidden)]
pub mod auv2 {}

/// Exports an AUv2 entrypoint named `GetPluginFactoryAUV2` that wraps a global CLAP entrypoint (`clap_entry`) exported from the resulting shared library.
/// Currently it only reexports the first plugin available through the CLAP entrypoint, but this limitation might be lifted in the future.
///
/// Requires `CLAP_WRAPPER_AUV2_SDK` environment variable to be set to a valid Apple AudioUnit SDK path, does nothing otherwise.
///
/// Failure to export a CLAP entrypoint might result in linker errors or missing symbols.
#[macro_export]
macro_rules! export_auv2 {
    () => {
        #[allow(unused_imports)]
        pub use $crate::auv2::*;
    };
}

/// Exports a VST3 entrypoint named `GetPluginFactory` that wraps a global CLAP entrypoint (`clap_entry`) exported from the resulting shared library.
///
/// Requires `CLAP_WRAPPER_VST3_SDK` environment variable to be set to a valid VST3 SDK path (GPLv3 or Steinberg Proprietary), does nothing otherwise.
///
/// Failure to export a CLAP entrypoint might result in linker errors or missing symbols.
#[macro_export]
macro_rules! export_vst3 {
    () => {
        #[allow(unused_imports)]
        pub use $crate::vst3::*;
    };
}
