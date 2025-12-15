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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Filters {
    F0 = 0,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    FVar1,
    FVar2,
}

impl Filters {
    pub fn from_usize(value: usize) -> Option<Self> {
        match value {
            0 => Some(Filters::F0),
            1 => Some(Filters::F1),
            2 => Some(Filters::F2),
            3 => Some(Filters::F3),
            4 => Some(Filters::F4),
            5 => Some(Filters::F5),
            6 => Some(Filters::F6),
            7 => Some(Filters::F7),
            8 => Some(Filters::F8),
            9 => Some(Filters::F9),
            10 => Some(Filters::FVar1),
            11 => Some(Filters::FVar2),
            _ => None,
        }
    }

    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}

use gtk::prelude::*;
use gtk::{Adjustment, Builder, Button, Grid, SpinButton};
use std::cell::RefCell;
use std::rc::Rc;
use crate::modes::Modes;

// Define a type for our callback function
pub type FilterClickCallback = Box<dyn Fn(usize)>;

pub struct Filter {
    low: f64,
    high: f64,
    label: &'static str,
}

#[derive(Clone)]
pub struct FilterGrid {
    pub grid: Grid,
    buttons: Vec<Button>,
    low_spinbutton: SpinButton,
    low_adjustment: Adjustment,
    high_spinbutton: SpinButton,
    high_adjustment: Adjustment,
    active_index: Rc<RefCell<Option<usize>>>,
    callback: Rc<RefCell<Box<dyn Fn(usize) + 'static>>>
}

impl FilterGrid {

        const filterLSB: [Filter; 12] = [
            Filter {low: -5150.0, high: -150.0, label: "5.0k"},
            Filter {low: -4550.0, high: -150.0, label: "4.4k"},
            Filter {low: -3950.0, high: -150.0, label: "3.8k"},
            Filter {low: -3450.0, high: -150.0, label: "3.3k"},
            Filter {low: -3050.0, high: -150.0, label: "2.9k"},
            Filter {low: -2850.0, high: -150.0, label: "2.7k"},
            Filter {low: -2550.0, high: -150.0, label: "2.4k"},
            Filter {low: -2250.0, high: -150.0, label: "2.1k"},
            Filter {low: -1950.0, high: -150.0, label: "1.8k"},
            Filter {low: -1150.0, high: -150.0, label: "1.0k"},
            Filter {low: -2850.0, high: -150.0, label: "Var1"},
            Filter {low: -2850.0, high: -150.0, label: "Var2"},
        ];
      
        const filterUSB: [Filter; 12] = [
            Filter {low: 150.0, high: 5150.0, label: "5.0k"},
            Filter {low: 150.0, high: 4550.0, label: "4.4k"},
            Filter {low: 150.0, high: 3950.0, label: "3.8k"},
            Filter {low: 150.0, high: 3450.0, label: "3.3k"},
            Filter {low: 150.0, high: 3050.0, label: "2.9k"},
            Filter {low: 150.0, high: 2850.0, label: "2.7k"},
            Filter {low: 150.0, high: 2550.0, label: "2.4k"},
            Filter {low: 150.0, high: 2250.0, label: "2.1k"},
            Filter {low: 150.0, high: 1950.0, label: "1.8k"},
            Filter {low: 150.0, high: 1150.0, label: "1.0k"},
            Filter {low: 150.0, high: 2850.0, label: "Var1"},
            Filter {low: 150.0, high: 2850.0, label: "Var2"},
        ];

        const filterCWL: [Filter; 12] = [
            Filter {low: -500.0, high: 500.0, label: "1.0k"},
            Filter {low: -400.0, high: 400.0, label: "800"},
            Filter {low: -375.0, high: 375.0, label: "750"},
            Filter {low: -300.0, high: 300.0, label: "600"},
            Filter {low: -250.0, high: 250.0, label: "500"},
            Filter {low: -200.0, high: 200.0, label: "400"},
            Filter {low: -125.0, high: 125.0, label: "250"},
            Filter {low: -50.0, high: 50.0, label: "100"},
            Filter {low: -25.0, high: 25.0, label: "50"},
            Filter {low: -13.0, high: 13.0, label: "25"},
            Filter {low: -100.0, high: 100.0, label: "Var1"},
            Filter {low: -100.0, high: 100.0, label: "Var2"},
        ];

        const filterCWU: [Filter; 12] = [
            Filter {low: -500.0, high: 500.0, label: "1.0k"},
            Filter {low: -400.0, high: 400.0, label: "800"},
            Filter {low: -375.0, high: 375.0, label: "750"},
            Filter {low: -300.0, high: 300.0, label: "600"},
            Filter {low: -250.0, high: 250.0, label: "500"},
            Filter {low: -200.0, high: 200.0, label: "400"},
            Filter {low: -125.0, high: 125.0, label: "250"},
            Filter {low: -50.0, high: 50.0, label: "100"},
            Filter {low: -25.0, high: 25.0, label: "50"},
            Filter {low: -13.0, high: 13.0, label: "25"},
            Filter {low: -100.0, high: 100.0, label: "Var1"},
            Filter {low: -100.0, high: 100.0, label: "Var2"},
        ];

        const filterDIGL: [Filter; 12] = [
            Filter {low: -5150.0, high: -150.0, label: "5.0k"},
            Filter {low: -4550.0, high: -150.0, label: "4.4k"},
            Filter {low: -3950.0, high: -150.0, label: "3.8k"},
            Filter {low: -3450.0, high: -150.0, label: "3.3k"},
            Filter {low: -3050.0, high: -150.0, label: "2.9k"},
            Filter {low: -2850.0, high: -150.0, label: "2.7k"},
            Filter {low: -2550.0, high: -150.0, label: "2.4k"},
            Filter {low: -2250.0, high: -150.0, label: "2.1k"},
            Filter {low: -1950.0, high: -150.0, label: "1.8k"},
            Filter {low: -1150.0, high: -150.0, label: "1.0k"},
            Filter {low: -4000.0, high: 0.0, label: "Var1"},
            Filter {low: -2850.0, high: -150.0, label: "Var2"}
        ];

        const filterDIGU: [Filter; 12] = [
            Filter {low: 150.0, high: 5150.0, label: "5.0k"},
            Filter {low: 150.0, high: 4550.0, label: "4.4k"},
            Filter {low: 150.0, high: 3950.0, label: "3.8k"},
            Filter {low: 150.0, high: 3450.0, label: "3.3k"},
            Filter {low: 150.0, high: 3050.0, label: "2.9k"},
            Filter {low: 150.0, high: 2850.0, label: "2.7k"},
            Filter {low: 150.0, high: 2550.0, label: "2.4k"},
            Filter {low: 150.0, high: 2250.0, label: "2.1k"},
            Filter {low: 150.0, high: 1950.0, label: "1.8k"},
            Filter {low: 150.0, high: 1150.0, label: "1.0k"},
            Filter {low: 0.0, high: 4000.0, label: "Var1"},
            Filter {low: 150.0, high: 2850.0, label: "Var2"},
        ];

        const filterAM: [Filter; 12] = [
            Filter {low: -8000.0, high: 8000.0, label: "16k"},
            Filter {low: -6000.0, high: 6000.0, label: "12k"},
            Filter {low: -5000.0, high: 5000.0, label: "10k"},
            Filter {low: -4000.0, high: 4000.0, label: "8k"},
            Filter {low: -3300.0, high: 3300.0, label: "6.6k"},
            Filter {low: -2600.0, high: 2600.0, label: "5.2k"},
            Filter {low: -2000.0, high: 2000.0, label: "4.0k"},
            Filter {low: -1550.0, high: 1550.0, label: "3.1k"},
            Filter {low: -1450.0, high: 1450.0, label: "2.9k"},
            Filter {low: -1200.0, high: 1200.0, label: "2.4k"},
            Filter {low: -3300.0, high: 3300.0, label: "Var1"},
            Filter {low: -3300.0, high: 3300.0, label: "Var2"},
        ];

        const filterSAM: [Filter; 12] = [
            Filter {low: -8000.0, high: 8000.0, label: "16k"},
            Filter {low: -6000.0, high: 6000.0, label: "12k"},
            Filter {low: -5000.0, high: 5000.0, label: "10k"},
            Filter {low: -4000.0, high: 4000.0, label: "8k"},
            Filter {low: -3300.0, high: 3300.0, label: "6.6k"},
            Filter {low: -2600.0, high: 2600.0, label: "5.2k"},
            Filter {low: -2000.0, high: 2000.0, label: "4.0k"},
            Filter {low: -1550.0, high: 1550.0, label: "3.1k"},
            Filter {low: -1450.0, high: 1450.0, label: "2.9k"},
            Filter {low: -1200.0, high: 1200.0, label: "2.4k"},
            Filter {low: -3300.0, high: 3300.0, label: "Var1"},
            Filter {low: -3300.0, high: 3300.0, label: "Var2"},
        ];

        const filterFMN: [Filter; 12] = [
            Filter {low: -8000.0, high: 8000.0, label: "8k"},
            Filter {low: -6000.0, high: 6000.0, label: "16k"},
            Filter {low: -5000.0, high: 5000.0, label: "10k"},
            Filter {low: -4000.0, high: 4000.0, label: "8k"},
            Filter {low: -3300.0, high: 3300.0, label: "6.6k"},
            Filter {low: -2600.0, high: 2600.0, label: "5.2k"},
            Filter {low: -2000.0, high: 2000.0, label: "4.0k"},
            Filter {low: -1550.0, high: 1550.0, label: "3.1k"},
            Filter {low: -1450.0, high: 1450.0, label: "2.9k"},
            Filter {low: -1200.0, high: 1200.0, label: "2.4k"},
            Filter {low: -3300.0, high: 3300.0, label: "Var1"},
            Filter {low: -3300.0, high: 3300.0, label: "Var2"},
        ];

        const filterDSB: [Filter; 12] = [
            Filter {low: -8000.0, high: 8000.0, label: "16k"},
            Filter {low: -6000.0, high: 6000.0, label: "12k"},
            Filter {low: -5000.0, high: 5000.0, label: "10k"},
            Filter {low: -4000.0, high: 4000.0, label: "8k"},
            Filter {low: -3300.0, high: 3300.0, label: "6.6k"},
            Filter {low: -2600.0, high: 2600.0, label: "5.2k"},
            Filter {low: -2000.0, high: 2000.0, label: "4.0k"},
            Filter {low: -1550.0, high: 1550.0, label: "3.1k"},
            Filter {low: -1450.0, high: 1450.0, label: "2.9k"},
            Filter {low: -1200.0, high: 1200.0, label: "2.4k"},
            Filter {low: -3300.0, high: 3300.0, label: "Var1"},
            Filter {low: -3300.0, high: 3300.0, label: "Var2"},
        ];

        const filterSPEC: [Filter; 12] = [
            Filter {low: -8000.0, high: 8000.0, label: "16k"},
            Filter {low: -6000.0, high: 6000.0, label: "12k"},
            Filter {low: -5000.0, high: 5000.0, label: "10k"},
            Filter {low: -4000.0, high: 4000.0, label: "8k"},
            Filter {low: -3300.0, high: 3300.0, label: "6.6k"},
            Filter {low: -2600.0, high: 2600.0, label: "5.2k"},
            Filter {low: -2000.0, high: 2000.0, label: "4.0k"},
            Filter {low: -1550.0, high: 1550.0, label: "3.1k"},
            Filter {low: -1450.0, high: 1450.0, label: "2.9k"},
            Filter {low: -1200.0, high: 1200.0, label: "2.4k"},
            Filter {low: -3300.0, high: 3300.0, label: "Var1"},
            Filter {low: -3300.0, high: 3300.0, label: "Var2"},
        ];

        const filterDRM: [Filter; 12] = [
            Filter {low: -8000.0, high: 8000.0, label: "16k"},
            Filter {low: -6000.0, high: 6000.0, label: "12k"},
            Filter {low: -5000.0, high: 5000.0, label: "10k"},
            Filter {low: -4000.0, high: 4000.0, label: "8k"},
            Filter {low: -3300.0, high: 3300.0, label: "6.6k"},
            Filter {low: -2600.0, high: 2600.0, label: "5.2k"},
            Filter {low: -2000.0, high: 2000.0, label: "4.0k"},
            Filter {low: -1550.0, high: 1550.0, label: "3.1k"},
            Filter {low: -1450.0, high: 1450.0, label: "2.9k"},
            Filter {low: -1200.0, high: 1200.0, label: "2.4k"},
            Filter {low: -3300.0, high: 3300.0, label: "Var1"},
            Filter {low: -3300.0, high: 3300.0, label: "Var2"},
        ];

    pub fn new(builder: &Builder) -> Self {
        let grid: Grid = builder
            .object("filter_grid")
            .expect("Could not get object 'filtermode_grid' from builder.");

        let mut buttons = Vec::with_capacity(15);
        let active_index = Rc::new(RefCell::new(None));
        let callback = Rc::new(RefCell::new(Box::new(|_: usize| {}) as Box<dyn Fn(usize)>));
    
        let labels = [
            "f0", "f1", "f2",
            "f3", "f4", "f5",
            "f6", "f7", "f8",
            "f9", "var1", "var2",
        ];

        for &label in labels.iter() {
            let id = format!("{}_button", label);
            let button: Button = builder
                .object(id)
                .expect("Could not get object filter_button from builder.");

            button.add_css_class("inactive-button");
            buttons.push(button.clone());
        }

        let low_spinbutton: SpinButton = builder
            .object("low_spinbutton")
            .expect("Could not get object low_spinbutton from builder.");

        let low_adjustment: Adjustment = builder
            .object("low_adjustment")
            .expect("Could not get object low_adjustment from builder.");

        let high_spinbutton: SpinButton = builder
            .object("high_spinbutton")
            .expect("Could not get object high_spinbutton from builder.");

        let high_adjustment: Adjustment = builder
            .object("high_adjustment")
            .expect("Could not get object high_adjustment from builder.");


        FilterGrid {
            grid,
            buttons,
            low_spinbutton,
            low_adjustment,
            high_spinbutton,
            high_adjustment,
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

        if initial_button == 10 || initial_button == 11 {
            self.low_spinbutton.set_sensitive(true);
            self.high_spinbutton.set_sensitive(true);
        } else {
            self.low_spinbutton.set_sensitive(false);
            self.high_spinbutton.set_sensitive(false);
        }
        
        // now add the callback
        for (i, button) in self.buttons.iter().enumerate() {
            let callback_clone = self.callback.clone();
            let button_index = i;
            let active_index_clone = self.active_index.clone();
            let buttons_clone = self.buttons.clone();
            let low_spinbutton_clone = self.low_spinbutton.clone();
            let high_spinbutton_clone = self.high_spinbutton.clone();
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

                if button_index == 10 || button_index == 11 {
                    low_spinbutton_clone.set_sensitive(true);
                    high_spinbutton_clone.set_sensitive(true);
                } else {
                    low_spinbutton_clone.set_sensitive(false);
                    high_spinbutton_clone.set_sensitive(false);
                }

                (callback_clone.borrow())(button_index);
            });

            if i == initial_button {
                button.remove_css_class("inactive-button");
                button.add_css_class("active-button");
            }
        }
    }

    pub fn get_widget(&self) -> &Grid {
        &self.grid
    }

    pub fn get_filter_values(&self,mode: usize, filter: usize) -> (f64, f64) {
        let m = match Modes::from_usize(mode) {
           Some(Modes::LSB) => Self::filterLSB,
           Some(Modes::USB) => Self::filterUSB,
           Some(Modes::DSB) => Self::filterDSB,
           Some(Modes::CWL) => Self::filterCWL,
           Some(Modes::CWU) => Self::filterCWU,
           Some(Modes::FMN) => Self::filterFMN,
           Some(Modes::AM) => Self::filterAM,
           Some(Modes::DIGU) => Self::filterDIGU,
           Some(Modes::SPEC) => Self::filterSPEC,
           Some(Modes::DIGL) => Self::filterDIGL,
           Some(Modes::SAM) => Self::filterSAM,
           Some(Modes::DRM) => Self::filterDRM,
           None => Self::filterUSB,
        };
        (m[filter].low, m[filter].high)
    }

    pub fn update_filter_buttons(&self, mode: usize) {
        let filters = match Modes::from_usize(mode) {
           Some(Modes::LSB) => Self::filterLSB,
           Some(Modes::USB) => Self::filterUSB,
           Some(Modes::DSB) => Self::filterDSB,
           Some(Modes::CWL) => Self::filterCWL,
           Some(Modes::CWU) => Self::filterCWU,
           Some(Modes::FMN) => Self::filterFMN,
           Some(Modes::AM) => Self::filterAM,
           Some(Modes::DIGU) => Self::filterDIGU,
           Some(Modes::SPEC) => Self::filterSPEC,
           Some(Modes::DIGL) => Self::filterDIGL,
           Some(Modes::SAM) => Self::filterSAM,
           Some(Modes::DRM) => Self::filterDRM,
           None => Self::filterUSB,
        };

        for (i, button) in self.buttons.iter().enumerate() {
            button.set_label(filters[i].label);
        }
    }

    pub fn get_active_index(&self) -> Option<usize> {
        *self.active_index.borrow()
    }

    pub fn set_active_index(&self, index: usize) {
        let old_index: usize = self.active_index.borrow().expect("Filters: set_active_index error using active_index");
        self.buttons[old_index].remove_css_class("active-button");
        self.buttons[old_index].add_css_class("inactive-button");
        let mut active_idx = self.active_index.borrow_mut();
        *active_idx = Some(index);
        self.buttons[index].remove_css_class("inactive-button");
        self.buttons[index].add_css_class("active-button");
        if index == 10 || index == 11 {
            self.low_spinbutton.set_sensitive(true);
            self.high_spinbutton.set_sensitive(true);
        } else {
            self.low_spinbutton.set_sensitive(false);
            self.high_spinbutton.set_sensitive(false);
        }
    }

    pub fn get_button(&self, index: usize) -> &Button {
        &self.buttons[index]
    }

    pub fn set_active_values(&self, low: f64, high: f64) {
        self.low_adjustment.set_value(low);
        self.high_adjustment.set_value(high);
    }

    pub fn set_filter_low(&self, low: f64, mode: usize, filter: usize) {
        let mut m = match Modes::from_usize(mode) {
           Some(Modes::LSB) => Self::filterLSB,
           Some(Modes::USB) => Self::filterUSB,
           Some(Modes::DSB) => Self::filterDSB,
           Some(Modes::CWL) => Self::filterCWL,
           Some(Modes::CWU) => Self::filterCWU,
           Some(Modes::FMN) => Self::filterFMN,
           Some(Modes::AM) => Self::filterAM,
           Some(Modes::DIGU) => Self::filterDIGU,
           Some(Modes::SPEC) => Self::filterSPEC,
           Some(Modes::DIGL) => Self::filterDIGL,
           Some(Modes::SAM) => Self::filterSAM,
           Some(Modes::DRM) => Self::filterDRM,
           None => Self::filterUSB,
        };
        m[filter].low = low;
    }

    pub fn set_filter_high(&self, high: f64, mode: usize, filter: usize) {
        let mut m = match Modes::from_usize(mode) {
           Some(Modes::LSB) => Self::filterLSB,
           Some(Modes::USB) => Self::filterUSB,
           Some(Modes::DSB) => Self::filterDSB,
           Some(Modes::CWL) => Self::filterCWL,
           Some(Modes::CWU) => Self::filterCWU,
           Some(Modes::FMN) => Self::filterFMN,
           Some(Modes::AM) => Self::filterAM,
           Some(Modes::DIGU) => Self::filterDIGU,
           Some(Modes::SPEC) => Self::filterSPEC,
           Some(Modes::DIGL) => Self::filterDIGL,
           Some(Modes::SAM) => Self::filterSAM,
           Some(Modes::DRM) => Self::filterDRM,
           None => Self::filterUSB,
        };
        m[filter].high = high;
    }

}
