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

use gtk::prelude::*;
use gtk::cairo::Context; 
use glib::source::SourceId;
use gdk_pixbuf::Pixbuf;

use std::cell::RefCell;
//use std::{env, fs, path::{PathBuf}};
use std::env;
use std::fmt::Write as FormatWrite;
use std::fs::*;
use std::io::{Read,Write};
use std::os::raw::c_int;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};


use crate::widgets::AppWidgets;
use crate::cat::CAT;
use crate::discovery::{Boards,Device};
use crate::modes::Modes;
use crate::receiver::Receiver;
use crate::transmitter::Transmitter;
use crate::wdsp::*;
use crate::audio::*;
use crate::alex::*;
use crate::adc::*;
use crate::notches::*;

#[derive(PartialEq, Serialize, Deserialize, Copy, Clone, Debug)]
pub enum RadioModels {
    Anan10,
    Anan10e,
    Anan100,
    Anan100b,
    Anan100d,
    Anan200d,
    Anan7000dle,
    Anan8000dle,
    AnanG1,
    AnanG2,
    HermesLite,
    HermesLite2,
    Undefined,
}

impl RadioModels {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => RadioModels::Anan10,
            1 => RadioModels::Anan10e,
            2 => RadioModels::Anan100,
            3 => RadioModels::Anan100b,
            4 => RadioModels::Anan100d,
            5 => RadioModels::Anan200d,
            6 => RadioModels::Anan7000dle,
            7 => RadioModels::Anan8000dle,
            8 => RadioModels::AnanG1,
            9 => RadioModels::AnanG2,
            10 => RadioModels::HermesLite,
            11 => RadioModels::HermesLite2,
            12 => RadioModels::Undefined,
            _ => RadioModels::Undefined,
        }
    }

    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}


#[derive(PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum FilterBoards {
    NONE,
    ALEX,
    APOLLO,
    N2ADR,
}

impl FilterBoards {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(FilterBoards::NONE),
            1 => Some(FilterBoards::ALEX),
            2 => Some(FilterBoards::APOLLO),
            3 => Some(FilterBoards::N2ADR),
            _ => None,
        }
    }

    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}


#[derive(PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum Keyer {
    Straight,
    ModeA,
    ModeB,
}

impl Keyer {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Keyer::Straight),
            1 => Some(Keyer::ModeA),
            2 => Some(Keyer::ModeB),
            _ => None,
        }
    }

    pub fn to_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Serialize, Deserialize)]
pub struct Radio {
    pub name: String,
    pub dev: u8,
    pub model: RadioModels,
    pub protocol: u8,
    pub supported_receivers: u8,
    pub sample_rate: i32,
    pub sample_rate_changed: bool,
    pub active_receiver: usize,
    pub receivers: u8,
    pub rx2_enabled: bool,
    pub split: bool,
    pub receiver: Vec<Receiver>,
#[serde(skip_serializing, skip_deserializing)]
    pub s_meter_dbm: f64,
#[serde(skip_serializing, skip_deserializing)]
    pub ptt: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub mox: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub vox: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub tune: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub dot: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub dash: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub external_mox: bool,
    pub audio: Vec<Audio>,
    pub transmitter: Transmitter,

    pub filter_board: FilterBoards,
    pub cw_keyer_mode: Keyer,
    pub cw_keyer_internal: bool,
    pub cw_keys_reversed: bool,
    pub cw_keyer_speed: i32,
    pub cw_keyer_weight: i32,
    pub cw_keyer_spacing: i32,
    pub cw_keyer_ptt_delay: i32,
    pub cw_keyer_hang_time: i32,
    pub cw_breakin: bool,
    pub cw_keyer_sidetone_volume: i32,
    pub cw_keyer_sidetone_frequency: i32,

    //pub local_microphone: bool,
    //pub local_microphone_name: String,

    pub adc: Vec<Adc>,

    pub alex: u32,
    pub mk2bpf: bool,

#[serde(skip_serializing, skip_deserializing)]
    pub updated: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub keepalive: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub received: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub pll_locked: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub adc_overload: bool,
#[serde(skip_serializing, skip_deserializing)]
    pub supply_volts: i32,

#[serde(skip_serializing, skip_deserializing)]
    pub swr: f32,

    pub line_in: bool,
    pub mic_boost: bool,
    pub mic_ptt: bool,
    pub mic_bias_ring: bool,
    pub mic_bias_enable: bool,
    pub mic_saturn_xlr: bool,
   
    pub waterfall_auto: bool,
    pub waterfall_calibrate: f32,

#[serde(skip_serializing, skip_deserializing)]
    pub spectrum_timeout_id: Option<SourceId>,
#[serde(skip_serializing, skip_deserializing)]
    pub waterfall_timeout_id: Option<SourceId>,
#[serde(skip_serializing, skip_deserializing)]
    pub meter_1_timeout_id: Option<SourceId>,
#[serde(skip_serializing, skip_deserializing)]
    pub meter_2_timeout_id: Option<SourceId>,

    pub notch: i32,
    pub notches: Vec<Notch>,

    pub cat_enabled: bool,

}

#[derive(Clone)]
pub struct RadioMutex {
    pub radio: Arc<Mutex<Radio>>,
}

impl RadioMutex {
    pub fn new(radio: Arc<Mutex<Radio>>) -> Self {
        RadioMutex {
            radio,
        }
    }
 
    pub fn update_spectrum(&self, width: i32) -> (c_int, Vec<f32>) {
        let mut r = self.radio.lock().unwrap();
        let (flag, pixels) = r.update_spectrum(width);
        (flag, pixels)
    }

    pub fn update_spectrum_2(&self, width: i32) -> (c_int, Vec<f32>) {
        let mut r = self.radio.lock().unwrap();
        let (flag, pixels) = r.update_spectrum_2(width);
        (flag, pixels)
    }

    pub fn update_waterfall(&self, width: i32) -> (c_int, Vec<f32>) {
        let mut r = self.radio.lock().unwrap();
        let (flag, pixels) = r.update_waterfall(width);
        (flag, pixels)
    }

    pub fn update_waterfall_2(&self, width: i32) -> (c_int, Vec<f32>) {
        let mut r = self.radio.lock().unwrap();
        let (flag, pixels) = r.update_waterfall_2(width);
        (flag, pixels)
    }

}

impl Radio {

    pub fn new(device: Device, spectrum_width: i32) -> Radio {
        let name = "HPSDR".to_string();
        let dev = device.device;
        // take a guess on the model based on the device
        let model = match device.board {
            Boards::Hermes => RadioModels::Anan100,
            Boards::Angelia => RadioModels::Anan100d,
            Boards::Orion => RadioModels::Anan200d,
            Boards::Orion2 => RadioModels::Anan8000dle,
            Boards::Saturn => RadioModels::AnanG1,
            Boards::HermesLite => RadioModels::HermesLite,
            Boards::HermesLite2 => RadioModels::HermesLite2,
            _ => RadioModels::Undefined,
        };
        let protocol = device.protocol;
        let supported_receivers = device.supported_receivers;
        let sample_rate = 384000;
        let sample_rate_changed = false;
        let active_receiver = 0;
        let receivers: u8 = 2;
        let rx2_enabled: bool = true;
        let split: bool = false;
        let mut receiver: Vec<Receiver> = Vec::new();
        for i in 0..receivers {
            receiver.push(Receiver::new(i, device.protocol, spectrum_width));
        }
        let s_meter_dbm = -121.0;
        let ptt = false;
        let mox = false;
        let vox = false;
        let tune = false;
        let dot = false;
        let dash = false;
        let external_mox = false;
        let mut audio: Vec<Audio> = Vec::new();
        for _i in 0..receivers {
            audio.push(Audio::new());
        }
        let transmitter = Transmitter::new(8, device.protocol, device.board);
        let filter_board = match device.board {
            Boards::HermesLite => FilterBoards::N2ADR,
            Boards::HermesLite2 => FilterBoards::N2ADR,
            _ => FilterBoards::ALEX,
        };

        let cw_keyer_mode = Keyer::Straight;
        let cw_keyer_internal = true;
        let cw_keys_reversed = false;
        let cw_keyer_speed = 12;
        let cw_keyer_weight = 30;
        let cw_keyer_spacing = 0;
        let cw_keyer_ptt_delay = 20;
        let cw_keyer_hang_time = 300;
        let cw_breakin = false;
        let cw_keyer_sidetone_volume = 20;
        let cw_keyer_sidetone_frequency = 650;
        //let local_microphone = false;
        //let local_microphone_name = "".to_string();

        let mut adc: Vec<Adc> = Vec::new();
        for _i in 0..device.adcs {
            adc.push(Adc::new());
        }
        let alex = ALEX_ANTENNA_1;
        let mk2bpf = match device.board {
            Boards::Orion2 => true,
            Boards::Saturn => true,
            _ => false,
        };

        let updated = false;
        let keepalive = false;
        let received = false;

        let pll_locked = false;
        let adc_overload = false;
        let supply_volts = 0;
        let swr = 1.0;

        let line_in = false;
        let mic_boost = true;
        let mic_ptt = true;
        let mic_bias_ring = false;
        let mic_bias_enable = true;
        let mic_saturn_xlr = false;

        let waterfall_auto = true;
        let waterfall_calibrate = 0.0;

        let spectrum_timeout_id = None;
        let waterfall_timeout_id = None;
        let meter_1_timeout_id = None;
        let meter_2_timeout_id = None;

        let notch = 0;
        let notches = vec![];

        let cat_enabled = false;

        Radio {
            name,
            dev,
            model,
            protocol,
            supported_receivers,
            sample_rate,
            sample_rate_changed,
            active_receiver,
            receivers,
            rx2_enabled,
            split,
            receiver,
            s_meter_dbm,
            ptt,
            mox,
            vox,
            tune,
            dot,
            dash,
            external_mox,
            audio,
            transmitter,
            filter_board,
            cw_keyer_mode,
            cw_keyer_internal,
            cw_keys_reversed,
            cw_keyer_speed,
            cw_keyer_weight,
            cw_keyer_spacing,
            cw_keyer_ptt_delay,
            cw_keyer_hang_time,
            cw_breakin,
            cw_keyer_sidetone_volume,
            cw_keyer_sidetone_frequency,

            //local_microphone,
            //local_microphone_name,

            adc,
            alex,
            mk2bpf,

            updated,
            keepalive,
            received,

            pll_locked,
            adc_overload,
            supply_volts,

            swr,
            line_in,
            mic_boost,
            mic_ptt,
            mic_bias_ring,
            mic_bias_enable,
            mic_saturn_xlr,
            waterfall_auto,
            waterfall_calibrate,

            spectrum_timeout_id,
            waterfall_timeout_id,
            meter_1_timeout_id,
            meter_2_timeout_id,
            notch,
            notches,

            cat_enabled,
        }
    }

    pub fn init(&mut self) {
        self.s_meter_dbm = -121.0;
        self.ptt = false;
        self.mox = false;
        self.vox = false;
        self.tune = false;
        self.dot = false;
        self.dash = false;
        self.external_mox = false;
        self.updated = false;

        self.pll_locked = false;
        self.adc_overload = false;
        self.supply_volts = 0;

        self.swr = 1.0;
    }

    pub fn is_transmitting(&self) -> bool {
        let cw = (self.dot | self.dash) && (self.receiver[0].mode == Modes::CWL.to_usize() || self.receiver[0].mode == Modes::CWU.to_usize());

        self.mox | self.ptt | cw | self.vox | self.tune | self.external_mox
    }

    pub fn run(&self) {
    }

    pub fn update_spectrum(&mut self, width: i32) -> (c_int, Vec<f32>) {
        let mut zoom = self.receiver[0].zoom;
        let mut channel = self.receiver[0].channel;
        if self.is_transmitting() {
            zoom = 1;
            channel = self.transmitter.channel;
        }
        let mut pixels_len = width * zoom;
        if self.is_transmitting() {
            if self.protocol == 1 {
                pixels_len = width * 3;
            } else {
                pixels_len = width * 12;
            }
        } 
        let mut pixels = vec![0.0; pixels_len as usize];
        let mut flag: c_int = 0;

        if !pixels.is_empty() { // may happen at start of application before spectrum is setup
            unsafe {
                GetPixels(channel, 0, pixels.as_mut_ptr(), &mut flag);
            }
        }
        (flag, pixels)
    }
    
    pub fn update_spectrum_2(&mut self, width: i32) -> (c_int, Vec<f32>) {
        let zoom = self.receiver[1].zoom;
        let channel = self.receiver[1].channel;
        let pixels_len = width * zoom;
        let mut pixels = vec![0.0; pixels_len as usize];
        let mut flag: c_int = 0;
        if !pixels.is_empty() { // may happen at start of application before spectrum is setup
            unsafe {
                GetPixels(channel, 0, pixels.as_mut_ptr(), &mut flag);
            }
        }
        (flag, pixels)
    }
    
    pub fn update_waterfall(&mut self, width: i32) -> (c_int, Vec<f32>) {
        let mut zoom = self.receiver[self.active_receiver].zoom;
        let mut channel = self.receiver[self.active_receiver].channel;
        if self.is_transmitting() {
            zoom = 1;
            channel = self.transmitter.channel;
        }

        let mut pixels_len = width * zoom;
        if self.is_transmitting() {
            pixels_len = width * 12;
        }

        let mut pixels = vec![0.0; pixels_len as usize];
        let mut flag: c_int = 0;
        if !pixels.is_empty() { // may happen at start of application before spectrum is setup
            unsafe {
                GetPixels(channel, 1, pixels.as_mut_ptr(), &mut flag);
            }
        }
        (flag, pixels)
    }

    pub fn update_waterfall_2(&mut self, width: i32) -> (c_int, Vec<f32>) {
        let zoom = self.receiver[1].zoom;
        let channel = self.receiver[1].channel;

        let pixels_len = width * zoom;

        let mut pixels = vec![0.0; pixels_len as usize];
        let mut flag: c_int = 0;
        if !pixels.is_empty() { // may happen at start of application before spectrum is setup
            unsafe {
                GetPixels(channel, 1, pixels.as_mut_ptr(), &mut flag);
            }
        }
        (flag, pixels)
    }

    pub fn set_state(&self) {
eprintln!("set_state: {}", self.is_transmitting());
        if self.is_transmitting() {
            unsafe {
                if self.rx2_enabled {
                    SetChannelState(self.receiver[0].channel, 0, 0);
                    SetChannelState(self.receiver[1].channel, 0, 1);
                } else {
                    SetChannelState(self.receiver[0].channel, 0, 1);
                }
                SetChannelState(self.transmitter.channel, 1, 0);
            }
        } else {
            unsafe {
                SetChannelState(self.transmitter.channel, 0, 1);
                SetChannelState(self.receiver[0].channel, 1, 0);
                if self.rx2_enabled {
                    SetChannelState(self.receiver[1].channel, 1, 0);
                }
            }
        }
    }

    fn config_file_path(device: Device) -> PathBuf {
        let d = format!("{:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}", device.mac[0], device.mac[1], device.mac[2], device.mac[3], device.mac[4], device.mac[5]);
        let app_name = env!("CARGO_PKG_NAME");
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join(app_name).join(d).join("radio.json")
    }

    pub fn load(device: Device, spectrum_width: i32) -> Self {
        let path = Self::config_file_path(device);
        if path.exists() {
            match File::open(&path) {
                Ok(mut file) => {
                    let mut s = String::new();
                    file.read_to_string(&mut s);
                    match serde_json::from_str::<Radio>(&s) {
                        Ok(mut radio) => {
                            println!("Successfully loaded data from {:?}", path);
                            radio.init();
                            radio
                        }
                        Err(_e) => {
                            Self::new(device, spectrum_width)
                        }
                    }
                },
                Err(_err) => {
                    Self::new(device, spectrum_width)
                }
            }
        } else {
            Self::new(device, spectrum_width)
        }
    }

    pub fn save(&self, device: Device) {
        let path = Self::config_file_path(device);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(_e) = create_dir_all(parent) {
                    return;
                }
            }
        }

        match serde_json::to_string_pretty(self) {
            Ok(s) => {
                match File::create(&path) {
                    Ok(mut file) => {
                        match file.write_all(s.as_bytes()) {
                            Ok(_file) => {
                                println!("Successfully saved data to {:?}", path);
                            },
                            Err(e) => {
                                eprintln!("Error writing config file {:?}: {}", path, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error creatoing config file: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error serializing data: {}", e);
            }
        }
    }

    pub fn add_notch_to_vector(&mut self, notch: Notch) {
        self.notches.push(notch);
    }

    pub fn add_notch(&mut self, notch: Notch) {
        unsafe {
            let _res = RXANBPAddNotch (notch.rx, self.notch, notch.frequency, notch.width, notch.active);
        }
        self.notch += 1;
    }

    // only called when radio is running protocol 1
    pub fn sample_rate_changed(&mut self, rate: i32) {
        self.sample_rate = rate;
        for i in 0..self.receivers {
            self.receiver[i as usize].sample_rate_changed(rate);
        }
        self.sample_rate_changed = true;
    }

}

fn format_u32_with_separators(value: u32) -> String {
    let mut result = String::new();
    let value_str = value.to_string(); 
    let len = value_str.len();

    // Iterate over the characters and insert separators
    for (i, ch) in value_str.chars().enumerate() {
        if (len - i) % 3 == 0 && i != 0 {
            write!(&mut result, ".").unwrap();
        }
        write!(&mut result, "{}", ch).unwrap();
    }                                   

    result
}

fn draw_meter(cr: &Context, dbm: f64) {
    let x_offset = 5.0;
    let y_offset = 0.0;
    let db = 1.0; // size in pixels of each dbm

    cr.set_source_rgb(0.0, 1.0, 0.0);
    cr.rectangle(x_offset, 0.0+y_offset, (dbm + 121.0) + db, 10.0);
    let _ = cr.fill();

    cr.set_source_rgb (0.0, 0.0, 0.0);
    for i in 0..54 {
        cr.move_to(x_offset+(i as f64 * db),10.0+y_offset);
        if i%18 == 0 {
            cr.line_to(x_offset+(i as f64 * db),0.0+y_offset);
        } else if i%6 == 0 {
            cr.line_to(x_offset+(i as f64 * db),5.0+y_offset);
        }
    }
    cr.move_to(x_offset+(54.0*db),10.0+y_offset);
    cr.line_to(x_offset+(54.0*db),0.0+y_offset);
    cr.move_to(x_offset+(74.0*db),10.0+y_offset);
    cr.line_to(x_offset+(74.0*db),0.0+y_offset);
    cr.move_to(x_offset+(94.0*db),10.0+y_offset);
    cr.line_to(x_offset+(94.0*db),0.0+y_offset);
    cr.move_to(x_offset+(114.0*db),10.0+y_offset);
    cr.line_to(x_offset+(114.0*db),0.0+y_offset);
    cr.stroke().unwrap();

    cr.move_to(x_offset+(18.0*db)-3.0,20.0+y_offset);
    let _ = cr.show_text("3");
    cr.move_to(x_offset+(36.0*db)-3.0,20.0+y_offset);
    let _ = cr.show_text("6");
    cr.move_to(x_offset+(54.0*db)-3.0,20.0+y_offset);
    let _ = cr.show_text("9");
    cr.move_to(x_offset+(74.0*db)-9.0,20.0+y_offset);
    let _ = cr.show_text("+20");
    cr.move_to(x_offset+(94.0*db)-9.0,20.0+y_offset);
    let _ = cr.show_text("+40");
    cr.move_to(x_offset+(114.0*db)-9.0,20.0+y_offset);
    let _ = cr.show_text("+60");

}

fn draw_waterfall(cr: &Context, width: i32, height: i32, pixbuf: &Rc<RefCell<Option<Pixbuf>>>) {
    let pixbuf_ref = pixbuf.borrow();
    if let Some(pixbuf) = pixbuf_ref.as_ref() {
        cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
        cr.paint().unwrap();
    } else {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        cr.fill().unwrap();
    }
}
