#[macro_export]
macro_rules! export {
    () => {
        #[allow(unused_imports)]
        pub use $crate::internal::*;
    };
}

#[doc(hidden)]
pub mod internal {
    #[cfg(target_os = "macos")]
    #[link(name = "clap_wrapper_auv2")]
    unsafe extern "C" {
        pub unsafe fn wrapAsAUV2_inst0Factory(
            desc: *const core::ffi::c_void,
        ) -> *mut core::ffi::c_void;
    }

    #[cfg(target_os = "macos")]
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn GetPluginFactoryAUV2(
        desc: *const core::ffi::c_void,
    ) -> *mut core::ffi::c_void {
        unsafe { wrapAsAUV2_inst0Factory(desc) }
    }
}
