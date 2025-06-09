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

#[macro_export]
macro_rules! export_auv2 {
    () => {
        #[allow(unused_imports)]
        pub use $crate::auv2::*;
    };
}

#[macro_export]
macro_rules! export_vst3 {
    () => {
        #[allow(unused_imports)]
        pub use $crate::vst3::*;
    };
}
