#[macro_export]
macro_rules! export_vst3 {
    () => {
        #[allow(unused_imports)]
        pub use $crate::vst3::*;
    };
}

#[macro_export]
macro_rules! export_auv2 {
    () => {
        #[allow(unused_imports)]
        pub use $crate::auv2::*;
    };
}

#[cfg(all(target_os = "macos", feature = "auv2"))]
#[doc(hidden)]
pub mod auv2 {
    #[link(name = "clap_wrapper_auv2")]
    unsafe extern "C" {
        pub unsafe fn wrapAsAUV2_inst0Factory(
            desc: *const core::ffi::c_void,
        ) -> *mut core::ffi::c_void;
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn GetPluginFactoryAUV2(
        desc: *const core::ffi::c_void,
    ) -> *mut core::ffi::c_void {
        unsafe { wrapAsAUV2_inst0Factory(desc) }
    }
}

#[cfg(feature = "vst3")]
#[doc(hidden)]
pub mod vst3 {
    #[link(name = "clap_wrapper_vst3")]
    unsafe extern "system" {
        unsafe fn clap_wrapper_GetPluginFactory() -> *mut core::ffi::c_void;
    }

    #[link(name = "clap_wrapper_vst3")]
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    unsafe extern "C" {
        unsafe fn clap_wrapper_ModuleEntry(lib_module: *mut core::ffi::c_void) -> bool;
        unsafe fn clap_wrapper_ModuleExit() -> bool;
    }

    #[link(name = "clap_wrapper_vst3")]
    #[cfg(target_os = "macos")]
    unsafe extern "C" {
        unsafe fn clap_wrapper_bundleEntry(lib_module: *mut core::ffi::c_void) -> bool;
        unsafe fn clap_wrapper_bundleExit() -> bool;
    }

    #[link(name = "clap_wrapper_vst3")]
    #[cfg(target_os = "windows")]
    unsafe extern "system" {
        unsafe fn clap_wrapper_InitDll() -> bool;
        unsafe fn clap_wrapper_ExitDll() -> bool;
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn GetPluginFactory() -> *mut core::ffi::c_void {
        unsafe { clap_wrapper_GetPluginFactory() }
    }

    #[unsafe(no_mangle)]
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    pub extern "C" fn ModuleEntry(lib_module: *mut core::ffi::c_void) -> bool {
        unsafe { clap_wrapper_ModuleEntry(lib_module) }
    }

    #[unsafe(no_mangle)]
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    pub extern "C" fn ModuleExit() -> bool {
        unsafe { clap_wrapper_ModuleExit() }
    }

    #[unsafe(no_mangle)]
    #[cfg(target_os = "macos")]
    pub extern "C" fn bundleEntry(lib_module: *mut core::ffi::c_void) -> bool {
        unsafe { clap_wrapper_bundleEntry(lib_module) }
    }

    #[unsafe(no_mangle)]
    #[cfg(target_os = "macos")]
    pub extern "C" fn bundleExit() -> bool {
        unsafe { clap_wrapper_bundleExit() }
    }

    #[unsafe(no_mangle)]
    #[cfg(target_os = "windows")]
    pub extern "system" fn InitDll() -> bool {
        unsafe { clap_wrapper_InitDll() }
    }

    #[unsafe(no_mangle)]
    #[cfg(target_os = "windows")]
    pub extern "system" fn ExitDll() -> bool {
        unsafe { clap_wrapper_ExitDll() }
    }
}

#[cfg(not(all(target_os = "macos", feature = "auv2")))]
#[doc(hidden)]
pub mod auv2 {}

#[cfg(not(feature = "vst3"))]
#[doc(hidden)]
pub mod vst3 {}
