/*
    Copyright (C) 2025  John Melton G0ORX/N6LYT

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::env;
use std::fs;
use std::path::Path;

fn main() {

    let fftw = pkg_config::probe_library("fftw3")
        .expect("Could not find fftw3. Please install it via your package manager.");

    let mut build = cc::Build::new();
    build.files([
"libspecbleach/src/processors/specbleach_adenoiser.c",
"libspecbleach/src/processors/specbleach_denoiser.c",
"libspecbleach/src/processors/adaptivedenoiser/adaptive_denoiser.c",
"libspecbleach/src/processors/denoiser/spectral_denoiser.c",
"libspecbleach/src/shared/stft/stft_windows.c",
"libspecbleach/src/shared/stft/fft_transform.c",
"libspecbleach/src/shared/stft/stft_buffer.c",
"libspecbleach/src/shared/stft/stft_processor.c",
"libspecbleach/src/shared/noise_estimation/noise_estimator.c",
"libspecbleach/src/shared/noise_estimation/noise_profile.c",
"libspecbleach/src/shared/noise_estimation/adaptive_noise_estimator.c",
"libspecbleach/src/shared/utils/general_utils.c",
"libspecbleach/src/shared/utils/spectral_features.c",
"libspecbleach/src/shared/utils/spectral_trailing_buffer.c",
"libspecbleach/src/shared/utils/denoise_mixer.c",
"libspecbleach/src/shared/utils/spectral_utils.c",
"libspecbleach/src/shared/gain_estimation/gain_estimators.c",
"libspecbleach/src/shared/post_estimation/spectral_whitening.c",
"libspecbleach/src/shared/post_estimation/noise_floor_manager.c",
"libspecbleach/src/shared/post_estimation/postfilter.c",
"libspecbleach/src/shared/pre_estimation/absolute_hearing_thresholds.c",
"libspecbleach/src/shared/pre_estimation/spectral_smoother.c",
"libspecbleach/src/shared/pre_estimation/noise_scaling_criterias.c",
"libspecbleach/src/shared/pre_estimation/critical_bands.c",
"libspecbleach/src/shared/pre_estimation/masking_estimator.c",
"libspecbleach/src/shared/pre_estimation/transient_detector.c",
"rnnoise/src/denoise.c",
"rnnoise/src/celt_lpc.c",
"rnnoise/src/kiss_fft.c",
"rnnoise/src/nnet.c",
"rnnoise/src/nnet_default.c",
"rnnoise/src/parse_lpcnet_weights.c",
"rnnoise/src/pitch.c",
"rnnoise/src/rnn.c",
"rnnoise/src/rnnoise_data.c",
"rnnoise/src/rnnoise_tables.c",
"wdsp/FDnoiseIQ.c",
"wdsp/calculus.c",
"wdsp/emnr.c",
"wdsp/icfir.c",
"wdsp/meter.c",
"wdsp/shift.c",
"wdsp/RXA.c",
"wdsp/cblock.c",
"wdsp/emph.c",
"wdsp/iir.c",
"wdsp/meterlog10.c",
"wdsp/siphon.c",
"wdsp/TXA.c",
"wdsp/cfcomp.c",
"wdsp/eq.c",
"wdsp/impulse_cache.c",
"wdsp/nbp.c",
"wdsp/slew.c",
"wdsp/amd.c",
"wdsp/cfir.c",
"wdsp/fcurve.c",
"wdsp/iobuffs.c",
"wdsp/nob.c",
"wdsp/snb.c",
"wdsp/ammod.c",
"wdsp/channel.c",
"wdsp/fir.c",
"wdsp/iqc.c",
"wdsp/nobII.c",
"wdsp/ssql.c",
"wdsp/amsq.c",
"wdsp/cmath.c",
"wdsp/firmin.c",
"wdsp/linux_port.c",
"wdsp/osctrl.c",
"wdsp/syncbuffs.c",
"wdsp/analyzer.c",
"wdsp/compress.c",
"wdsp/fmd.c",
"wdsp/lmath.c",
"wdsp/patchpanel.c",
"wdsp/utilities.c",
"wdsp/anf.c",
"wdsp/delay.c",
"wdsp/fmmod.c",
"wdsp/main.c",
"wdsp/resample.c",
"wdsp/varsamp.c",
"wdsp/anr.c",
"wdsp/dexp.c",
"wdsp/fmsq.c",
"wdsp/rmatch.c",
"wdsp/version.c",
"wdsp/apfshadow.c",
"wdsp/div.c",
"wdsp/gain.c",
"wdsp/rnnr.c",
"wdsp/wcpAGC.c",
"wdsp/bandpass.c",
"wdsp/doublepole.c",
"wdsp/gaussian.c",
"wdsp/sbnr.c",
"wdsp/wisdom.c",
"wdsp/calcc.c",
"wdsp/eer.c",
"wdsp/gen.c",
"wdsp/matchedCW.c",
"wdsp/sender.c",
"wdsp/zetaHat.c",
    ]);

    // Include directory for headers
    build.include("libspecbleach/include");
    build.include("libspecbleach/src");
    build.include("libspecbleach/src/processors");
    build.include("libspecbleach/src/processors/adaptivedenoiser");
    build.include("libspecbleach/src/processors/denoiser");
    build.include("libspecbleach/src/shared/stft");
    build.include("libspecbleach/src/shared/noise_estimation");
    build.include("libspecbleach/src/shared");
    build.include("libspecbleach/src/shared/utils");
    build.include("libspecbleach/src/shared/gain_estimation");
    build.include("libspecbleach/src/shared/post_estimation");
    build.include("libspecbleach/src/shared/pre_estimation");
    build.include("wdsp");
    build.include("rnnoise/src");
    build.include("rnnoise/include");

    // Pass specific C flags if the Makefile had them
    build.flag("-O3");
    build.flag("-pthread");
    build.flag("-D_GNU_SOURCE");
    build.flag("-Wno-parentheses");
    build.flag("-march=native");

    for path in fftw.include_paths {
        build.include(path);
    }

    build.compile("wdsp");
    build.compile("libspecbleach");
    build.compile("rnnoise");

    println!("cargo:rustc-link-lib=fftw3");
    println!("cargo:rustc-link-lib=fftw3f");
}
