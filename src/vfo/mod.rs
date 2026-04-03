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
use gtk::{Adjustment, Builder, Button, Window};

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::bands::*;
use crate::modes::*;
use crate::filters::*;
use crate::radio::{Keyer, RadioModels, RadioMutex};
use crate::receiver::*;
use crate::wdsp::*;
use crate::widgets::*;
use crate::util::*;



pub fn create_vfo_dialog(rc_app_widgets: &Rc<RefCell<AppWidgets>>, radio_mutex: &RadioMutex, rx: usize) -> Window {
    let ui_xml = include_str!("../ui/vfo.xml");
    let builder = Builder::from_string(ui_xml);

    let window: Window = builder
            .object("vfo_window")
            .expect("Could not get object `vfo_window` from builder.");

    let id = if rx==0 {"A"} else {"B"};
    let title = format!("rustyHPSDR: VFO {}", id);
    window.set_title(Some(&title));

    let app_widgets = rc_app_widgets.borrow();
    window.set_transient_for(Some(&app_widgets.main_window)); // keeps it on top

    let r = radio_mutex.radio.lock().unwrap();
    let band = r.receiver[rx].band;
    let mode = r.receiver[rx].mode;
    let filter = r.receiver[rx].filter;
    let low = r.receiver[rx].filter_low;
    let high = r.receiver[rx].filter_high;
    let cw_pitch = r.receiver[rx].cw_pitch;
    drop(r);

    let mut band_grid = BandGrid::new(&builder);
    let mut mode_grid = ModeGrid::new(&builder);
    let mut filter_grid = FilterGrid::new(&builder);

    let rc_app_widgets_clone = rc_app_widgets.clone();
    let radio_mutex_clone = radio_mutex.clone();
    let band_grid_clone = band_grid.clone();
    let mode_grid_clone = mode_grid.clone();
    let filter_grid_clone = filter_grid.clone();
    band_grid.set_callback(move|index| {
        let app_widgets = rc_app_widgets_clone.borrow();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let b = r.receiver[rx].band.to_usize();
        if b != index { // band has changed
            // save current band info
            r.receiver[rx].band_info[b].current = r.receiver[rx].frequency;
            r.receiver[rx].band_info[b].ctun = r.receiver[rx].ctun_frequency;
            r.receiver[rx].band_info[b].mode = Modes::from_usize(r.receiver[rx].mode).expect("Invalid mode");
            r.receiver[rx].band_info[b].filter = Filters::from_usize(r.receiver[rx].filter).expect("Invalid Filter");

            // get new band info
            r.receiver[rx].band = Bands::from_usize(index).expect("invalid band index");
            r.receiver[rx].frequency = r.receiver[rx].band_info[index].current;
            r.receiver[rx].ctun_frequency = r.receiver[rx].band_info[index].ctun;
            if r.receiver[rx].ctun {
                r.receiver[rx].set_ctun_frequency();
            }

            if !r.receiver[rx].filters_manual {
                r.receiver[rx].filters = r.receiver[rx].band_info[index].filters;
            }
        }
        filter_grid_clone.update_filter_buttons(r.receiver[rx].band_info[index].mode.to_usize());
        filter_grid_clone.set_active_index(r.receiver[rx].band_info[index].filter.to_usize());

        let f = r.receiver[rx].frequency;
        if b != index { // band has changed
            r.receiver[rx].mode = r.receiver[rx].band_info[index].mode.to_usize();
            mode_grid_clone.set_active_index(r.receiver[rx].mode);
            let (mut low, mut high) = filter_grid_clone.get_filter_values(r.receiver[rx].band_info[index].mode.to_usize(), r.receiver[rx].band_info[index].filter.to_usize());
            filter_grid_clone.set_active_values(low, high);
            if r.receiver[rx].mode == Modes::CWL.to_usize() {
                low += -r.receiver[rx].cw_pitch;
                high += -r.receiver[rx].cw_pitch;
            } else if r.receiver[rx].mode == Modes::CWU.to_usize() {
                low += r.receiver[rx].cw_pitch;
                high += r.receiver[rx].cw_pitch;
            }
            r.receiver[rx].filter_low = low;
            r.receiver[rx].filter_high = high;
            r.receiver[rx].set_mode();

            r.transmitter.filter_low = low;
            r.transmitter.filter_high = high;
            r.transmitter.mode = r.receiver[rx].band_info[index].mode.to_usize();
            r.transmitter.set_mode();
            r.transmitter.set_filter();

            let formatted_value = format_u32_with_separators(
                                      if r.receiver[rx].ctun {
                                          r.receiver[rx].ctun_frequency as u32
                                      } else {
                                           r.receiver[rx].frequency as u32
                                      });
            if rx == 0 {
                app_widgets.vfo_a_frequency.set_label(&formatted_value);
            } else {
                app_widgets.vfo_b_frequency.set_label(&formatted_value);
            }

            let mut b = r.receiver[rx].band.to_usize();
            let mut attenuation = r.receiver[rx].band_info[b].attenuation;
            if r.dev == 6 { // HEMES_LITE
                b = r.receiver[0].band.to_usize();
                attenuation = r.receiver[0].band_info[b].attenuation;
            }
            drop(r);
            app_widgets.attenuation_adjustment.set_value(attenuation.into());
        }
        unsafe {
            RXANBPSetTuneFrequency(rx as i32, f as f64);
        }
    }, band.to_usize());

    let radio_mutex_clone = radio_mutex.clone();
    let filter_grid_clone = filter_grid.clone();
    mode_grid.set_callback(move|index| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = if r.receiver[0].active { 0 } else { 1 };
        r.receiver[rx].mode = index; 
        filter_grid_clone.update_filter_buttons(index);
            
        let (mut low, mut high) = filter_grid_clone.get_filter_values(r.receiver[rx].mode, r.receiver[rx].filter);
        filter_grid_clone.set_active_values(low, high);
        if r.receiver[rx].mode == Modes::CWL.to_usize() {
            low += -r.receiver[rx].cw_pitch;
            high += -r.receiver[rx].cw_pitch;
        } else if r.receiver[rx].mode == Modes::CWU.to_usize() {
            low += r.receiver[rx].cw_pitch;
            high += r.receiver[rx].cw_pitch;
        }
        r.receiver[rx].filter_low = low;
        r.receiver[rx].filter_high = high;
        r.receiver[rx].set_mode();
        r.transmitter.mode = index;
        r.transmitter.set_mode();
        r.transmitter.filter_low = low; 
        r.transmitter.filter_high = high;
        r.transmitter.set_filter();
    }, mode);

    let radio_mutex_clone = radio_mutex.clone();
    let filter_grid_clone = filter_grid.clone();
    filter_grid.set_callback(move|index| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = if r.receiver[0].active { 0 } else { 1 };
        r.receiver[rx].filter = index;
        let (mut low, mut high) = filter_grid_clone.get_filter_values(r.receiver[rx].mode, r.receiver[rx].filter);
        if r.receiver[rx].mode == Modes::CWL.to_usize() {
            low += -r.receiver[rx].cw_pitch;
            high += -r.receiver[rx].cw_pitch;
        } else if r.receiver[rx].mode == Modes::CWU.to_usize() {
            low += r.receiver[rx].cw_pitch;
            high += r.receiver[rx].cw_pitch;
        }

        filter_grid_clone.set_active_values(low, high);
        r.receiver[rx].filter_low = low;
        r.receiver[rx].filter_high = high;
        r.receiver[rx].set_filter();
        r.transmitter.filter_low = low;
        r.transmitter.filter_high = high;
        r.transmitter.set_filter();
    }, filter);


    let cw_pitch_adjustment: Adjustment = builder
            .object("cw_pitch_adjustment")
            .expect("Could not get object `cw_pitch_adjustment` from builder.");
    cw_pitch_adjustment.set_value(cw_pitch);
    let radio_mutex_clone = radio_mutex.clone();
    let filter_grid_clone = filter_grid.clone();
    cw_pitch_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        let b = r.receiver[rx].band.to_usize();
        let m = r.receiver[rx].mode;
        let f = r.receiver[rx].filter;
        r.receiver[rx].cw_pitch = adjustment.value() as f64;
        let (mut low, mut high) = filter_grid_clone.get_filter_values(m, f);

eprintln!("cw_pitch_adjustment band={} mode={} filter={} low={} high={}",
         b, m, f, low, high);
        if r.receiver[rx].mode == Modes::CWL.to_usize() {
            low += -r.receiver[rx].cw_pitch;
            high += -r.receiver[rx].cw_pitch;
        } else if r.receiver[rx].mode == Modes::CWU.to_usize() {
            low += r.receiver[rx].cw_pitch;
            high += r.receiver[rx].cw_pitch;
        }
        r.receiver[rx].filter_low = low;
        r.receiver[rx].filter_high = high;
        r.receiver[rx].set_filter();
    });

    band_grid.set_active_index(band.to_usize());
    mode_grid.set_active_index(mode);
    filter_grid.update_filter_buttons(mode);
    filter_grid.set_active_index(filter);
    filter_grid.set_active_values(low, high);


    // OK button
    let ok_button: Button = builder
            .object("ok_button")
            .expect("Could not get object `ok_button` from builder.");
    let window_for_ok = window.clone();
    ok_button.connect_clicked(move |_| {
        window_for_ok.close();
    });

    window
    
}
