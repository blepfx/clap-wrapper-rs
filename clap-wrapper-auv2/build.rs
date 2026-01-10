fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let debug = std::env::var("DEBUG").unwrap_or_default() == "true";

    if os != "macos" {
        return;
    }

    let mut cc = cc::Build::new();
    cc.cpp(true).std("c++20"); //latest AudioUnitSDK requires C++20
    cc.flag_if_supported("-fno-char8_t");

    cc.include("./src/cpp");
    cc.include("./external/clap/include");
    cc.include("./external/clap-wrapper/include");
    cc.include("./external/clap-wrapper/libs/fmt");
    cc.include("./external/clap-wrapper/src");
    cc.include("./external/AudioUnitSDK/include");

    cc.define("CLAP_WRAPPER_VERSION", Some("\"0.11.0\""));
    cc.define("CLAP_WRAPPER_BUILD_AUV2", Some("1"));
    cc.define("STATICALLY_LINKED_CLAP_ENTRY", Some("1"));
    cc.define("DICTIONARY_STREAM_FORMAT_WRAPPER", Some("1"));

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
        "./external/clap-wrapper/src/clap_proxy.cpp",
        "./external/clap-wrapper/src/detail/clap/fsutil.cpp",
        "./external/clap-wrapper/src/detail/shared/sha1.cpp",
    ]);

    // clap auv2 wrapper
    cc.files([
        "./external/clap-wrapper/src/wrapasauv2.cpp",
        "./external/clap-wrapper/src/detail/auv2/auv2_shared.mm",
        "./external/clap-wrapper/src/detail/auv2/process.cpp",
        "./external/clap-wrapper/src/detail/auv2/wrappedview.mm",
        "./external/clap-wrapper/src/detail/auv2/parameter.cpp",
    ]);

    cc.file("./external/clap-wrapper/src/detail/os/macos.mm");
    cc.file("./external/clap-wrapper/src/detail/clap/mac_helpers.mm");

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

    println!("cargo:rustc-cfg=clap_wrapper_auv2");
}
