#pragma once

#define CLAP_WRAPPER_COCOA_CLASS_NSVIEW wrapAsAUV2_inst0_cocoaUI_nsview
#define CLAP_WRAPPER_COCOA_CLASS wrapAsAUV2_inst0_cocoaUI
#define CLAP_WRAPPER_TIMER_CALLBACK timerCallback_wrapAsAUV2_inst0_cocoaUI
#define CLAP_WRAPPER_FILL_AUCV fillAUCV_wrapAsAUV2_inst0_cocoaUI
#include "detail/auv2/wrappedview.asinclude.mm"

bool fillAudioUnitCocoaView(AudioUnitCocoaViewInfo *viewInfo, std::shared_ptr<Clap::Plugin> _plugin)
{
    return fillAUCV_wrapAsAUV2_inst0_cocoaUI(viewInfo);
}