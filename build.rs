use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    path::{Path, PathBuf, absolute},
};

fn main() {
    println!("cargo:rerun-if-env-changed=CLAP_WRAPPER_VST3_SDK");
    println!("cargo:rerun-if-env-changed=CLAP_WRAPPER_AUV2_SDK");
    println!("cargo:rerun-if-env-changed=CLAP_WRAPPER_VERBOSE");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(clap_wrapper_vst3)");
    println!("cargo:rustc-check-cfg=cfg(clap_wrapper_auv2)");

    let context = BuildContext {
        os: std::env::var("CARGO_CFG_TARGET_OS").unwrap(),
        debug: std::env::var("DEBUG").unwrap_or_default() == "true",
        verbose: std::env::var("CLAP_WRAPPER_VERBOSE").unwrap_or_default() == "1",
        vst3_sdk: std::env::var_os("CLAP_WRAPPER_VST3_SDK").map(PathBuf::from),
        auv2_sdk: std::env::var_os("CLAP_WRAPPER_AUV2_SDK").map(PathBuf::from),
    };

    build_vst3(&context);
    build_auv2(&context);
}

struct BuildContext {
    os: String,
    debug: bool,
    verbose: bool,

    vst3_sdk: Option<PathBuf>,
    auv2_sdk: Option<PathBuf>,
}

fn build_vst3(context: &BuildContext) {
    let sdk = match context.vst3_sdk {
        Some(ref sdk) => {
            if !sdk.is_dir() {
                panic!(
                    "invalid vst3 sdk path: {}",
                    absolute(sdk).unwrap_or(sdk.clone()).display(),
                )
            }

            sdk
        }
        None => return,
    };

    run_cached(format_args!("vst3-{}", sdk.display()), |dir| {
        let mut cc = cc::Build::new();
        cc.cpp(true).std("c++17");

        // msvc stuff
        cc.flag_if_supported("/utf-8");
        cc.flag_if_supported("/EHsc");

        cc.out_dir(dir);

        cc.include("clap/include");
        cc.include("clap-wrapper/include");
        cc.include("clap-wrapper/libs/fmt");
        cc.include("clap-wrapper/src");
        cc.include(&sdk);
        cc.include(sdk.join("public.sdk"));
        cc.include(sdk.join("pluginterfaces"));

        cc.define("CLAP_WRAPPER_VERSION", Some("\"0.11.0\""));
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
        cc.files(walk_files(sdk.join("base/source"), "cpp"));
        cc.files(walk_files(sdk.join("base/thread/source"), "cpp"));
        cc.files(walk_files(sdk.join("public.sdk/source/common"), "cpp"));
        cc.files(walk_files(sdk.join("pluginterfaces/base"), "cpp"));
        cc.files(
            [
                "public.sdk/source/main/pluginfactory.cpp",
                "public.sdk/source/main/moduleinit.cpp",
                "public.sdk/source/vst/vstinitiids.cpp",
                "public.sdk/source/vst/vstnoteexpressiontypes.cpp",
                "public.sdk/source/vst/vstsinglecomponenteffect.cpp",
                "public.sdk/source/vst/vstaudioeffect.cpp",
                "public.sdk/source/vst/vstcomponent.cpp",
                "public.sdk/source/vst/vstsinglecomponenteffect.cpp",
                "public.sdk/source/vst/vstcomponentbase.cpp",
                "public.sdk/source/vst/vstbus.cpp",
                "public.sdk/source/vst/vstparameters.cpp",
                "public.sdk/source/vst/utility/stringconvert.cpp",
            ]
            .iter()
            .map(|s| sdk.join(s)),
        );

        // clap wrapper shared
        cc.files([
            "clap-wrapper/src/clap_proxy.cpp",
            "clap-wrapper/src/detail/clap/fsutil.cpp",
            "clap-wrapper/src/detail/shared/sha1.cpp",
        ]);

        // clap vst3 wrapper
        cc.files([
            "clap-wrapper/src/wrapasvst3_export_entry.cpp",
            "clap-wrapper/src/wrapasvst3.cpp",
            "clap-wrapper/src/wrapasvst3_entry.cpp",
            "clap-wrapper/src/detail/vst3/parameter.cpp",
            "clap-wrapper/src/detail/vst3/plugview.cpp",
            "clap-wrapper/src/detail/vst3/process.cpp",
            "clap-wrapper/src/detail/vst3/categories.cpp",
        ]);

        if context.debug {
            cc.define("DEVELOPMENT", Some("1"));
        } else {
            cc.define("RELEASE", Some("1"));
        }

        match context.os.as_str() {
            "macos" => {
                cc.file(sdk.join("public.sdk/source/main/macmain.cpp"));
                cc.file("clap-wrapper/src/detail/os/macos.mm");
                cc.file("clap-wrapper/src/detail/clap/mac_helpers.mm");
                cc.define("MACOS_USE_STD_FILESYSTEM", None); //FIXME: this makes the minimum macos version 10.15
                cc.define("MAC", None);

                println!("cargo:rustc-link-lib=framework=CoreFoundation");
                println!("cargo:rustc-link-lib=framework=Foundation");
            }
            "windows" => {
                cc.file(sdk.join("public.sdk/source/main/dllmain.cpp"));
                cc.file("clap-wrapper/src/detail/os/windows.cpp");
                cc.define("WIN", None);
            }
            "linux" => {
                cc.file(sdk.join("public.sdk/source/main/linuxmain.cpp"));
                cc.file("clap-wrapper/src/detail/os/linux.cpp");
                cc.define("LIN", None);
            }
            _ => {
                panic!("Unsupported target OS: {}", context.os);
            }
        }

        if !context.verbose {
            cc.warnings(false);
            cc.cargo_warnings(false);
        }

        cc.try_compile("clap_wrapper_vst3")
            .unwrap_or_else(|e| panic!("failed to compile clap-wrapper (vst3): {}", e));

        println!("cargo:rustc-cfg=clap_wrapper_vst3");
    });
}

/// FIXME: not fully implemented/tested yet
fn build_auv2(context: &BuildContext) {
    let sdk = match context.auv2_sdk {
        Some(ref sdk) if context.os == "macos" => {
            if !sdk.is_dir() {
                panic!(
                    "invalid auv2 sdk path: {}",
                    absolute(sdk).unwrap_or(sdk.clone()).display(),
                )
            }

            sdk
        }
        _ => return,
    };

    run_cached(format_args!("auv2-{}", sdk.display()), |dir| {
        let mut cc = cc::Build::new();
        cc.cpp(true).std("c++20");
        cc.flag_if_supported("-fno-char8_t");
        cc.out_dir(dir);

        cc.include("src/cpp");
        cc.include("clap/include");
        cc.include("clap-wrapper/include");
        cc.include("clap-wrapper/libs/fmt");
        cc.include("clap-wrapper/src");
        cc.include(sdk.join("include"));

        cc.define("CLAP_WRAPPER_VERSION", Some("\"0.11.0\""));
        cc.define("CLAP_WRAPPER_BUILD_AUV2", Some("1"));
        cc.define("STATICALLY_LINKED_CLAP_ENTRY", Some("1"));
        cc.define("DICTIONARY_STREAM_FORMAT_WRAPPER", Some("1"));

        // auv2 sdk
        cc.files(
            [
                "src/AudioUnitSDK/AUBase.cpp",
                "src/AudioUnitSDK/AUBuffer.cpp",
                "src/AudioUnitSDK/AUBufferAllocator.cpp",
                "src/AudioUnitSDK/AUEffectBase.cpp",
                "src/AudioUnitSDK/AUMIDIBase.cpp",
                "src/AudioUnitSDK/AUMIDIEffectBase.cpp",
                "src/AudioUnitSDK/AUInputElement.cpp",
                "src/AudioUnitSDK/AUOutputElement.cpp",
                "src/AudioUnitSDK/AUScopeElement.cpp",
                "src/AudioUnitSDK/AUPlugInDispatch.cpp",
                "src/AudioUnitSDK/ComponentBase.cpp",
                "src/AudioUnitSDK/MusicDeviceBase.cpp",
            ]
            .iter()
            .map(|s| sdk.join(s)),
        );

        // clap wrapper shared
        cc.files([
            "clap-wrapper/src/clap_proxy.cpp",
            "clap-wrapper/src/detail/clap/fsutil.cpp",
            "clap-wrapper/src/detail/shared/sha1.cpp",
        ]);

        // clap auv2 wrapper
        cc.files([
            "clap-wrapper/src/wrapasauv2.cpp",
            "clap-wrapper/src/detail/auv2/auv2_shared.mm",
            "clap-wrapper/src/detail/auv2/process.cpp",
            "clap-wrapper/src/detail/auv2/wrappedview.mm",
            "clap-wrapper/src/detail/auv2/parameter.cpp",
        ]);

        cc.file("clap-wrapper/src/detail/os/macos.mm");
        cc.file("clap-wrapper/src/detail/clap/mac_helpers.mm");
        cc.define("MACOS_USE_STD_FILESYSTEM", None); //FIXME: this makes the minimum macos version 10.15
        cc.define("MAC", None);

        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
        println!("cargo:rustc-link-lib=framework=CoreMIDI");
        println!("cargo:rustc-link-lib=framework=AppKit");

        if context.debug {
            cc.define("DEVELOPMENT", Some("1"));
        } else {
            cc.define("RELEASE", Some("1"));
        }

        if !context.verbose {
            cc.warnings(false);
            cc.cargo_warnings(false);
        }

        cc.try_compile("clap_wrapper_auv2")
            .unwrap_or_else(|e| panic!("failed to compile clap-wrapper (auv2): {}", e));

        println!("cargo:rustc-cfg=clap_wrapper_auv2");
    });
}

fn walk_files(dir: PathBuf, ext: &str) -> Vec<PathBuf> {
    let mut stack = Vec::new();
    let mut files = Vec::new();

    stack.push(dir.clone());
    while let Some(top) = stack.pop() {
        for entry in std::fs::read_dir(&top).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().map_or(false, |e| e == ext) {
                files.push(path);
            }
        }
    }

    files
}

fn run_cached(key: impl Display, f: impl FnOnce(&Path)) {
    let hash = {
        let key = format!("{}", key);
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    };

    let out_path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default());
    let lock_path = out_path.join(format!("build-{}.lock", hash));
    let log_path = out_path.join(format!("build-{}.output", hash));
    let cache_dir = out_path.join(format!("build-{}", hash));

    let output = match std::fs::read_to_string(&log_path).ok() {
        Some(s) => s,
        _ => {
            let stdio = stdio_override::StdoutOverride::from_file(&lock_path).unwrap();
            let result = catch_unwind(AssertUnwindSafe(|| {
                f(&cache_dir);
            }));

            drop(stdio);

            let output = std::fs::read_to_string(&lock_path).unwrap();
            match result {
                Ok(_) => {
                    std::fs::rename(&lock_path, &log_path).unwrap();
                    output
                }
                Err(e) => {
                    println!("{}", output);
                    resume_unwind(e)
                }
            }
        }
    };

    print!("{}", output);
}
