#[link(name = "clap_wrapper_auv2")]
unsafe extern "C" {
    pub unsafe fn wrapAsAUV2_inst0Factory(desc: *const core::ffi::c_void)
    -> *mut core::ffi::c_void;
}

pub unsafe fn GetPluginFactoryAUV2(desc: *const core::ffi::c_void) -> *mut core::ffi::c_void {
    wrapAsAUV2_inst0Factory(desc)
}
