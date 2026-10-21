+++
title = "Synthphone Vocals"
category = "Synthphone-E / Audio DSP"

[[links]]
label = "View source"
url = "https://github.com/nathansbradshaw/synthphone_e_vocal_dsp"
+++

The audio-processing side of Synthphone-E: pitch correction, vocoding, and formant shifting in Rust.

The library supports musical keys and scales, configurable FFT sizes, and `no_std` builds for embedded hardware. The effects live separately from the hardware and UI code, so they can also be used in desktop tools.
