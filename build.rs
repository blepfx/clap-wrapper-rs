use std::{
    path::{Path, PathBuf, absolute},
    time::SystemTime,
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CLAP_WRAPPER_CPP_DIR");

    let clap_wrapper_dir = std::env::var_os("CLAP_WRAPPER_CPP_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./external/clap-wrapper"));
    println!("cargo:rerun-if-changed={}", clap_wrapper_dir.display());

    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let debug = std::env::var("DEBUG").unwrap() == "true";

    if cfg!(feature = "vst3") {
        build_vst3(&os, debug, &clap_wrapper_dir);
    }

    if cfg!(feature = "auv2") && os == "macos" {
        build_auv2(debug, &clap_wrapper_dir);
    }
}

fn build_vst3(os: &str, debug: bool, clap_wrapper_dir: &Path) {
    let mut cc = cc::Build::new();
    cc.cpp(true).std("c++17");

    // msvc stuff
    cc.flag_if_supported("/utf-8");
    cc.flag_if_supported("/EHsc");

    cc.include("./external/clap/include");
    cc.include(clap_wrapper_dir.join("include"));
    cc.include(clap_wrapper_dir.join("libs/fmt"));
    cc.include(clap_wrapper_dir.join("libs/psl"));
    cc.include(clap_wrapper_dir.join("src"));
    cc.include("./external/vst3sdk");
    cc.include("./external/vst3sdk/public.sdk");
    cc.include("./external/vst3sdk/pluginterfaces");

    cc.define("CLAP_WRAPPER_VERSION", Some("\"0.14.0\""));
    cc.define("CLAP_WRAPPER_BUILD_FOR_VST3", Some("1"));
    cc.define("STATICALLY_LINKED_CLAP_ENTRY", Some("1"));

    // absolutely cursed fucked up evil and twisted way to "reexport" the entry point symbols
    // FIXME: if anyone knows a better way to do this, please let me know
    cc.define("GetPluginFactory", Some("clap_wrapper_GetPluginFactory"));
    cc.define("ModuleEntry", Some("clap_wrapper_ModuleEntry"));
    cc.define("ModuleExit", Some("clap_wrapper_ModuleExit"));
    cc.define("InitDll", Some("clap_wrapper_InitDll"));
    cc.define("ExitDll", Some("clap_wrapper_ExitDll"));
    cc.define("bundleEntry", Some("clap_wrapper_bundleEntry"));
    cc.define("bundleExit", Some("clap_wrapper_bundleExit"));

    // vst3 sdk
    cc.files(walk_files("./external/vst3sdk/base/source", "cpp"));
    cc.files(walk_files("./external/vst3sdk/base/thread/source", "cpp"));
    cc.files(walk_files("./external/vst3sdk/pluginterfaces/base", "cpp"));
    cc.files(walk_files(
        "./external/vst3sdk/public.sdk/source/common",
        "cpp",
    ));
    cc.files([
        "./external/vst3sdk/public.sdk/source/main/pluginfactory.cpp",
        "./external/vst3sdk/public.sdk/source/main/moduleinit.cpp",
        "./external/vst3sdk/public.sdk/source/vst/vstinitiids.cpp",
        "./external/vst3sdk/public.sdk/source/vst/vstnoteexpressiontypes.cpp",
        "./external/vst3sdk/public.sdk/source/vst/vstaudioeffect.cpp",
        "./external/vst3sdk/public.sdk/source/vst/vstcomponent.cpp",
        "./external/vst3sdk/public.sdk/source/vst/vstsinglecomponenteffect.cpp",
        "./external/vst3sdk/public.sdk/source/vst/vstcomponentbase.cpp",
        "./external/vst3sdk/public.sdk/source/vst/vstbus.cpp",
        "./external/vst3sdk/public.sdk/source/vst/vstparameters.cpp",
        "./external/vst3sdk/public.sdk/source/vst/utility/stringconvert.cpp",
    ]);

    // clap wrapper shared
    cc.files([
        clap_wrapper_dir.join("src/clap_proxy.cpp"),
        clap_wrapper_dir.join("src/detail/clap/fsutil.cpp"),
        clap_wrapper_dir.join("src/detail/shared/sha1.cpp"),
    ]);

    // clap vst3 wrapper
    cc.files([
        clap_wrapper_dir.join("src/wrapasvst3_export_entry.cpp"),
        clap_wrapper_dir.join("src/wrapasvst3.cpp"),
        clap_wrapper_dir.join("src/wrapasvst3_entry.cpp"),
        clap_wrapper_dir.join("src/detail/vst3/parameter.cpp"),
        clap_wrapper_dir.join("src/detail/vst3/plugview.cpp"),
        clap_wrapper_dir.join("src/detail/vst3/process.cpp"),
        clap_wrapper_dir.join("src/detail/vst3/categories.cpp"),
    ]);

    if debug {
        cc.define("DEVELOPMENT", Some("1"));
    } else {
        cc.define("RELEASE", Some("1"));
    }

    match os {
        "macos" => {
            cc.file("./external/vst3sdk/public.sdk/source/main/macmain.cpp");
            cc.file(clap_wrapper_dir.join("src/detail/os/macos.mm"));
            cc.file(clap_wrapper_dir.join("src/detail/clap/mac_helpers.mm"));

            cc.define("MAC", None);

            // TODO: test on macos below 10.15
            cc.define("MACOS_USE_GHC_FILESYSTEM", None);
            cc.include("./external/filesystem/include");

            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=Foundation");
        }
        "windows" => {
            cc.file("./external/vst3sdk/public.sdk/source/main/dllmain.cpp");
            cc.file(clap_wrapper_dir.join("src/detail/os/windows.cpp"));
            cc.define("WIN", None);

            println!("cargo:rustc-link-lib=Shell32");
            println!("cargo:rustc-link-lib=user32");
            println!("cargo:rustc-link-lib=ole32");
        }
        "linux" => {
            cc.file("./external/vst3sdk/public.sdk/source/main/linuxmain.cpp");
            cc.file(clap_wrapper_dir.join("src/detail/os/linux.cpp"));
            cc.define("LIN", None);
        }
        _ => {
            panic!("Unsupported target OS: {}", os);
        }
    }

    cc.try_compile("clap_wrapper_vst3")
        .unwrap_or_else(|e| panic!("failed to compile clap-wrapper (vst3): {}", e));
}

fn build_auv2(debug: bool, clap_wrapper_dir: &Path) {
    let mut cc = cc::Build::new();
    cc.cpp(true).std("c++17"); //AudioUnitSDK requires C++17 (rolled back to 1.1.0)
    cc.flag_if_supported("-fno-char8_t");

    cc.include("./src/auv2-cpp");
    cc.include("./external/clap/include");
    cc.include(clap_wrapper_dir.join("include"));
    cc.include(clap_wrapper_dir.join("libs/fmt"));
    cc.include(clap_wrapper_dir.join("src"));
    cc.include("./external/AudioUnitSDK/include");

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    cc.define("CLAP_WRAPPER_VERSION", Some("\"0.14.0\""));
    cc.define("CLAP_WRAPPER_BUILD_AUV2", Some("1"));
    cc.define("STATICALLY_LINKED_CLAP_ENTRY", Some("1"));
    cc.define("DICTIONARY_STREAM_FORMAT_WRAPPER", Some("1"));
    cc.define(
        "CLAP_WRAPPER_COCOA_CLASS_NSVIEW",
        Some(format!("wrapAsAUV2_cocoaUI_nsview_{}", time).as_str()),
    );
    cc.define(
        "CLAP_WRAPPER_COCOA_CLASS",
        Some(format!("wrapAsAUV2_cocoaUI_{}", time).as_str()),
    );

    // auv2 sdk
    cc.files([
        "./external/AudioUnitSDK/src/AudioUnitSDK/AUBase.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/AUBuffer.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/AUBufferAllocator.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/AUEffectBase.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/AUMIDIBase.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/AUMIDIEffectBase.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/AUInputElement.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/AUOutputElement.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/AUScopeElement.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/AUPlugInDispatch.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/ComponentBase.cpp",
        "./external/AudioUnitSDK/src/AudioUnitSDK/MusicDeviceBase.cpp",
    ]);

    // clap wrapper shared
    cc.files([
        clap_wrapper_dir.join("src/clap_proxy.cpp"),
        clap_wrapper_dir.join("src/detail/clap/fsutil.cpp"),
        clap_wrapper_dir.join("src/detail/shared/sha1.cpp"),
    ]);

    // clap auv2 wrapper
    cc.files([
        clap_wrapper_dir.join("src/wrapasauv2.cpp"),
        clap_wrapper_dir.join("src/detail/auv2/auv2_shared.mm"),
        clap_wrapper_dir.join("src/detail/auv2/process.cpp"),
        clap_wrapper_dir.join("src/detail/auv2/wrappedview.mm"),
        clap_wrapper_dir.join("src/detail/auv2/parameter.cpp"),
    ]);

    cc.file(clap_wrapper_dir.join("src/detail/os/macos.mm"));
    cc.file(clap_wrapper_dir.join("src/detail/clap/mac_helpers.mm"));

    // TODO: test on macos below 10.15
    cc.define("MACOS_USE_GHC_FILESYSTEM", None);
    cc.include("./external/filesystem/include");

    cc.define("MAC", None);

    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=AudioToolbox");
    println!("cargo:rustc-link-lib=framework=CoreMIDI");
    println!("cargo:rustc-link-lib=framework=AppKit");

    if debug {
        cc.define("DEVELOPMENT", Some("1"));
    } else {
        cc.define("RELEASE", Some("1"));
    }

    cc.try_compile("clap_wrapper_auv2")
        .unwrap_or_else(|e| panic!("failed to compile clap-wrapper (auv2): {}", e));
}

fn walk_files(dir: impl Into<PathBuf>, ext: &str) -> Vec<PathBuf> {
    let mut stack = Vec::new();
    let mut files = Vec::new();

    stack.push(dir.into());
    while let Some(top) = stack.pop() {
        for entry in std::fs::read_dir(&top).unwrap_or_else(|_| {
            panic!(
                "failed to list directory {:?}",
                absolute(&top).unwrap_or(top.clone())
            )
        }) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == ext) {
                files.push(path);
            }
        }
    }

    files
}
