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
pub enum Modes {
    LSB = 0,
    USB,
    DSB,
    CWL,
    CWU,
    FMN,
    AM,
    DIGU,
    SPEC,
    DIGL,
    SAM,
    DRM,
}

impl Modes {
    pub fn from_usize(value: usize) -> Option<Self> {
        match value {
            0 => Some(Modes::LSB),
            1 => Some(Modes::USB),
            2 => Some(Modes::DSB),
            3 => Some(Modes::CWL),
            4 => Some(Modes::CWU),
            5 => Some(Modes::FMN),
            6 => Some(Modes::AM),
            7 => Some(Modes::DIGU),
            8 => Some(Modes::SPEC),
            9 => Some(Modes::DIGL),
            10 => Some(Modes::SAM),
            11 => Some(Modes::DRM),
            _ => None,
        }
    }

    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}


use gtk::prelude::*;
use gtk::{Builder, Button, Grid};
use std::cell::RefCell;
use std::rc::Rc;

// Define a type for our callback function
pub type ModeClickCallback = Box<dyn Fn(usize)>;

#[derive(Clone)]
pub struct ModeGrid {
    pub grid: Grid,
    buttons: Vec<Button>,
    active_index: Rc<RefCell<Option<usize>>>,
    callback: Rc<RefCell<Box<dyn Fn(usize) + 'static>>>
}

impl ModeGrid {
    pub fn new(builder: &Builder) -> Self {
        let grid: Grid = builder
            .object("mode_grid")
            .expect("Could not get object 'mode_grid' from builder."); 
        
        let labels = [
        "lsb", "usb", "dsb",
        "cwl", "cwu", "fmn",
        "am", "digu", "spec",
        "digl", "sam", "drm",
        ];

        let mut buttons = Vec::with_capacity(12);
        for &label in labels.iter() {
            let id = format!("{}_button", label);
            let button: Button = builder
                .object(id)
                .expect("Could not get object mode_button from builder.");
            button.add_css_class("inactive-button");
            buttons.push(button.clone());
        }

        let active_index = Rc::new(RefCell::new(None));
        let callback = Rc::new(RefCell::new(Box::new(|_| {}) as Box<dyn Fn(usize)>));

        ModeGrid {
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
            
        // now add the callback
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

    pub fn set_active_index(&self, index: usize) {
        let old_index: usize = self.active_index.borrow().expect("Modes: set_active_index error using active_index");
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

}

