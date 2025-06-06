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
use gtk::{Adjustment, Align, ApplicationWindow, Button, CheckButton, Grid, Label, Notebook, Orientation, PositionType, Scale, SpinButton, Window};

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::radio::Radio;
use crate::audio::*;

pub fn create_configure_dialog(parent: &ApplicationWindow, radio: &Arc<Mutex<Radio>>) -> Window {

    let window = Window::builder()
        .title("rustyHPSDR Discovery")
        .modal(true)
        .transient_for(parent)
        .destroy_with_parent(true)
        .default_width(800)
        .default_height(200)
        .build();

    let notebook = Notebook::new();

    let mut r = radio.lock().unwrap();
    let (grid, label, audio) = r.audio.configure();
    drop(r);
    notebook.append_page(&grid, Some(&label));

    let display_label = Label::new(Some("Display"));
    let display_grid = Grid::builder()
            .margin_start(0)
            .margin_end(0)
            .margin_top(0)
            .margin_bottom(0)
            .halign(Align::Center)
            .valign(Align::Center)
            .row_spacing(0)
            .column_spacing(0)
            .build();

    display_grid.set_column_homogeneous(true);
    display_grid.set_row_homogeneous(true);

    let band_title = Label::new(Some("Band"));
    display_grid.attach(&band_title, 0, 0, 1, 1);
    let spectrum_low_title = Label::new(Some("Spectrum Low"));
    display_grid.attach(&spectrum_low_title, 1, 0, 1, 1);
    let spectrum_high_title = Label::new(Some("Spectrum High"));
    display_grid.attach(&spectrum_high_title, 2, 0, 1, 1);
    let waterfall_low_title = Label::new(Some("Waterfall Low"));
    display_grid.attach(&waterfall_low_title, 3, 0, 1, 1);
    let waterfall_high_title = Label::new(Some("Waterfall High"));
    display_grid.attach(&waterfall_high_title, 4, 0, 1, 1);

    let band_info = radio.lock().unwrap().band_info.clone();

    for (i, info) in band_info.iter().enumerate() {

        let row = (i+1) as i32;
        let band_label = Label::new(Some(info.label.as_str()));
        display_grid.attach(&band_label, 0, row, 1, 1);

        let spectrum_low_spin_button = SpinButton::with_range(-140.0, -40.0, 1.0);
        spectrum_low_spin_button.set_value(info.spectrum_low.into());
        display_grid.attach(&spectrum_low_spin_button, 1, row, 1, 1);

        let radio_clone = Arc::clone(&radio);
        let band_index = i;
        spectrum_low_spin_button.connect_value_changed(move |spin_button| {
            let value = spin_button.value() as f32;
            let mut radio_lock = radio_clone.lock().unwrap();
            radio_lock.band_info[band_index].spectrum_low = value;
        });

        let spectrum_high_spin_button = SpinButton::with_range(-140.0, -40.0, 1.0);
        spectrum_high_spin_button.set_value(info.spectrum_high.into());
        display_grid.attach(&spectrum_high_spin_button, 2, row, 1, 1);

        let radio_clone = Arc::clone(&radio);
        let band_index = i;
        spectrum_high_spin_button.connect_value_changed(move |spin_button| {
            let value = spin_button.value() as f32;
            let mut radio_lock = radio_clone.lock().unwrap();
            radio_lock.band_info[band_index].spectrum_high = value;
        });

        let waterfall_low_spin_button = SpinButton::with_range(-140.0, -40.0, 1.0);
        waterfall_low_spin_button.set_value(info.waterfall_low.into());
        display_grid.attach(&waterfall_low_spin_button, 3, row, 1, 1);

        let radio_clone = Arc::clone(&radio);
        let band_index = i;
        waterfall_low_spin_button.connect_value_changed(move |spin_button| {
            let value = spin_button.value() as f32;
            let mut radio_lock = radio_clone.lock().unwrap();
            radio_lock.band_info[band_index].waterfall_low = value;
        });

        let waterfall_high_spin_button = SpinButton::with_range(-140.0, -40.0, 1.0);
        waterfall_high_spin_button.set_value(info.waterfall_high.into());
        display_grid.attach(&waterfall_high_spin_button, 4, row, 1, 1);

        let radio_clone = Arc::clone(&radio);
        let band_index = i;
        waterfall_high_spin_button.connect_value_changed(move |spin_button| {
            let value = spin_button.value() as f32;
            let mut radio_lock = radio_clone.lock().unwrap();
            radio_lock.band_info[band_index].waterfall_high = value;
        });

    }

    notebook.append_page(&display_grid, Some(&display_label));

    let rx_equalizer_label = Label::new(Some("RX Equalizer"));
    let rx_equalizer_grid = Grid::builder()
            .margin_start(0)
            .margin_end(0)
            .margin_top(0)
            .margin_bottom(0)
            .halign(Align::Center)
            .valign(Align::Center)
            .row_spacing(5)
            .column_spacing(5)
            .build();
    rx_equalizer_grid.set_column_homogeneous(true);
    rx_equalizer_grid.set_row_homogeneous(true);

    let mut r = radio.lock().unwrap();
    let enabled = r.receiver[0].equalizer_enabled;
    let preamp = r.receiver[0].equalizer_preamp as f64;
    let low = r.receiver[0].equalizer_low as f64;
    let mid = r.receiver[0].equalizer_mid as f64;
    let high = r.receiver[0].equalizer_high as f64;
    drop(r);

    let equalizer_enabled_check_button = CheckButton::with_label("Equalizer Enabled");
    equalizer_enabled_check_button.set_active(enabled);
    rx_equalizer_grid.attach(&equalizer_enabled_check_button, 0, 0, 2, 1);
    let enabled_radio = Arc::clone(&radio);
    equalizer_enabled_check_button.connect_toggled(move |button| {
        let mut r = enabled_radio.lock().unwrap();
        r.receiver[0].equalizer_enabled = button.is_active();
        r.receiver[0].enable_equalizer();
    });


    let preamp_label = Label::new(Some("Preamp"));
    rx_equalizer_grid.attach(&preamp_label, 0, 1, 1, 1);
    let low_label = Label::new(Some("Low"));
    rx_equalizer_grid.attach(&low_label, 1, 1, 1, 1);
    let mid_label = Label::new(Some("Mid"));
    rx_equalizer_grid.attach(&mid_label, 2, 1, 1, 1);
    let high_label = Label::new(Some("High"));
    rx_equalizer_grid.attach(&high_label, 3, 1, 1, 1);

    let preamp_adjustment = Adjustment::new( 
        preamp, // initial value
        -12.0,  // Minimum value
        15.0, // Maximum value
        1.0,  // Step increment
        1.0, // Page increment 
        0.0,  // Page size (not typically used for simple scales)
    );
    let preamp_scale = Scale::new(Orientation::Vertical, Some(&preamp_adjustment)); 
    preamp_scale.set_digits(0); // Display whole numbers
    preamp_scale.set_draw_value(true);
    preamp_scale.set_inverted(true);
    preamp_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    preamp_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    preamp_scale.add_mark(15.0, PositionType::Left, Some("15dB"));
    rx_equalizer_grid.attach(&preamp_scale, 0, 2, 1, 10);

    let low_adjustment = Adjustment::new(
        low, // initial value
        -12.0,  // Minimum value
        15.0, // Maximum value
        1.0,  // Step increment
        1.0, // Page increment
        0.0,  // Page size (not typically used for simple scales)
    );
    let low_scale = Scale::new(Orientation::Vertical, Some(&low_adjustment));
    low_scale.set_digits(0); // Display whole numbers
    low_scale.set_draw_value(true);
    low_scale.set_inverted(true);
    low_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    low_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    low_scale.add_mark(15.0, PositionType::Left, Some("15dB"));
    rx_equalizer_grid.attach(&low_scale, 1, 2, 1, 10);


    let mid_adjustment = Adjustment::new(
        mid, // initial value
        -12.0,  // Minimum value
        15.0, // Maximum value
        1.0,  // Step increment
        1.0, // Page increment
        0.0,  // Page size (not typically used for simple scales)
    );
    let mid_scale = Scale::new(Orientation::Vertical, Some(&mid_adjustment));
    mid_scale.set_digits(0); // Display whole numbers
    mid_scale.set_draw_value(true);
    mid_scale.set_inverted(true);
    mid_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    mid_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    mid_scale.add_mark(15.0, PositionType::Left, Some("15dB"));
    rx_equalizer_grid.attach(&mid_scale, 2, 2, 1, 10);

    let high_adjustment = Adjustment::new(
        high, // initial value
        -12.0,  // Minimum value
        15.0, // Maximum value
        1.0,  // Step increment
        1.0, // Page increment
        0.0,  // Page size (not typically used for simple scales)
    );
    let high_scale = Scale::new(Orientation::Vertical, Some(&high_adjustment));
    high_scale.set_digits(0); // Display whole numbers
    high_scale.set_draw_value(true);
    high_scale.set_inverted(true);
    high_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    high_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    high_scale.add_mark(15.0, PositionType::Left, Some("15dB"));
    rx_equalizer_grid.attach(&high_scale, 3, 2, 1, 10);

    let preamp_radio = Arc::clone(&radio);
    preamp_adjustment.connect_value_changed(move |adjustment| {
        let mut r = preamp_radio.lock().unwrap();
        r.receiver[0].equalizer_preamp = adjustment.value() as f32;
        r.receiver[0].set_equalizer_values();
    });
    let low_radio = Arc::clone(&radio);
    low_adjustment.connect_value_changed(move |adjustment| {
        let mut r = low_radio.lock().unwrap();
        r.receiver[0].equalizer_low = adjustment.value() as f32;
        r.receiver[0].set_equalizer_values();
    });
    let mid_radio = Arc::clone(&radio);
    mid_adjustment.connect_value_changed(move |adjustment| {
        let mut r = mid_radio.lock().unwrap();
        r.receiver[0].equalizer_mid = adjustment.value() as f32;
        r.receiver[0].set_equalizer_values();
    });
    let high_radio = Arc::clone(&radio);
    high_adjustment.connect_value_changed(move |adjustment| {
        let mut r = high_radio.lock().unwrap();
        r.receiver[0].equalizer_high = adjustment.value() as f32;
        r.receiver[0].set_equalizer_values();
    });

    notebook.append_page(&rx_equalizer_grid, Some(&rx_equalizer_label));

    let button_box = gtk::Box::new(Orientation::Horizontal, 5);
    button_box.set_halign(gtk::Align::End);

    let ok_button = Button::builder().label("Ok").build();

    button_box.append(&ok_button);

    let main_vbox = gtk::Box::new(Orientation::Vertical, 0);
    main_vbox.append(&notebook);
    main_vbox.append(&button_box);
    window.set_child(Some(&main_vbox));

    let window_for_ok = window.clone();
    let audio_clone = Rc::clone(&audio);
    let radio_clone = radio.clone();
    ok_button.connect_clicked(move |_| {
        let mut r = radio_clone.lock().unwrap();

        if r.audio.input_device != String::from(&audio_clone.borrow().input_device) {
            // input device changed
            if r.audio.local_input {
                //input was active
                r.audio.close_input();
            }
            r.audio.input_device = String::from(&audio_clone.borrow().input_device);
            r.audio.local_input = audio_clone.borrow().local_input;
            if r.audio.local_input {
                //input is active
                r.audio.open_input();
            }
        } else if r.audio.local_input != audio_clone.borrow().local_input {
            // device the same but state changed
            r.audio.local_input = audio_clone.borrow().local_input;
            if r.audio.local_input {
                r.audio.open_input();
            } else {
                r.audio.close_input();
            }
        }

        if r.audio.output_device != String::from(&audio_clone.borrow().output_device) {
            // input device changed
            if r.audio.local_output {
                //input was active
                r.audio.close_output();
            }
            r.audio.output_device = String::from(&audio_clone.borrow().output_device);
            r.audio.local_output = audio_clone.borrow().local_output;
            if r.audio.local_output {
                //input is active
                r.audio.open_output();
            }
        } else if r.audio.local_output != audio_clone.borrow().local_output {
            // device the same but state changed
            r.audio.local_output = audio_clone.borrow().local_output;
            if r.audio.local_output {
                r.audio.open_output();
            } else {
                r.audio.close_output();
            }
        }

        window_for_ok.close();
    });


    window
}

