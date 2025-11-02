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


use std::cmp::{max, min};
use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use serde::{Deserialize, Serialize};

use crate::alex::*;
use crate::discovery::Boards;
use crate::modes::Modes;
use crate::wdsp::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transmitter {
    pub protocol: u8,
    pub board: Boards,
    pub channel: i32,
    pub sample_rate: i32,
    pub dsp_rate: i32,
    pub output_rate: i32,
    pub output_samples: i32,
    pub packet_counter: i32,
    pub is_transmitting: bool,
    pub microphone_buffer_size: usize,
#[serde(skip_serializing, skip_deserializing)]
    pub microphone_buffer: Vec<f64>,
#[serde(skip_serializing, skip_deserializing)]
    pub microphone_samples: usize,
#[serde(skip_serializing, skip_deserializing)]
    pub iq_buffer: Vec<f64>,
#[serde(skip_serializing, skip_deserializing)]
    pub iq_samples: usize,
    pub fft_size: i32,
    pub low_latency: bool,
    pub use_rx_filter: bool,
    pub mode: usize,
    pub filter_low: f32,
    pub filter_high: f32,
    pub drive: f32,
    pub spectrum_width: i32,
    pub fps: f32,
    pub display_average_time: f32,
    pub spectrum_high: f32,
    pub spectrum_low: f32,
    pub micgain: f32,
    pub tx_antenna: u32,
#[serde(skip_serializing, skip_deserializing)]
    pub exciter_power: u16,
#[serde(skip_serializing, skip_deserializing)]
    pub alex_forward_power: u16,
#[serde(skip_serializing, skip_deserializing)]
    pub alex_reverse_power: u16,
    pub pa_calibration: Vec<f32>,
    pub c1: f32,
    pub c2: f32,
    pub remote_input: bool,
    pub local_input: bool,
    pub local_input_changed: bool,
    pub input_device: String,
    pub input_device_changed: bool,
    pub alc: f64,
#[serde(skip_serializing, skip_deserializing)]
    pub input_level: f64,
}

impl Transmitter {

    pub fn new(chan: u8, proto: u8, board: Boards ) -> Transmitter {
        let protocol: u8 = proto;
        let board: Boards = board;
        let channel: i32 = chan as i32;
        let sample_rate = 48000; // protocol 1 & 2 input audio
        let mut dsp_rate = 48000;    // protocol 1
        let mut output_rate = 48000; // protocol 1
        if protocol == 2 {
            dsp_rate = 96000;
            output_rate = 192000;
        }
        let mut output_samples=1024;
        if protocol == 2 {
            output_samples = 1024*(output_rate/sample_rate);
        }
        let packet_counter = 0;
        let is_transmitting = false;

        let microphone_buffer_size = 1024_usize;
        let microphone_buffer = vec![0.0f64; microphone_buffer_size * 2];
        let microphone_samples = 0;


        let fft_size = 2048;

        let iq_buffer = vec![0.0f64; (output_samples * 2) as usize];
        let iq_samples = 0_usize;

        let low_latency = false;
        let use_rx_filter = false;

        let mode = Modes::USB.to_usize();
        let filter_low = 300.0;
        let filter_high = 2700.0;
        
        let drive  = 25.0;

        let spectrum_width = 1024;
        let fps = 10.0;
        let display_average_time = 60.0;

        let spectrum_high = 6.0;
        let spectrum_low = -54.0;

        let micgain = 0.0;

        let tx_antenna = ALEX_ANTENNA_1;
        let exciter_power:u16 = 0;
        let alex_forward_power:u16 = 0;
        let alex_reverse_power:u16 = 0;
        let pa_calibration: Vec<f32> =  vec![38.8, 38.8, 38.8, 38.8, 38.8, 38.8, 38.8, 38.8, 38.8, 38.8, 38.8, 38.8, 38.8, 38.8, 38.8]; // 15 bands

        let (c1, c2) = match board {
            Boards::Metis => (3.3, 0.09),
            Boards::Hermes => (3.3, 0.095),
            Boards::Hermes2 => (3.3, 0.095),
            Boards::Angelia => (3.3, 0.095),
            Boards::Orion => (5.0, 0.108),
            Boards::Orion2 => (5.0, 0.08),
            Boards::Saturn => (3.3, 0.09),
            Boards::HermesLite => (3.3, 1.4),
            Boards::HermesLite2 => (3.3, 1.4),
            Boards::Unknown => (3.3, 0.09),
        };
        let remote_input = true;
        let local_input = false;
        let local_input_changed = false;
        let input_device = String::from("default");
        let input_device_changed = false;
        let alc = 0.0;
        let input_level = 0.0;


        

        Transmitter{ protocol,
            board,
            channel,
            sample_rate,
            dsp_rate,
            output_rate,
            output_samples,
            packet_counter,
            is_transmitting,
            microphone_buffer_size,
            microphone_buffer,
            microphone_samples,
            iq_buffer,
            iq_samples,
            fft_size,
            low_latency,
            use_rx_filter,
            mode,
            filter_low,
            filter_high,
            drive,
            spectrum_width,
            fps,
            display_average_time,
            spectrum_high,
            spectrum_low,
            micgain,
            tx_antenna,
            exciter_power,
            alex_forward_power,
            alex_reverse_power,
            pa_calibration,
            c1,
            c2,
            remote_input,
            local_input,
            local_input_changed,
            input_device,
            input_device_changed,
            alc,
            input_level,
        }
    }

    pub fn init(&mut self) {
        eprintln!("Transmitter::init input_samples {} output_samples {}", self.microphone_buffer_size, self.output_samples);
        self.microphone_buffer = vec![0.0f64; self.microphone_buffer_size * 2];
        self.microphone_samples = 0;
        self.iq_buffer = vec![0.0f64; (self.output_samples * 2) as usize];
        self.iq_samples = 0;
        self.input_level = 0.0;

        self.init_wdsp();

        let id_string = String::from("TX");
        let c_string = CString::new(id_string).expect("CString::new failed");
        let c_char_ptr: *mut c_char = c_string.into_raw();


        unsafe {
            let mut result: c_int = 0;
            XCreateAnalyzer(self.channel, &mut result, 262144, 1, 1, c_char_ptr);
        }

        self.init_analyzer(self.spectrum_width);
    }

    pub fn init_analyzer(&mut self, width: i32) {
        self.spectrum_width = width;
        let mut flp = [0];
        let keep_time: f32 = 0.1;
        let max_w = self.fft_size + min((keep_time * self.fps) as i32, (keep_time * self.fft_size as f32  * self.fps) as i32);
        let buffer_size: i32 = self.output_samples * 4;
        let mut multiplier = 3; // protocol1
        if self.protocol == 2 {
            multiplier = 12; // protocol2
        }
        let pixels = self.spectrum_width * multiplier;
        eprintln!("init_analyzer channel {} width {} buffer_size {} multiplier {} pixels {} spectrum_width {}", self.channel, width, buffer_size, multiplier, pixels, self.spectrum_width);
        unsafe {
            SetAnalyzer(self.channel,
                1,
                1,
                1,
                flp.as_mut_ptr(),
                self.fft_size,
                self.output_samples,
                4,
                14.0,
                4096,
                0,
                0,
                0,
                pixels,
                1,
                0,
                0.0,
                0.0,
                max_w);
            SetDisplayDetectorMode(self.channel, 0, DETECTOR_MODE_AVERAGE.try_into().expect("SetDisplayDetectorMode failed!"));
            SetDisplayAverageMode(self.channel, 0,  AVERAGE_MODE_LOG_RECURSIVE.try_into().expect("SetDisplayAverageMode failed!"));
            let t = 0.001 * self.display_average_time;
            let display_avb = (-1.0 / (self.fps * t)).exp();
            let display_average = max(2, min(60, (self.fps * t) as i32));
            SetDisplayAvBackmult(self.channel, 0, display_avb.into());
            SetDisplayNumAverage(self.channel, 0, display_average);
        }
   } 

    

    fn init_wdsp(&mut self) {
        eprintln!("Transmitter::init_wdsp channel {} microphone_buffer_size {} fft_size {} sample_rate {} dsp_rate {} output_rate {} output_samples {}",
            self.channel, self.microphone_buffer_size, self.fft_size, self.sample_rate, self.dsp_rate, self.output_rate, self.output_samples);
        unsafe {
            OpenChannel(self.channel,
                self.microphone_buffer_size as i32,
                self.fft_size,
                self.sample_rate,
                self.dsp_rate,
                self.output_rate,
                1,
                0,
                0.010,
                0.025,
                0.0,
                0.010,
                0);
            TXASetNC(self.channel, self.fft_size);
            TXASetMP(self.channel, self.low_latency as i32);
            SetTXABandpassWindow(self.channel, 1);
            SetTXABandpassRun(self.channel, 1);
            SetTXAFMEmphPosition(self.channel,false as i32);
            if self.protocol == 1 {
                SetTXACFIRRun(self.channel, 0); // not needed for Protocol 1 as implemented in FPGA
            } else {
                SetTXACFIRRun(self.channel, 1); // only Protocol 2
            }
            SetTXAEQRun(self.channel, 0);
            SetTXAAMSQRun(self.channel, 0);
            SetTXAosctrlRun(self.channel, 0);
            SetTXAALCAttack(self.channel, 1);
            SetTXAALCDecay(self.channel, 10);
            SetTXAALCSt(self.channel, 1); // turn it on (always on)

            SetTXALevelerAttack(self.channel, 1);
            SetTXALevelerDecay(self.channel, 500);
            SetTXALevelerTop(self.channel, 5.0);
            SetTXALevelerSt(self.channel, false as i32);

            SetTXAPreGenMode(self.channel, 0);
            SetTXAPreGenToneMag(self.channel, 0.0);
            SetTXAPreGenToneFreq(self.channel, 0.0);
            SetTXAPreGenRun(self.channel, 0);
  
            SetTXAPostGenMode(self.channel, 0); // Tone
            SetTXAPostGenToneMag(self.channel, 0.2);
            SetTXAPostGenTTMag(self.channel, 0.2, 0.2);
            SetTXAPostGenToneFreq(self.channel, 0.0);
            SetTXAPostGenRun(self.channel, 0);

            SetTXAPanelGain1(self.channel,10.0_f32.powf(self.micgain / 20.0) as f64);
            SetTXAPanelRun(self.channel, 1);

            SetTXAFMDeviation(self.channel, 2500.0);
            SetTXAAMCarrierLevel(self.channel, 0.5);

            SetTXACompressorGain(self.channel, 0.0);
            SetTXACompressorRun(self.channel, false as i32);
        }

        self.set_mode();
        self.set_filter();

    }

    pub fn set_mode(&self) {
        eprintln!("Trasnmitter::set_mode {:?}", self.mode);
        unsafe {
            SetTXAMode(self.channel, self.mode as i32);
        }
    }

    pub fn set_filter(&self) {
        eprintln!("Trasnmitter::set_filter {} {}", self.filter_low, self.filter_high);
        unsafe {
            SetTXABandpassFreqs(self.channel, self.filter_low.into(), self.filter_high.into());
        }
    }


    pub fn set_tuning(&self, state: bool, cw_keyer_sidetone_frequency: i32) {
        unsafe {
            if state {
                let mut frequency = (self.filter_low + ((self.filter_high - self.filter_low) / 2.0)) as f64;
                if self.mode == Modes::CWL.to_usize() {
                    frequency = -cw_keyer_sidetone_frequency as f64;
                } else if self.mode == Modes::CWU.to_usize() {
                    frequency = cw_keyer_sidetone_frequency as f64;
                } else if self.mode == Modes::LSB.to_usize() {
                    frequency = (-self.filter_low - ((self.filter_high - self.filter_low) / 2.0)) as f64;
                } else if self.mode == Modes::USB.to_usize() {
                    frequency = (self.filter_low + ((self.filter_high - self.filter_low) / 2.0)) as f64;
                }
                SetTXAPostGenToneFreq(self.channel, frequency);
                SetTXAPostGenToneMag(self.channel, 0.99999);
                SetTXAPostGenMode(self.channel, 0); // Tone
                SetTXAPostGenRun(self.channel, 1);
            } else {
                SetTXAPostGenRun(self.channel, 0);
            }
        }
    }

    pub fn set_micgain(&self) {
        unsafe {
            SetTXAPanelGain1(self.channel,10.0_f32.powf(self.micgain / 20.0) as f64);
        }
    }

    pub fn run(&mut self) {
    }

    pub fn process_mic_samples(&mut self) {
        /*
        let mut input_level = 0.0;
        for i in 0..(self.microphone_buffer.len()/2) {
            let ix = i * 2;
            if self.microphone_buffer[ix] < 0.0 {
                if -self.microphone_buffer[ix] > input_level {
                    input_level = -self.microphone_buffer[ix];
                } else if self.microphone_buffer[ix] > input_level {
                    input_level = self.microphone_buffer[ix];
                }
            }
        }
        self.input_level = input_level;
        */
        self.input_level = self.microphone_buffer.iter().fold(f64::NEG_INFINITY, |acc, &val| {
            acc.max(val)
        });

        let raw_ptr: *mut f64 = self.microphone_buffer.as_mut_ptr() as *mut f64;
        let iq_ptr: *mut f64 =  self.iq_buffer.as_mut_ptr() as *mut f64;
        let mut result: c_int = 0;
        unsafe {
            fexchange0(self.channel, raw_ptr, iq_ptr, &mut result);
            if result == 0 {
                Spectrum0(1, self.channel, 0, 0, iq_ptr);
            }
        }
    }

}
