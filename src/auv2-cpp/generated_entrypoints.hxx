#pragma once
#include "detail/auv2/auv2_base_classes.h"

#define CLAP_AUV2_ENTRY(n)                                           \
    struct wrapAsAUV2_inst##n : free_audio::auv2_wrapper::WrapAsAUV2 \
    {                                                                \
        wrapAsAUV2_inst##n(AudioComponentInstance ci)                \
            : free_audio::auv2_wrapper::WrapAsAUV2("", "", n, ci) {} \
    };                                                               \
    AUSDK_COMPONENT_ENTRY(ausdk::AUMusicDeviceFactory, wrapAsAUV2_inst##n);

CLAP_AUV2_ENTRY(0);
CLAP_AUV2_ENTRY(1);
CLAP_AUV2_ENTRY(2);
CLAP_AUV2_ENTRY(3);