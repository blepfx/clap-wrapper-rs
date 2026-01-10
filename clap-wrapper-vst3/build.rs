use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let debug = std::env::var("DEBUG").unwrap_or_default() == "true";

    let mut cc = cc::Build::new();
    cc.cpp(true).std("c++17");

    // msvc stuff
    cc.flag_if_supported("/utf-8");
    cc.flag_if_supported("/EHsc");

    cc.include("./external/clap/include");
    cc.include("./external/clap-wrapper/include");
    cc.include("./external/clap-wrapper/libs/fmt");
    cc.include("./external/clap-wrapper/libs/psl");
    cc.include("./external/clap-wrapper/src");
    cc.include("./external/vst3sdk");
    cc.include("./external/vst3sdk/public.sdk");
    cc.include("./external/vst3sdk/pluginterfaces");

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
        "./external/clap-wrapper/src/clap_proxy.cpp",
        "./external/clap-wrapper/src/detail/clap/fsutil.cpp",
        "./external/clap-wrapper/src/detail/shared/sha1.cpp",
    ]);

    // clap vst3 wrapper
    cc.files([
        "./external/clap-wrapper/src/wrapasvst3_export_entry.cpp",
        "./external/clap-wrapper/src/wrapasvst3.cpp",
        "./external/clap-wrapper/src/wrapasvst3_entry.cpp",
        "./external/clap-wrapper/src/detail/vst3/parameter.cpp",
        "./external/clap-wrapper/src/detail/vst3/plugview.cpp",
        "./external/clap-wrapper/src/detail/vst3/process.cpp",
        "./external/clap-wrapper/src/detail/vst3/categories.cpp",
    ]);

    if debug {
        cc.define("DEVELOPMENT", Some("1"));
    } else {
        cc.define("RELEASE", Some("1"));
    }

    match os.as_str() {
        "macos" => {
            cc.file("./external/vst3sdk/public.sdk/source/main/macmain.cpp");
            cc.file("./external/clap-wrapper/src/detail/os/macos.mm");
            cc.file("./external/clap-wrapper/src/detail/clap/mac_helpers.mm");

            cc.define("MAC", None);

            // TODO: test on macos below 10.15
            cc.define("MACOS_USE_GHC_FILESYSTEM", None);
            cc.include("./external/filesystem/include");

            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=Foundation");
        }
        "windows" => {
            cc.file("./external/vst3sdk/public.sdk/source/main/dllmain.cpp");
            cc.file("./external/clap-wrapper/src/detail/os/windows.cpp");
            cc.define("WIN", None);

            println!("cargo:rustc-link-lib=Shell32");
            println!("cargo:rustc-link-lib=user32");
            println!("cargo:rustc-link-lib=ole32");
        }
        "linux" => {
            cc.file("./external/vst3sdk/public.sdk/source/main/linuxmain.cpp");
            cc.file("./external/clap-wrapper/src/detail/os/linux.cpp");
            cc.define("LIN", None);
        }
        _ => {
            panic!("Unsupported target OS: {}", os);
        }
    }

    cc.try_compile("clap_wrapper_vst3")
        .unwrap_or_else(|e| panic!("failed to compile clap-wrapper (vst3): {}", e));

    println!("cargo:rustc-cfg=clap_wrapper_vst3");
}

fn walk_files(dir: impl Into<PathBuf>, ext: &str) -> Vec<PathBuf> {
    let mut stack = Vec::new();
    let mut files = Vec::new();

    stack.push(dir.into());
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
