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
use gtk::{Builder, Button, Grid};

use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::antenna::Antenna;
use crate::modes::Modes;
use crate::filters::Filters;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bands {
   Band160=0,
   Band80,
   Band60,
   Band40,
   Band30,
   Band20,
   Band17,
   Band15,
   Band12,
   Band10,
   Band6,
   BandWWV,
   XVTR1,
   XVTR2,
   XVTR3,
}

impl Bands {
    pub fn from_usize(value: usize) -> Option<Self> {
        match value {
            0 => Some(Bands::Band160),
            1 => Some(Bands::Band80),
            2 => Some(Bands::Band60),
            3 => Some(Bands::Band40),
            4 => Some(Bands::Band30),
            5 => Some(Bands::Band20),
            6 => Some(Bands::Band17),
            7 => Some(Bands::Band15),
            8 => Some(Bands::Band12),
            9 => Some(Bands::Band10),
            10 => Some(Bands::Band6),
            11 => Some(Bands::BandWWV),
            12 => Some(Bands::XVTR1),
            13 => Some(Bands::XVTR2),
            14 => Some(Bands::XVTR3),
            _ => None,
        }
    }

    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}


// Def Serialize, Deserialize,ine a type for our callback function
pub type BandClickCallback = Box<dyn Fn(usize)>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BandInfo {
    pub band: Bands,
    pub label: String,
    pub low: f64,
    pub high: f64,
    pub current: f64,
    pub ctun: f64,
    pub filters: u32,
    pub spectrum_low: f32,
    pub spectrum_high: f32,
    pub waterfall_low: f32,
    pub waterfall_high: f32,
    pub mode: Modes,
    pub filter: Filters,
    pub attenuation: i32,
    pub antenna: Antenna,
    pub ext_antenna: Antenna,
    pub xvtr_antenna: Antenna,
    pub tx_antenna: Antenna,
    // for xvtrs
    pub lo: f64,        // Local Oscillator
    pub lo_error: f64,  // Oscillator error
}

impl BandInfo {
    pub fn new() -> Vec<BandInfo> {
        let bands = vec![
            BandInfo{ band: Bands::Band160, label: String::from("160"), low: 1800000.0, high: 2000000.0, current: 1900000.0, ctun: 1900000.0, filters: 0x01800040, spectrum_low: -120.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::LSB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::Band80, label: String::from("80"), low: 3500000.0, high: 3800000.0, current: 3750000.0, ctun: 3750000.0, filters: 0x01400020, spectrum_low: -100.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::LSB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::Band60, label: String::from("60"), low: 5330500.0, high: 5403500.0, current: 5365500.0, ctun: 5365500.0, filters: 0x01200020, spectrum_low: -110.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::LSB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::Band40, label: String::from("40"), low: 7000000.0, high: 7300000.0, current: 7150000.0, ctun: 7150000.0, filters: 0x01200010, spectrum_low: -110.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::LSB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::Band30, label: String::from("30"), low: 10100000.0, high: 10150000.0, current: 10125000.0, ctun: 10125000.0, filters: 0x01200010, spectrum_low: -110.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::USB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::Band20, label: String::from("20"), low: 14000000.0, high: 14350000.0, current: 14175000.0, ctun: 14175000.0, filters: 0x01100002, spectrum_low: -120.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::USB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::Band17, label: String::from("17"), low: 18068000.0, high: 18168000.0, current: 18118000.0, ctun: 18118000.0, filters: 0x81000002, spectrum_low: -120.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::USB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::Band15, label: String::from("15"), low: 21000000.0, high: 21450000.0, current: 21215000.0, ctun: 21215000.0, filters: 0x81000002, spectrum_low: -130.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::USB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::Band12, label: String::from("12"), low: 24890000.0, high: 24990000.0, current: 24940000.0, ctun: 24940000.0, filters: 0x41000004, spectrum_low: -130.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::USB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::Band10, label: String::from("10"), low: 28000000.0, high: 29700000.0, current: 28300000.0, ctun: 28300000.0, filters: 0x41000004, spectrum_low: -130.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::USB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::Band6, label: String::from("6"), low: 50000000.0, high: 54000000.0, current: 52000000.0, ctun: 52999999.0, filters: 0x21000008, spectrum_low: -120.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::USB, filter: Filters::F5, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::BandWWV, label: String::from("WWV"), low: 10000000.0, high: 10000000.0, current: 10000000.0, ctun: 10000000.0, filters: 0x20001000, spectrum_low: -120.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::SAM, filter: Filters::F3, antenna: Antenna::ANT1, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 0.0, lo_error: 0.0},
            BandInfo{ band: Bands::XVTR1, label: String::from("144"), low: 144000000.0, high: 148000000.0, current: 145000000.0, ctun: 145000000.0, filters: 0x41000004, spectrum_low: -120.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::FMN, filter: Filters::F3, antenna: Antenna::XVTR, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 116000000.0, lo_error: 0.0},
            BandInfo{ band: Bands::XVTR2, label: String::from("432"), low: 432000000.0, high: 436000000.0, current: 435000000.0, ctun: 435000000.0, filters: 0x41000004, spectrum_low: -120.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::FMN, filter: Filters::F3, antenna: Antenna::XVTR, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 404000000.0, lo_error: 0.0},
            BandInfo{ band: Bands::XVTR3, label: String::from("XVTR3"), low: 144000000.0, high: 148000000.0, current: 145000000.0, ctun: 145000000.0, filters: 0x41000004, spectrum_low: -120.0, spectrum_high: -60.0, waterfall_low: -130.0, waterfall_high: -80.0, mode: Modes::FMN, filter: Filters::F3, antenna: Antenna::XVTR, ext_antenna: Antenna::NONE, xvtr_antenna: Antenna::NONE, tx_antenna: Antenna::ANT1, attenuation: 0, lo: 116000000.0, lo_error: 0.0},
        ];
        bands
    }

/*
    pub fn find_band_for_freq(&self, freq: f64) -> Option<&BandInfo> {
        self.bands.iter().find(|band| {
            freq >= band.low && freq < band.high
        })
    }
*/
}

#[derive(Clone)]
pub struct BandGrid {
    pub grid: Grid,
    buttons: Vec<Button>,
    active_index: Rc<RefCell<Option<usize>>>,
    callback: Rc<RefCell<Box<dyn Fn(usize) + 'static>>>
}

impl BandGrid {
    pub fn new(builder: &Builder) -> Self {
        let grid: Grid = builder
                .object("band_grid")
                .expect("Could not get object 'band_grid' from builder.");
        let mut buttons = Vec::with_capacity(18);
        let active_index = Rc::new(RefCell::new(None));
        let callback = Rc::new(RefCell::new(Box::new(|_| {}) as Box<dyn Fn(usize)>));

        let band_info = BandInfo::new();
        for info in band_info.iter() {
            let id = format!("{:?}_button", info.band);
            let button: Button = builder
                .object(id)
                .expect("Could not get object band_button from builder.");
            // update the label
            button.set_label(&info.label);
            // Set initial button style class
            button.add_css_class("inactive-button");
            buttons.push(button.clone());
        }

        BandGrid {
            grid,
            buttons,
            active_index,
            callback,
        }
    }
            
    pub fn set_callback<F>(&mut self, callback: F, initial_button: usize)
    where
        F: Fn(usize) + 'static,
    {
        self.callback = Rc::new(RefCell::new(Box::new(callback)));
    
        let mut active_idx = self.active_index.borrow_mut();
        *active_idx = Some(initial_button);
        for (i, button) in self.buttons.iter().enumerate() {
            let callback_clone = self.callback.clone();
            let button_index = i;
            let active_index_clone = self.active_index.clone();
            let buttons_clone = self.buttons.clone();
            button.connect_clicked(move |clicked_button| {
                let mut active_idx = active_index_clone.borrow_mut();
                if let Some(prev_idx) = *active_idx {
                    let prev_button = &buttons_clone[prev_idx];
                    prev_button.remove_css_class("active-button");
                    prev_button.add_css_class("inactive-button");
                }

                // Set the style of the newly active button
                clicked_button.remove_css_class("inactive-button");
                clicked_button.add_css_class("active-button");

                // Update the active index
                *active_idx = Some(button_index);
                (callback_clone.borrow())(button_index);
            });
            if i == initial_button {
                button.remove_css_class("inactive-button");
                button.add_css_class("active-button");
            }
        }
    }

    pub fn get_active_index(&self) -> Option<usize> {
        *self.active_index.borrow()
    }

    pub fn set_active_index(&mut self, index: usize) {
        let old_index: usize = self.active_index.borrow().expect("Bands: set_active_index error using active_index");
        self.buttons[old_index].remove_css_class("active-button");
        self.buttons[old_index].add_css_class("inactive-button");
        let mut active_idx = self.active_index.borrow_mut();
        *active_idx = Some(index);
        self.buttons[index].remove_css_class("inactive-button");
        self.buttons[index].add_css_class("active-button");
    }

    pub fn get_button(&self, index: usize) -> &Button {
        &self.buttons[index]
    }

    pub fn get_widget(&self) -> &Grid {
        &self.grid
    }

    pub fn update_band_label(&mut self, band: Bands, label: &str) {
        self.buttons[band.to_usize()].set_label(label);
    }

}

