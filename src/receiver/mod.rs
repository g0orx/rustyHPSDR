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
use crate::bands::Bands;
use crate::bands::BandInfo;
use crate::filters::Filters;
use crate::modes::Modes;
use crate::wdsp::*;

const DEFAULT_SAMPLE_RATE: i32 =384000;
const DISPLAY_AVERAGE_TIME: f32 = 170.0;
const SUBRX_BASE_CHANNEL: i32 = 16;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Receiver {
    pub channel: i32,
    pub buffer_size: usize,
    pub fft_size: i32,
    pub sample_rate: i32,
    pub dsp_rate: i32,
    pub output_rate: i32,
    pub output_samples: usize,
    pub band: Bands,
    pub filters_manual: bool,
    pub filters: u32,
    pub frequency_a: f32,
    pub frequency_b: f32,
    pub step_index: usize,
    pub step: f32,
    pub ctun:  bool,
    pub ctun_frequency: f32,
    pub nr: bool,
    pub nb: bool,
    pub anf: bool,
    pub snb: bool,
    pub fps: f32,
    pub spectrum_width: i32,
    pub spectrum_step: f32,
    pub zoom: i32,
    pub pan: i32,
    pub afgain:  f32,
    pub afpan:  f32,
    pub agc: AGC,
    pub agcgain:  f32,
    pub agcslope:  i32,
    pub agcchangethreshold:  i32,
    pub filter_low: f32,
    pub filter_high: f32,
    pub mode: usize,
    pub filter: usize,
    #[serde(skip_serializing, skip_deserializing)]
    pub iq_input_buffer: Vec<f64>,
    pub samples: usize,
    pub local_audio_buffer_size: usize,
    #[serde(skip_serializing, skip_deserializing)]
    //pub local_audio_buffer: Vec<f64>,
    pub local_audio_buffer: Vec<i16>,
    pub local_audio_buffer_offset: usize,
    pub remote_audio_buffer_size: usize,
    #[serde(skip_serializing, skip_deserializing)]
    pub remote_audio_buffer: Vec<u8>,
    pub remote_audio_buffer_offset: usize,
    pub attenuation: i32,
    pub rxgain: i32,
    pub cw_pitch: f32,

    pub subrx: bool,
    pub subrx_channel: i32,

    pub equalizer_enabled: bool,
    pub equalizer_preamp: f32,
    pub equalizer_low: f32,
    pub equalizer_mid: f32,
    pub equalizer_high: f32,
}

impl Receiver {

    pub fn new(chan: u8, band_info: &Vec<BandInfo>) -> Receiver {
        let channel: i32 = chan as i32;
        let buffer_size: usize = 1024;
        let fft_size: i32 = 2048;
        let sample_rate: i32 = DEFAULT_SAMPLE_RATE;
        let dsp_rate: i32 = 48000;
        let output_rate: i32 = 48000;
        let output_samples: usize = buffer_size/(sample_rate/48000) as usize;
        let band: Bands = Bands::Band20;
        let filters_manual: bool = false;
        let filters: u32 = 0x01100002; // for Band20
        let frequency_a: f32 = 14175000.0;
        let frequency_b: f32 = 14250000.0;
        let step_index: usize = 7; // 1KHz
        let step: f32 = 1000.0; // 1KHz
        let ctun: bool = false;
        let ctun_frequency: f32 = 0.0;
        let nr: bool = false;
        let nb: bool = false;
        let anf: bool = false;
        let snb: bool = false;
        let fps = 50.0;
        let spectrum_width: i32 = 1024;
        let spectrum_step: f32 = 10.0;
        let zoom: i32 = 1;
        let pan: i32 = 0;
        let afgain: f32 = 0.5;
        let afpan: f32 = 0.5;
        let agc: AGC = AGC::FAST;
        let agcgain: f32 = 80.0;
        let agcslope: i32 = 35;
        let agcchangethreshold: i32 = 0;
        let filter_low: f32 = 300.0;
        let filter_high: f32 = 2700.0;
        let mode = Modes::USB.to_usize();
        let filter = Filters::F6.to_usize(); // 2.4k
        let iq_input_buffer = vec![0.0; (2*buffer_size) as usize];
        let samples: usize = 0;
        let local_audio_buffer_size: usize = 2048;
        //let local_audio_buffer = vec![0.0; local_audio_buffer_size*2];
        let local_audio_buffer = vec![0i16; local_audio_buffer_size*2];
        let local_audio_buffer_offset: usize = 0;
        let remote_audio_buffer_size: usize = 260;
        let remote_audio_buffer = vec![0u8; remote_audio_buffer_size];
        let remote_audio_buffer_offset: usize = 4;
        let attenuation: i32 = 0;
        let rxgain: i32 = 0;
        let cw_pitch: f32 = 200.0;
        let subrx: bool = false;
        let subrx_channel: i32 = channel + SUBRX_BASE_CHANNEL;
        let equalizer_enabled: bool = true;
        let equalizer_preamp: f32 = 0.0;
        let equalizer_low: f32 = 0.0;
        let equalizer_mid: f32 = 0.0;
        let equalizer_high: f32 = 0.0;


        let rx = Receiver{ channel, buffer_size, fft_size, sample_rate, dsp_rate, output_rate, output_samples, band, filters_manual, filters, frequency_a, frequency_b, step_index, step, ctun, ctun_frequency, nr, nb, anf, snb, fps, spectrum_width, spectrum_step, zoom, pan, afgain, afpan, agc, agcgain, agcslope, agcchangethreshold, filter_low, filter_high, mode, filter, iq_input_buffer, samples, local_audio_buffer_size, local_audio_buffer, local_audio_buffer_offset, remote_audio_buffer_size, remote_audio_buffer, remote_audio_buffer_offset, attenuation, rxgain, cw_pitch, subrx, subrx_channel, equalizer_enabled, equalizer_preamp, equalizer_low, equalizer_mid, equalizer_high };

        rx
    }

    pub fn init(&mut self) {
        self.iq_input_buffer = vec![0.0; (2*self.buffer_size) as usize];
        self.samples = 0;
        //self.local_audio_buffer = vec![0.0; self.local_audio_buffer_size*2];
        self.local_audio_buffer = vec![0i16; self.local_audio_buffer_size*2];
        self.local_audio_buffer_offset = 0;
        self.remote_audio_buffer = vec![0u8; self.remote_audio_buffer_size];
        self.remote_audio_buffer_offset = 4;

        self.init_wdsp(self.channel);
        self.create_display(self.channel);
        self.init_analyzer(self.channel);
        self.init_wdsp(self.subrx_channel);

        self.enable_equalizer();
    }

    fn init_wdsp(&self, channel: i32) {
        unsafe {
            OpenChannel(channel, self.buffer_size as i32, self.fft_size, self.sample_rate, self.dsp_rate, self.output_rate, 0, 1, 0.010, 0.025, 0.0, 0.010, 0);
            create_anbEXT(channel, 1, self.buffer_size as i32, self.sample_rate.into(), 0.0001, 0.0001, 0.0001, 0.05, 20.0);
            create_nobEXT(channel,1,0,self.buffer_size as i32,self.sample_rate.into(),0.0001,0.0001,0.0001,0.05,20.0);
            RXASetNC(channel, self.fft_size);
            RXASetMP(channel, 0); // low_latency

            SetRXAPanelGain1(channel, self.afgain.into());
            SetRXAPanelPan(channel, self.afpan.into());
            AGC::set_agc(&self, channel);
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

            SetEXTANBRun(channel, 0); //self.nb);
            SetEXTNOBRun(channel, self.nb.into()); //self.nb2);

            SetRXAEMNRPosition(channel, 0); //self.nr_agc);
            SetRXAEMNRgainMethod(channel, 2); //self.nr2_gain_method);
            SetRXAEMNRnpeMethod(channel, 0); //self.nr2_npe_method);
            SetRXAEMNRRun(channel, self.nr.into()); //self.nr2);
            SetRXAEMNRaeRun(channel, 1); //self.nr2_ae);

            SetRXAANRVals(channel, 64, 16, 16e-4, 10e-7); // defaults
            SetRXAANRRun(channel, 0); //self.nr);
            SetRXAANFRun(channel, self.anf.into()); //self.anf);
            SetRXASNBARun(channel, self.snb.into()); //self.snb);

            SetRXAMode(channel, self.mode as i32);
            if self.mode == Modes::CWL.to_usize() || self.mode == Modes::CWU.to_usize() {
                RXASetPassband(channel,(self.cw_pitch - self.filter_low).into(), (self.cw_pitch +self.filter_high).into());
            } else {
                RXASetPassband(channel,self.filter_low.into(),self.filter_high.into());
            }

            if self.ctun {
                let mut offset = self.ctun_frequency - self.frequency_a;
                if self.mode == Modes::CWL.to_usize() {
                     offset = offset + self.cw_pitch;
                } else if self.mode == Modes::CWU.to_usize() {
                     offset = offset - self.cw_pitch;
                }
                SetRXAShiftRun(channel, 1);
                SetRXAShiftFreq(channel, offset.into());
                RXANBPSetShiftFrequency(channel, 0.0);
            }
        }
    }

    fn create_display(&self, display: i32) {
        let empty_string = String::from("");
        let c_string = CString::new(empty_string).expect("CString::new failed");
        let c_char_ptr: *mut c_char = c_string.into_raw();
        unsafe {
            let mut result: c_int = 0;
            XCreateAnalyzer(display, &mut result, 262144, 1, 1, c_char_ptr);
            SetDisplayDetectorMode(display, 0, DETECTOR_MODE_AVERAGE.try_into().expect("SetDisplayDetectorMode failed!"));
            SetDisplayAverageMode(display, 0,  AVERAGE_MODE_LOG_RECURSIVE.try_into().expect("SetDisplayAverageMode failed!"));
            let t = 0.001 * DISPLAY_AVERAGE_TIME;
            let display_avb = (-1.0 / (self.fps * t)).exp();
            let display_average = max(2, min(60, (self.fps * t) as i32));
            SetDisplayAvBackmult(display, 0, display_avb.into());
            SetDisplayNumAverage(display, 0, display_average);

        }
    }

    pub fn init_analyzer(&self, display: i32) {
        let mut flp = [0];
        let keep_time: f32 = 0.1;
        let fft_size = 8192; 
        let max_w = fft_size + min((keep_time * self.fps) as i32, (keep_time * fft_size as f32  * self.fps) as i32);
        let buffer_size: i32 = self.buffer_size as i32;
        let pixels = self.spectrum_width * self.zoom;
        unsafe {
            SetAnalyzer(display, 1, 1, 1, flp.as_mut_ptr(), fft_size, buffer_size, 4, 14.0, 2048, 0, 0, 0, pixels, 1, 0, 0.0, 0.0, max_w);
        }
    }

    pub fn set_filter(&self) {
        unsafe {
            RXASetPassband(self.channel, self.filter_low.into(), self.filter_high.into());
            RXASetPassband(self.subrx_channel, self.filter_low.into(), self.filter_high.into());
        }
    }

    pub fn set_mode(&self) {
        unsafe {
            SetRXAMode(self.channel, self.mode as i32);
            SetRXAMode(self.subrx_channel, self.mode as i32);
        }
        self.set_filter();
    }

    pub fn set_ctun_frequency(&self) {
        let mut offset = self.ctun_frequency - self.frequency_a;
        if self.mode == Modes::CWL.to_usize() {
             offset = offset + self.cw_pitch;
        } else if self.mode == Modes::CWU.to_usize() {
             offset = offset - self.cw_pitch;
        }
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
            SetRXAPanelGain1(self.subrx_channel, self.afgain.into());
        }
    }

    pub fn set_afpan(&self) {
        unsafe {
            SetRXAPanelPan(self.channel, self.afpan.into());
            SetRXAPanelPan(self.subrx_channel, self.afpan.into());
        }
    }

    pub fn set_agcgain(&self) {
        unsafe {
            SetRXAAGCTop(self.channel, self.agcgain.into());
            SetRXAAGCTop(self.subrx_channel, self.agcgain.into());
        }
    }

    pub fn set_nr(&self) {
        unsafe {
            SetRXAEMNRRun(self.channel, self.nr as i32);
            SetRXAEMNRRun(self.subrx_channel, self.nr as i32);
        }  
    }

    pub fn set_nb(&self) {
        unsafe {
            SetEXTNOBRun(self.channel, self.nb as i32);
            SetEXTNOBRun(self.subrx_channel, self.nb as i32);
        }
    }

    pub fn set_anf(&self) {
        unsafe {
            SetRXAANFRun(self.channel, self.anf as i32);
            SetRXAANFRun(self.subrx_channel, self.anf as i32);
        }
    }

    pub fn set_snb(&self) {
        unsafe {
            SetRXASNBARun(self.channel, self.snb as i32);
            SetRXASNBARun(self.subrx_channel, self.snb as i32);
        }
    }

    pub fn set_subrx_frequency(&self) {
        let mut offset = self.frequency_b - self.frequency_a;
        if self.mode == Modes::CWL.to_usize() {
             offset = offset + self.cw_pitch;
        } else if self.mode == Modes::CWU.to_usize() {
             offset = offset - self.cw_pitch;
        }
        unsafe {
            SetRXAShiftFreq(self.subrx_channel, offset.into());
            RXANBPSetShiftFrequency(self.subrx_channel, offset.into());
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

}
