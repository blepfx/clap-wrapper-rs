#pragma once
#include "detail/auv2/auv2_base_classes.h"

AUV2_Type typeFromAUInstance(AudioComponentInstance &ci)
{
    const AudioComponent comp = AudioComponentInstanceGetComponent(ci);
    if (comp == nullptr)
        return AUV2_Type::aumu_musicdevice;

    AudioComponentDescription desc = AudioComponentGetDescription(comp, outDesc);
    if (desc.componentType == "aumu")
        return AUV2_Type::aumu_musicdevice;
    if (desc.componentType == "aufx")
        return AUV2_Type::aufx_effect;
    if (desc.componentType == "aumi")
        return AUV2_Type::aumi_noteeffect;

    return AUV2_Type::aumu_musicdevice;
}

struct wrapAsAUV2_inst0 : free_audio::auv2_wrapper::WrapAsAUV2
{
    wrapAsAUV2_inst0(AudioComponentInstance ci) : free_audio::auv2_wrapper::WrapAsAUV2(typeFromAUInstance(&ci), "", "", 0, ci)
    {
    }
}

AUSDK_COMPONENT_ENTRY(ausdk::AUMusicDeviceFactory, wrapAsAUV2_inst0);
