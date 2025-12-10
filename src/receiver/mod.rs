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

use serde::{Deserialize, Serialize};

use std::cmp::{max, min};
use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use crate::agc::AGC;
use crate::antenna::Antenna;
use crate::bands::{Bands, BandInfo};
use crate::filters::Filters;
use crate::modes::Modes;
use crate::wdsp::*;

const DEFAULT_SAMPLE_RATE: i32 = 384000; // 1536000;// 768000; // 384000;
const DEFAULT_SPECTRUM_AVERAGE_TIME: f32 = 250.0;
const DEFAULT_WATERFALL_AVERAGE_TIME: f32 = 10.0;
const SUBRX_BASE_CHANNEL: i32 = 16;

#[derive(PartialEq, Serialize, Deserialize, Copy, Clone, Debug)]
pub enum AudioOutput {
    Stereo,
    Left,
    Right,
    Mute,
}

impl AudioOutput {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => AudioOutput::Stereo,
            1 => AudioOutput::Left,
            2 => AudioOutput::Right,
            3 => AudioOutput::Mute,
            _ => AudioOutput::Stereo,
        }
    }

    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Receiver {
    pub protocol: u8,
    pub channel: i32,
    pub active: bool,
    pub adc: usize,
    pub buffer_size: usize,
    pub fft_size: i32,
    pub sample_rate: i32,
    pub sample_rate_changed: bool,
    pub dsp_rate: i32,
    pub output_rate: i32,
    pub output_samples: usize,
    pub band: Bands,
    pub filters_manual: bool,
    pub filters: u32,
    pub frequency: f64,
    pub step_index: usize,
    pub step: f64,
    pub ctun:  bool,
    pub ctun_frequency: f64,
    pub nr: bool,
    pub nr_taps: i32,
    pub nr_delay: i32,
    pub nr_gain: f32,
    pub nr_leak: f32,
    pub nr2: bool,
    pub nb: bool,
    pub nb2: bool,
    pub anf: bool,
    pub anf_taps: i32,
    pub anf_delay: i32,
    pub anf_gain: f32,
    pub anf_leak: f32,
    pub snb: bool,

    pub agc_position: i32,

    pub spectrum_fps: f32,
    pub spectrum_width: i32,
    pub spectrum_step: f32,
    pub waterfall_fps: f32,
    pub waterfall_width: i32,
    pub zoom: i32,
    pub pan: i32,
    pub afgain:  f32,
    pub afpan:  f32,
    pub agc: AGC,
    pub agcgain:  f32,
    pub agcslope:  i32,
    pub agcchangethreshold:  i32,
    pub filter_low: f64,
    pub filter_high: f64,
    pub mode: usize,
    pub filter: usize,
#[serde(skip_serializing, skip_deserializing)]
    pub iq_input_buffer: Vec<f64>,
    pub samples: usize,
    pub audio_output: AudioOutput,
#[serde(skip_serializing, skip_deserializing)]
    pub audio_buffer: Vec<f64>,
    pub local_audio_buffer_size: usize,
#[serde(skip_serializing, skip_deserializing)]
    pub local_audio_buffer: Vec<f32>,
    pub local_audio_buffer_offset: usize,
    pub remote_audio_buffer_size: usize,
#[serde(skip_serializing, skip_deserializing)]
    pub remote_audio_buffer: Vec<u8>,
    pub remote_audio_buffer_offset: usize,
    pub rxgain: i32,
    pub cw_pitch: f64,
    pub cw_decoder: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub cw_decoder_audio_buffer_offset: usize,
#[serde(skip_serializing, skip_deserializing)]
    pub cw_decoder_audio_buffer: Vec<f32>,

    pub equalizer_enabled: bool,
    pub equalizer_preamp: f32,
    pub equalizer_low: f32,
    pub equalizer_mid: f32,
    pub equalizer_high: f32,

    pub spectrum_average_time: f32,
    pub waterfall_average_time: f32,
    pub band_info: Vec<BandInfo>,

    pub remote_output: bool,
    pub local_output: bool,
    pub output_device: String,

#[serde(skip_serializing, skip_deserializing)]
    pub local_output_changed: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub local_output_changed_to: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub local_output_device_changed: bool,

    pub am_squelch: bool,
    pub am_squelch_threshold: f64,
    pub fm_squelch: bool,
    pub fm_squelch_threshold: f64,

}

impl Receiver {

    pub fn new(chan: u8, proto: u8, pixels: i32) -> Receiver {
        let protocol: u8 = proto;
        let channel: i32 = chan as i32;
        let active: bool = chan == 0;
        let adc: usize = 0;
        let buffer_size: usize = 1024;
        let fft_size: i32 = 2048;
        let sample_rate: i32 = DEFAULT_SAMPLE_RATE;
        let sample_rate_changed: bool = false;
        let dsp_rate: i32 = 48000;
        let output_rate: i32 = 48000;
        let output_samples: usize = buffer_size/(sample_rate/48000) as usize;
        let band: Bands = Bands::Band20;
        let filters_manual: bool = false;
        let filters: u32 = 0x01100002; // for Band20
        let frequency: f64 = 14175000.0;
        let step_index: usize = 7; // 1KHz
        let step: f64 = 1000.0; // 1KHz
        let ctun: bool = false;
        let ctun_frequency: f64 = 0.0;
        let nr: bool = false;
        let nr_taps: i32 = 64;
        let nr_delay: i32 = 16;
        let nr_gain: f32 = 100.0;
        let nr_leak: f32 = 100.0;
        let nr2: bool = false;
        let nb: bool = false;
        let nb2: bool = false;
        let anf: bool = false;
        let anf_taps: i32 = 64;
        let anf_delay: i32 = 16;
        let anf_gain: f32 = 100.0;
        let anf_leak: f32 = 100.0;
        let snb: bool = false;
        let agc_position: i32 = 0;
        let spectrum_fps = 80.0;
        let spectrum_width: i32 = pixels;
        let spectrum_step: f32 = 10.0;
        let waterfall_fps = 80.0;
        let waterfall_width: i32 = pixels;
        let zoom: i32 = 1;
        let pan: i32 = 0;
        let afgain: f32 = 0.1;
        let afpan: f32 = 0.5;
        let agc: AGC = AGC::FAST;
        let agcgain: f32 = 80.0;
        let agcslope: i32 = 35;
        let agcchangethreshold: i32 = 0;
        let filter_low: f64 = 300.0;
        let filter_high: f64 = 2700.0;
        let mode = Modes::USB.to_usize();
        let filter = Filters::F6.to_usize(); // 2.4k
        let iq_input_buffer = vec![0.0; buffer_size * 2];
        let samples: usize = 0;
        let audio_output: AudioOutput = AudioOutput::Stereo;
        let audio_buffer = vec![0.0; output_samples * 2];
        let local_audio_buffer_size: usize = 2048;
        let local_audio_buffer = vec![0.0f32; local_audio_buffer_size*2];
        let local_audio_buffer_offset: usize = 0;
        let remote_audio_buffer_size: usize = 260;
        let remote_audio_buffer = vec![0u8; remote_audio_buffer_size];
        let remote_audio_buffer_offset: usize = 4;
        let rxgain: i32 = 0;
        let cw_pitch: f64 = 900.0;
        let cw_decoder: bool =  false;
        let cw_decoder_audio_buffer_offset: usize =0;
        let cw_decoder_audio_buffer = vec![0.0f32; local_audio_buffer_size];
        let equalizer_enabled: bool = true;
        let equalizer_preamp: f32 = 0.0;
        let equalizer_low: f32 = 0.0;
        let equalizer_mid: f32 = 0.0;
        let equalizer_high: f32 = 0.0;

        let spectrum_average_time: f32 = DEFAULT_SPECTRUM_AVERAGE_TIME;
        let waterfall_average_time: f32 = DEFAULT_WATERFALL_AVERAGE_TIME;
        let band_info = BandInfo::new();

        let remote_output = true;
        let local_output = false;
        let output_device = String::from("default");
        let local_output_changed = false;
        let local_output_changed_to = false;
        let local_output_device_changed = false;
        let am_squelch: bool = false;
        let am_squelch_threshold: f64 = 0.0;
        let fm_squelch: bool = false;
        let fm_squelch_threshold: f64 = 0.0;

        

        Receiver { protocol,
                            channel,
                            active,
                            adc,
                            buffer_size,
                            fft_size,
                            sample_rate,
                            sample_rate_changed,
                            dsp_rate,
                            output_rate,
                            output_samples,
                            band,
                            filters_manual,
                            filters,
                            frequency,
                            step_index,
                            step,
                            ctun,
                            ctun_frequency,
                            nr,
                            nr_taps,
                            nr_delay,
                            nr_gain,
                            nr_leak,
                            nr2,
                            nb,
                            nb2,
                            anf,
                            anf_taps,
                            anf_delay,
                            anf_gain,
                            anf_leak,
                            snb,
                            agc_position,
                            spectrum_fps,
                            spectrum_width,
                            spectrum_step,
                            waterfall_fps,
                            waterfall_width,
                            zoom,
                            pan,
                            afgain,
                            afpan,
                            agc,
                            agcgain,
                            agcslope,
                            agcchangethreshold,
                            filter_low,
                            filter_high,
                            mode,
                            filter,
                            iq_input_buffer,
                            samples,
                            audio_output,
                            audio_buffer,
                            local_audio_buffer_size,
                            local_audio_buffer,
                            local_audio_buffer_offset,
                            remote_audio_buffer_size,
                            remote_audio_buffer,
                            remote_audio_buffer_offset,
                            rxgain,
                            cw_pitch,
                            cw_decoder,
                            cw_decoder_audio_buffer_offset,
                            cw_decoder_audio_buffer,
                            equalizer_enabled,
                            equalizer_preamp,
                            equalizer_low,
                            equalizer_mid,
                            equalizer_high,
                            spectrum_average_time,
                            waterfall_average_time,
                            band_info,
                            remote_output,
                            local_output,
                            output_device,
                            local_output_changed,
                            local_output_changed_to,
                            local_output_device_changed,
                            am_squelch,
                            am_squelch_threshold,
                            fm_squelch,
                            fm_squelch_threshold,
        }
    }

    pub fn init(&mut self) {
        self.iq_input_buffer = vec![0.0; self.buffer_size * 2];
        self.samples = 0;
        self.audio_buffer = vec![0.0; self.output_samples * 2];
        self.local_audio_buffer = vec![0.0f32; self.local_audio_buffer_size*2];
        self.local_audio_buffer_offset = 0;
        self.local_output_changed = false;
        self.local_output_changed_to = false;
        self.local_output_device_changed = false;
        self.remote_audio_buffer = vec![0u8; self.remote_audio_buffer_size];
        self.remote_audio_buffer_offset = 4;
        self.cw_decoder_audio_buffer_offset = 0;
        self.cw_decoder_audio_buffer = vec![0.0f32; self.local_audio_buffer_size];
        //self.morsedecoder = MorseDecoder::new((self.output_rate/4) as f32, 20.0);

        self.init_wdsp(self.channel);
        self.create_display(self.channel);
        self.init_analyzer(self.channel, self.spectrum_width);

        self.enable_equalizer();

        if self.nb {
            self.set_nb();
        }
        if self.nb2 {
            self.set_nb2();
        }
        if self.nr {
            self.set_nr();
        }
        if self.nr2 {
            self.set_nr2();
        }
        if self.snb {
            self.set_snb();
        }

    }

    fn init_wdsp(&mut self, channel: i32) {
        unsafe {
            OpenChannel(channel, self.buffer_size as i32, self.fft_size, self.sample_rate, self.dsp_rate, self.output_rate, 0, 1, 0.010, 0.025, 0.0, 0.010, 0);
            create_anbEXT(channel, 1, self.buffer_size as i32, self.sample_rate.into(), 0.0001, 0.0001, 0.0001, 0.05, 20.0);
            create_nobEXT(channel, 1, 0, self.buffer_size as i32, self.sample_rate.into(), 0.0001, 0.0001, 0.0001, 0.05, 20.0);
            RXASetNC(channel, self.fft_size);
            RXASetMP(channel, 0); // low_latency

            SetRXAPanelGain1(channel, self.afgain.into());
            SetRXAPanelPan(channel, self.afpan.into());
            AGC::set_agc(self, channel);
            SetRXAAGCTop(channel, self.agcgain.into());
            SetRXAPanelSelect(channel, 3);
            SetRXAPanelPan(channel, 0.5);
            SetRXAPanelCopy(channel, 0);
            SetRXAPanelBinaural(channel, 0);
            SetRXAPanelRun(channel, 1);

            //if(self.enable_equalizer) {
            //  SetRXAGrphEQ(channel, rx->equalizer);
            //  SetRXAEQRun(channel, 1);
            //} else {
              SetRXAEQRun(channel, 0);
            //}

            SetEXTANBSamplerate (channel, self.sample_rate);
            SetEXTNOBSamplerate (channel, self.sample_rate);
            SetEXTANBRun(channel, 0); //self.nb);
            SetEXTNOBRun(channel, self.nb.into()); //self.nb2);

            SetRXAEMNRPosition(channel, self.agc_position);
            SetRXAEMNRgainMethod(channel, 2); //self.nr2_gain_method);
            SetRXAEMNRnpeMethod(channel, 0); //self.nr2_npe_method);
            SetRXAEMNRRun(channel, self.nr.into()); //self.nr2);
            SetRXAEMNRaeRun(channel, 1); //self.nr2_ae);

            SetRXAANRPosition(channel, self.agc_position);
            SetRXAANRVals(channel, self.nr_taps, self.nr_delay, 1e-6 * self.nr_gain as f64, 1e-3 * self.nr_leak as f64);
            SetRXAANRRun(channel, 0); //self.nr);

            SetRXAANFPosition(channel, self.agc_position);
            SetRXAANFVals(channel, self.anf_taps, self.anf_delay, 1e-6 * self.anf_gain as f64, 1e-3 * self.anf_leak as f64);
            SetRXAANFRun(channel, self.anf.into()); //self.anf);
            SetRXASNBARun(channel, self.snb.into()); //self.snb);

            SetRXAMode(channel, self.mode as i32);
            RXASetPassband(channel,self.filter_low,self.filter_high);

            if self.ctun {
                let mut offset = self.ctun_frequency - self.frequency;
                SetRXAShiftRun(channel, 1);
                SetRXAShiftFreq(channel, offset.into());
                RXANBPSetShiftFrequency(channel, 0.0);
            }

            self.set_squelch_threshold();
        }
    }

    pub fn update_Nrvals(&self) {
        unsafe {
            SetRXAANRVals(self.channel, self.nr_taps, self.nr_delay, 1e-6 * self.nr_gain as f64, 1e-3 * self.nr_leak as f64);
        }
    }

    pub fn update_Anfvals(&self) {
        unsafe {
            SetRXAANFVals(self.channel, self.anf_taps, self.anf_delay, 1e-6 * self.anf_gain as f64, 1e-3 * self.anf_leak as f64);
        }
    }

    pub fn update_AgcPosition(&self) {
        unsafe {
            SetRXAEMNRPosition(self.channel, self.agc_position);
            SetRXAANRPosition(self.channel, self.agc_position);
            SetRXAANFPosition(self.channel, self.agc_position);
        }
    }

    fn create_display(&self, display: i32) {
        let empty_string = String::from("");
        let c_string = CString::new(empty_string).expect("CString::new failed");
        let c_char_ptr: *mut c_char = c_string.into_raw();
        unsafe {
            let mut result: c_int = 0;
            XCreateAnalyzer(display, &mut result, 262144, 1, 1, c_char_ptr);
        }
    }

    pub fn update_spectrum_average(&self, display: i32) {
        unsafe {
            let t = 0.001 * self.spectrum_average_time;
            let display_avb = (-1.0 / (self.spectrum_fps * t)).exp(); 
            let display_average = max(2, min(60, (self.spectrum_fps * t) as i32));
            SetDisplayAvBackmult(display, 0, display_avb.into());
            SetDisplayNumAverage(display, 0, display_average);
        }
    }

    pub fn update_waterfall_average(&self, display: i32) {
        unsafe {
            let t = 0.001 * self.waterfall_average_time;
            let display_avb = (-1.0 / (self.waterfall_fps * t)).exp(); 
            let display_average = max(2, min(60, (self.waterfall_fps * t) as i32));
            SetDisplayAvBackmult(display, 1, display_avb.into());
            SetDisplayNumAverage(display, 1, display_average);
        }
    }

    pub fn init_analyzer(&mut self, display: i32, width: i32) {
        self.spectrum_width = width;
        let mut flp = [0];
        let keep_time: f32 = 0.1;
        let fft_size = 8192; 
        let max_w = fft_size + min((keep_time * self.spectrum_fps) as i32, (keep_time * fft_size as f32  * self.spectrum_fps) as i32);
        let buffer_size: i32 = self.buffer_size as i32;
        let pixels = self.spectrum_width * self.zoom;
        unsafe {
            SetAnalyzer(display, 2, 1, 1, flp.as_mut_ptr(), fft_size, buffer_size, 4, 14.0, 2048, 0, 0, 0, pixels, 1, 0, 0.0, 0.0, max_w);
            SetDisplayDetectorMode(display, 0, DETECTOR_MODE_AVERAGE.try_into().expect("SetDisplayDetectorMode failed!"));
            SetDisplayAverageMode(display, 0,  AVERAGE_MODE_LOG_RECURSIVE.try_into().expect("SetDisplayAverageMode failed!"));
            SetDisplayDetectorMode(display, 1, DETECTOR_MODE_AVERAGE.try_into().expect("SetDisplayDetectorMode failed!"));
            SetDisplayAverageMode(display, 1,  AVERAGE_MODE_LOG_RECURSIVE.try_into().expect("SetDisplayAverageMode failed!"));
        }
        self.update_spectrum_average(display);
        self.update_waterfall_average(display);
    }

    pub fn set_filter(&self) {
        unsafe {
            RXASetPassband(self.channel, self.filter_low.into(), self.filter_high.into());
        }
    }

    pub fn set_mode(&mut self) {
        unsafe {
            SetRXAMode(self.channel, self.mode as i32);
            self.set_squelch_threshold();
        }
        self.set_filter();
    }

    pub fn set_frequency(&mut self, frequency: f64) {
        if self.ctun {
            self.ctun_frequency = frequency;
            let mut offset = self.ctun_frequency - self.frequency;
            unsafe {
                SetRXAShiftFreq(self.channel, offset.into());
                RXANBPSetShiftFrequency(self.channel, offset.into());
            }
        } else {
            self.frequency = frequency;
            unsafe {
                RXANBPSetTuneFrequency(self.channel, frequency.into());
            }
        }
    }

    pub fn set_ctun_frequency(&self) {
        let mut offset = self.ctun_frequency - self.frequency;
        unsafe {
            SetRXAShiftFreq(self.channel, offset.into());
            RXANBPSetShiftFrequency(self.channel, offset.into());
        }
    }

    pub fn set_ctun(&self, state: bool) {
        if state {
            unsafe {
                SetRXAShiftRun(self.channel, 1);
                self.set_ctun_frequency();
            }
        } else {
            unsafe {
                SetRXAShiftRun(self.channel, 0);
            }
        }
    }

    pub fn set_afgain(&self) {
        unsafe {
            SetRXAPanelGain1(self.channel, self.afgain.into());
        }
    }

    pub fn set_afpan(&self) {
        unsafe {
            SetRXAPanelPan(self.channel, self.afpan.into());
        }
    }

    pub fn set_agcgain(&self) {
        unsafe {
            SetRXAAGCTop(self.channel, self.agcgain.into());
        }
    }

    pub fn set_nr(&self) {
        unsafe {
            SetRXAANRRun(self.channel, self.nr as i32);
        }  
    }

    pub fn set_squelch_threshold(&mut self) {
        if self.mode == Modes::FMN.to_usize() {
            let sq = 10.0_f32.powf((-2.0 * self.fm_squelch_threshold / 100.0) as f32);
            unsafe {
                SetRXAFMSQThreshold(self.channel, sq as f64);
            }
            if self.fm_squelch_threshold > 0.0 {
                self.fm_squelch = true;
            } else {
                self.fm_squelch = false;
            }
            unsafe {
                SetRXAFMSQRun(self.channel, self.fm_squelch as i32);
            }
        } else {
            let sq = -140.0 + self.am_squelch_threshold;
            unsafe {
                SetRXAAMSQThreshold(self.channel, sq as f64);
            }
            if self.am_squelch_threshold > 0.0 {
                self.am_squelch = true;
            } else {
                self.am_squelch = false;
            }
            unsafe {
                SetRXAAMSQRun(self.channel, self.am_squelch as i32);
            }
        }
    }

    pub fn set_nr2(&self) {
        unsafe {
            SetRXAEMNRRun(self.channel, self.nr2 as i32);
        }  
    }

    pub fn set_nb(&self) {
        unsafe {
            SetEXTANBRun(self.channel, self.nb as i32);
        }
    }

    pub fn set_nb2(&self) {
        unsafe {
            SetEXTNOBRun(self.channel, self.nb as i32);
        }
    }

    pub fn set_anf(&self) {
        unsafe {
            SetRXAANFRun(self.channel, self.anf as i32);
        }
    }

    pub fn set_snb(&self) {
        unsafe {
            SetRXASNBARun(self.channel, self.snb as i32);
        }
    }

    pub fn enable_equalizer(&self) {
        if self.equalizer_enabled {
            self.set_equalizer_values();
        }
        unsafe {
            SetRXAEQRun(self.channel, self.equalizer_enabled.into());
        }
    }

    pub fn set_equalizer_values(&self) {

        let mut values: Vec<i32> = vec![
            self.equalizer_preamp as i32,
            self.equalizer_low as i32,
            self.equalizer_mid as i32,
            self.equalizer_high as i32,
        ];
        unsafe {
            SetRXAGrphEQ(self.channel, values.as_mut_ptr());
        }
    }

    pub fn process_iq_samples(&mut self) {
        let raw_ptr: *mut f64 = self.iq_input_buffer.as_mut_ptr() as *mut f64;
        let audio_ptr: *mut f64 = self.audio_buffer.as_mut_ptr() as *mut f64;
        if self.nb {
            unsafe {
                xanbEXT(self.channel, raw_ptr, raw_ptr);
            }
        }
        if self.nb2{
            unsafe {
                xnobEXT(self.channel, raw_ptr, raw_ptr);
            }
        }

        let mut result: c_int = 0;
        unsafe {
            fexchange0(self.channel, raw_ptr, audio_ptr, &mut result);
            Spectrum0(1, self.channel, 0, 0, raw_ptr);
        }
    }

    pub fn sample_rate_changed(&mut self, rate: i32) {
        self.sample_rate = rate;
        self.output_samples = self.buffer_size/(self.sample_rate/48000) as usize;
        self.audio_buffer = vec![0.0; self.output_samples * 2];
        unsafe {
            SetChannelState(self.channel, 0, 1);
        }
        self.init_analyzer(self.channel, self.spectrum_width);
        unsafe {
            SetInputSamplerate(self.channel, rate);
            SetEXTANBSamplerate(self.channel, rate);
            SetEXTNOBSamplerate(self.channel, rate);
            SetChannelState(self.channel, 1, 0);
        }
        self.sample_rate_changed = false;
    }

}
