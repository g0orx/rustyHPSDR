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

/*
    A Rust implemnetation of a Software Defined Radio to work with radios that use
    the OpenHPSDR protocols over Ethernet.
*/

use glib::{self, clone};
use glib::ControlFlow::Continue;
use glib::timeout_add_local;
use gtk::prelude::*;
use gtk::{Application, Builder, Label, Window};
use gtk::{EventController, EventControllerMotion, EventControllerScroll, EventControllerScrollFlags, GestureClick};
use gtk::gdk::Cursor;
use gtk::glib::Propagation;

use std::cell::{Cell, RefCell};
use std::env;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::{atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;


use rustyHPSDR::agc::*;
use rustyHPSDR::bands::*;
use rustyHPSDR::cat::{CatMessage, CAT};
use rustyHPSDR::midi::{MidiMessage, MIDI};
use rustyHPSDR::modes::*;
use rustyHPSDR::discovery::create_discovery_dialog;
use rustyHPSDR::discovery::device_name;
use rustyHPSDR::discovery::Boards;
use rustyHPSDR::radio::Radio;
use rustyHPSDR::radio::RadioMutex;
use rustyHPSDR::configure::*;
use rustyHPSDR::protocol1::Protocol1;
use rustyHPSDR::protocol2::Protocol2;
use rustyHPSDR::spectrum::*;
use rustyHPSDR::waterfall::*;
use rustyHPSDR::meter::*;
use rustyHPSDR::util::*;
use rustyHPSDR::wdsp::*;
use rustyHPSDR::notches::*;
use rustyHPSDR::widgets::*;

fn main() {
    let id = format!("org.g0orx.rustyHPSDR.pid{}", process::id());
    let application = Application::builder()
        .application_id(id)
        .build();
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(app: &Application) {

    // check wisdom file exists - if not create it
    let home_dir: PathBuf = match env::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: Could not determine home directory.");
            return;
        }
    };

    let my_dir = home_dir.join(".config").join("rustyHPSDR").join("");
    if !my_dir.is_dir() {
        match fs::create_dir_all(&my_dir) {
            Ok(_) => {
            }
            Err(e) => {
                let error_message = format!("Failed to create directory {:?}: {}", my_dir, e);
                eprintln!("{}", error_message);
            }
        }
    }

    let os_string = my_dir.clone().into_os_string();
    let c_string = match CString::new(os_string.into_vec()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error converting path to C string (contains null bytes?): {}", e);
            return;
        }
    };

    let wisdom_completed = Arc::new(AtomicBool::new(false));

    let wisdom_completed_clone = wisdom_completed.clone();
    thread::spawn(move || {
        let c_path_ptr: *const c_char = c_string.as_ptr();
        unsafe {
            WDSPwisdom(c_path_ptr);
        }
        wisdom_completed_clone.store(true, Ordering::SeqCst);
    });

    let wisdom_xml = include_str!("./ui/wisdom.xml");
    let wisdom_builder = Builder::from_string(wisdom_xml);

    let wisdom_window: Window = wisdom_builder
            .object("wisdom_window")
            .expect("Could not get object `wisdom_window` from builder.");

    wisdom_window.set_modal(true);

    wisdom_window.present();

    let wisdom_label: Label = wisdom_builder
            .object("wisdom_label")
            .expect("Could not get object `wisdom_label` from builder.");

    while !wisdom_completed.load(Ordering::Relaxed) {
        unsafe {
            let c_ptr: *mut c_char = unsafe {
                wisdom_get_status()
            };
            if !c_ptr.is_null() {
                let c_str: &CStr = unsafe {
                    CStr::from_ptr(c_ptr)
                };
                let rust_string: String = c_str.to_string_lossy().into_owned();
                wisdom_label.set_text(&rust_string);
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
    eprintln!("WDSPwisdom completed");
    wisdom_window.close();

    let ui_css = include_str!("ui/ui.css");
    let provider = gtk::CssProvider::new();
    provider.load_from_data(ui_css);
    gtk::StyleContext::add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let ui_xml = include_str!("ui/ui.xml");
    let builder = Builder::from_string(ui_xml);

    let app_widgets = AppWidgets::from_builder(&builder);
    let rc_app_widgets = Rc::new(RefCell::new(app_widgets));

    let rc_app_widgets_clone = rc_app_widgets.clone();
    let app_widgets = rc_app_widgets_clone.borrow();
    app_widgets.main_window.set_application(Some(app));

    let spectrum = Spectrum::new(0,1024,168);
    let rc_spectrum = Rc::new(RefCell::new(spectrum));
    let waterfall = Waterfall::new(0,1024,168);
    let rc_waterfall = Rc::new(RefCell::new(waterfall));
    let spectrum_2 = Spectrum::new(1,1024,168);
    let rc_spectrum_2 = Rc::new(RefCell::new(spectrum_2));
    let waterfall_2 = Waterfall::new(1,1024,168);
    let rc_waterfall_2 = Rc::new(RefCell::new(waterfall_2));
    let meter_1 = Meter::new(256,36);
    let rc_meter_1 = Rc::new(RefCell::new(meter_1));
    let meter_2 = Meter::new(256,36);
    let rc_meter_2 = Rc::new(RefCell::new(meter_2));
    let meter_tx = Meter::new(256,36);
    let rc_meter_tx = Rc::new(RefCell::new(meter_tx));

    let discovery_data = Rc::new(RefCell::new(Vec::new()));
    let selected_index: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
    let selected_index_for_discovery_dialog = selected_index.clone();
    let discovery_data_clone = Rc::clone(&discovery_data);


    let rc_app_widgets_clone = rc_app_widgets.clone();
    let app_widgets = rc_app_widgets_clone.borrow();
    let discovery_dialog = create_discovery_dialog(&app_widgets.main_window.clone(), discovery_data_clone, selected_index_for_discovery_dialog);


    let selected_index_clone = selected_index.clone();
    let discovery_data_clone = Rc::clone(&discovery_data);
    let app_clone = app.clone();
    let builder_clone = builder.clone();
    let rc_spectrum_clone = rc_spectrum.clone();
    let rc_waterfall_clone = rc_waterfall.clone();
    let rc_spectrum_2_clone = rc_spectrum_2.clone();
    let rc_waterfall_2_clone = rc_waterfall_2.clone();
    let rc_meter_1_clone = rc_meter_1.clone();
    let rc_meter_2_clone = rc_meter_2.clone();
    let rc_meter_tx_clone = rc_meter_tx.clone();
    let rc_app_widgets_clone = rc_app_widgets.clone();
    discovery_dialog.connect_close_request(move |_| {
        let mut app_widgets = rc_app_widgets_clone.borrow_mut();

        let index = *selected_index_clone.borrow();
        match index {
            Some(i) => {
                if i >= 0 {
                    let device = discovery_data_clone.borrow()[(i-1) as usize];

                    let radio_mutex = RadioMutex::new(Arc::new(Mutex::new(Radio::load(device, app_widgets.spectrum_display.width()))));

                    {
                    let r = radio_mutex.radio.lock().unwrap();
                    let title = format!("rustyHPSDR: {:?} ({}) {:?} Protocol {}", r.model, device_name(device), device.address.ip(), device.protocol);
                    app_widgets.main_window.set_title(Some(&title));
                    }

                    if device.board == Boards::HermesLite || device.board == Boards::HermesLite2 {
                        app_widgets.band_6_button.set_label("");
                        app_widgets.band_6_button.set_sensitive(false);
                    } 

                    let rc_spectrum_clone2 = rc_spectrum_clone.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.spectrum_display.connect_resize(move |_, width, height| { 
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let channel = r.receiver[0].channel;
                        r.receiver[0].init_analyzer(channel, width);
                        r.transmitter.init_analyzer(width);
                        let mut spectrum = rc_spectrum_clone2.borrow_mut();
                        spectrum.resize(width, height);
                    });

                    let rc_waterfall_clone2 = rc_waterfall_clone.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.waterfall_display.connect_resize(move |_, width, height| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.receiver[0].waterfall_width = width;
                        let mut waterfall = rc_waterfall_clone2.borrow_mut();
                        waterfall.resize(width, height);
                    });

                    let rc_spectrum_2_clone2 = rc_spectrum_2_clone.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.spectrum_2_display.connect_resize(move |_, width, height| { 
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let channel = r.receiver[1].channel;
                        r.receiver[1].init_analyzer(channel, width);
                        let mut spectrum = rc_spectrum_2_clone2.borrow_mut();
                        spectrum.resize(width, height);
                    });

                    let rc_waterfall_2_clone2 = rc_waterfall_2_clone.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.waterfall_2_display.connect_resize(move |_, width, height| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.receiver[1].waterfall_width = width;
                        let mut waterfall = rc_waterfall_2_clone2.borrow_mut();
                        waterfall.resize(width, height);
                    });

                    // setup the ui state
                    {
                        let mut r = radio_mutex.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        if rx==0 {
                            app_widgets.band_frame.set_label(Some("RX1 Band"));
                            app_widgets.mode_frame.set_label(Some("RX1 Mode"));
                            app_widgets.filter_frame.set_label(Some("RX1 Filter"));
                        } else {
                            app_widgets.band_frame.set_label(Some("RX2 Band"));
                            app_widgets.mode_frame.set_label(Some("RX2 Mode"));
                            app_widgets.filter_frame.set_label(Some("RX2 Filter"));
                        }

                        if r.receiver[0].ctun {
                            let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                        } else {
                            let formatted_value = format_u32_with_separators(r.receiver[0].frequency as u32);
                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                        }

                        if r.receiver[1].ctun {
                            let formatted_value = format_u32_with_separators(r.receiver[1].ctun_frequency as u32);
                            app_widgets.vfo_b_frequency.set_label(&formatted_value);
                        } else {
                            let formatted_value = format_u32_with_separators(r.receiver[1].frequency as u32);
                            app_widgets.vfo_b_frequency.set_label(&formatted_value);
                        }

                        let formatted_value = format_u32_with_separators(r.receiver[1].frequency as u32);
                        app_widgets.vfo_b_frequency.set_label(&formatted_value);

                        let style_context = app_widgets.a_to_b_button.style_context();
                        style_context.add_class("basic-button");
                        let style_context = app_widgets.b_to_a_button.style_context();
                        style_context.add_class("basic-button");
                        let style_context = app_widgets.a_swap_b_button.style_context();
                        style_context.add_class("basic-button");

                        let style_context = app_widgets.ctun_button.style_context();
                        style_context.add_class("toggle");
                        app_widgets.ctun_button.set_active(r.receiver[rx].ctun);

                        let style_context = app_widgets.cat_button.style_context();
                        style_context.add_class("toggle");
                        app_widgets.cat_button.set_active(r.cat_enabled);

                        let style_context = app_widgets.midi_button.style_context();
                        style_context.add_class("toggle");
                        app_widgets.midi_button.set_active(r.midi_enabled);

                        let style_context = app_widgets.split_button.style_context();
                        style_context.add_class("toggle");
                        app_widgets.split_button.set_active(r.split);

                        let style_context = app_widgets.rx2_button.style_context();
                        style_context.add_class("toggle");
                        app_widgets.rx2_button.set_active(r.rx2_enabled);
                        if r.rx2_enabled {
                            app_widgets.spectrum_2_display.set_visible(true);
                            app_widgets.waterfall_2_display.set_visible(true);
                            app_widgets.meter_2_display.set_visible(true);
                        } else {
                            app_widgets.spectrum_2_display.set_visible(false);
                            app_widgets.waterfall_2_display.set_visible(false);
                            app_widgets.meter_2_display.set_visible(false);
                        }

                        app_widgets.afgain_adjustment.set_value((r.receiver[rx].afgain * 100.0).into());
                        app_widgets.agc_dropdown.set_selected(r.receiver[rx].agc as u32);
                        app_widgets.agcgain_adjustment.set_value(r.receiver[rx].agcgain.into());
                        if r.dev == 6 { // HEMES_LITE
                            app_widgets.attenuation_adjustment.set_lower(-12.0);
                            app_widgets.attenuation_adjustment.set_upper(48.0);
                            let b = r.receiver[0].band.to_usize();
                            app_widgets.attenuation_adjustment.set_value(r.receiver[0].band_info[b].attenuation.into());
                        } else {
                            let b = r.receiver[rx].band.to_usize();
                            app_widgets.attenuation_adjustment.set_value(r.receiver[rx].band_info[b].attenuation.into());
                        }

                        let mut sq = r.receiver[rx].am_squelch_threshold;
                        if r.receiver[rx].mode == Modes::FMN.to_usize() {
                            sq = r.receiver[rx].fm_squelch_threshold;
                        }
                        app_widgets.squelch_adjustment.set_value(sq);

                        app_widgets.micgain_adjustment.set_value(r.transmitter.micgain.into());
                        app_widgets.drive_adjustment.set_value(r.transmitter.drive.into());
                        app_widgets.cwpitch_adjustment.set_value(r.receiver[rx].cw_pitch.into());

                        let rc_spectrum_clone2 = rc_spectrum_clone.clone();
                        let mut spectrum = rc_spectrum_clone2.borrow_mut();
                        spectrum.resize(app_widgets.spectrum_display.width(), app_widgets.spectrum_display.height());
                        r.receiver[0].spectrum_width = app_widgets.spectrum_display.width();
                        r.receiver[0].init();
                        let channel = r.receiver[0].channel;
                        let width = r.receiver[0].spectrum_width;
                        r.receiver[0].init_analyzer(channel, width);

                        let rc_spectrum_2_clone2 = rc_spectrum_2_clone.clone();
                        let mut spectrum = rc_spectrum_2_clone2.borrow_mut();
                        spectrum.resize(app_widgets.spectrum_2_display.width(), app_widgets.spectrum_2_display.height());
                        r.receiver[1].spectrum_width = app_widgets.spectrum_2_display.width();
                        r.receiver[1].init();
                        let channel = r.receiver[1].channel;
                        let width = r.receiver[1].spectrum_width;
                        r.receiver[1].init_analyzer(channel, width);

                        let rc_waterfall_clone2 = rc_waterfall_clone.clone();
                        let mut waterfall = rc_waterfall_clone2.borrow_mut();
                        waterfall.resize(app_widgets.waterfall_display.width(), app_widgets.waterfall_display.height());
                        r.receiver[0].spectrum_width = app_widgets.spectrum_display.width();

                        let rc_waterfall_2_clone2 = rc_waterfall_2_clone.clone();
                        let mut waterfall = rc_waterfall_2_clone2.borrow_mut();
                        waterfall.resize(app_widgets.waterfall_2_display.width(), app_widgets.waterfall_2_display.height());
                        r.receiver[1].spectrum_width = app_widgets.spectrum_2_display.width();

                        app_widgets.step_dropdown.set_selected(r.receiver[rx].step_index as u32);
                        app_widgets.zoom_adjustment.set_value(r.receiver[rx].zoom.into());
                        app_widgets.pan_adjustment.set_value(r.receiver[rx].pan.into());

                        let style_context = app_widgets.nr_button.style_context();
                        style_context.add_class("toggle");
                        app_widgets.nr_button.set_active(r.receiver[rx].nr | r.receiver[rx].nr2);
                        if r.receiver[rx].nr {
                            r.receiver[rx].set_nr();
                            app_widgets.nr_button.set_label("NR");
                        }
                        if r.receiver[rx].nr2 {
                            r.receiver[rx].set_nr2();
                            app_widgets.nr_button.set_label("NR2");
                        }
                        if r.receiver[rx].nr3 {
                            r.receiver[rx].set_nr3();
                            app_widgets.nr_button.set_label("NR3");
                        }
                        if r.receiver[rx].nr4 {
                            r.receiver[rx].set_nr4();
                            app_widgets.nr_button.set_label("NR4");
                        }
                        
                        let style_context = app_widgets.nb_button.style_context();
                        style_context.add_class("toggle");
                        app_widgets.nb_button.set_active(r.receiver[rx].nb | r.receiver[rx].nb2);
                        if r.receiver[rx].nb {
                            r.receiver[rx].set_nb();
                            app_widgets.nb_button.set_label("NB");
                        }
                        if r.receiver[rx].nb2 {
                            r.receiver[rx].set_nb2();
                            app_widgets.nb_button.set_label("NB2");
                        }

                        let style_context = app_widgets.anf_button.style_context();
                        style_context.add_class("toggle");
                        app_widgets.anf_button.set_active(r.receiver[rx].anf);
                        r.receiver[rx].set_anf();

                        let style_context = app_widgets.snb_button.style_context();
                        style_context.add_class("toggle");
                        app_widgets.snb_button.set_active(r.receiver[rx].snb);
                        r.receiver[rx].set_snb();

                        let style_context = app_widgets.mox_button.style_context();
                        style_context.add_class("toggle");

                        let style_context = app_widgets.tun_button.style_context();
                        style_context.add_class("toggle");
                    }

                    // handle ui events
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.configure_button.connect_clicked(move |_| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let configure_dialog = create_configure_dialog(&rc_app_widgets_clone_clone.clone(), &radio_mutex_clone);
                        app_widgets.configure_button.set_sensitive(false);
                        configure_dialog.present();
                        let rc_app_widgets = rc_app_widgets_clone_clone.clone();
                        configure_dialog.connect_close_request(move |_| {
                            let app_widgets = rc_app_widgets.borrow();
                            app_widgets.configure_button.set_sensitive(true);
                            Propagation::Proceed
                        });
                    });                         

                    let scroll_controller_a = EventControllerScroll::new(
                        EventControllerScrollFlags::VERTICAL
                    );
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    scroll_controller_a.connect_scroll(move |controller, dx, dy| {
                        spectrum_waterfall_scroll(&radio_mutex_clone, &rc_app_widgets_clone_clone, 0, dy);
                        Propagation::Proceed
                    });
                    app_widgets.vfo_a_frequency.add_controller(scroll_controller_a);

                    let scroll_controller_b = EventControllerScroll::new(
                        EventControllerScrollFlags::VERTICAL
                    );
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    scroll_controller_b.connect_scroll(move |_controller, _dx, dy| {
                        spectrum_waterfall_scroll(&radio_mutex_clone, &rc_app_widgets_clone_clone, 1, dy);
                        Propagation::Proceed
                    });
                    app_widgets.vfo_b_frequency.add_controller(scroll_controller_b);


                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.a_to_b_button.connect_clicked(move |_| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        if r.receiver[0].ctun {
                            r.receiver[1].frequency = r.receiver[0].ctun_frequency; 
                        } else {
                            r.receiver[1].frequency = r.receiver[0].frequency; 
                        }
                        r.receiver[1].band = r.receiver[0].band; 
                        let formatted_value = format_u32_with_separators(r.receiver[1].frequency as u32);
                        app_widgets.vfo_b_frequency.set_label(&formatted_value);
                        unsafe {
                            RXANBPSetTuneFrequency(1, r.receiver[1].frequency as f64);
                        }
                    });                         
                    
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.b_to_a_button.connect_clicked(move |_| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        if r.receiver[1].ctun {
                            r.receiver[0].ctun_frequency = r.receiver[1].frequency;
                            r.receiver[0].set_ctun_frequency();
                        } else {
                            r.receiver[0].frequency = r.receiver[1].frequency;
                        }
                        r.receiver[0].band = r.receiver[1].band; 
                        let formatted_value = format_u32_with_separators(r.receiver[0].frequency as u32);
                        app_widgets.vfo_a_frequency.set_label(&formatted_value);
                        unsafe {
                            RXANBPSetTuneFrequency(0, r.receiver[0].frequency as f64);
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.a_swap_b_button.connect_clicked(move |_| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let temp_frequency = r.receiver[1].frequency;
                        let temp_band = r.receiver[1].band;
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        if r.receiver[0].ctun {
                            r.receiver[1].frequency = r.receiver[0].ctun_frequency;
                            r.receiver[0].ctun_frequency = temp_frequency;
                            let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                            r.receiver[0].set_ctun_frequency();
                        } else {
                            r.receiver[1].frequency = r.receiver[0].frequency;
                            r.receiver[0].frequency = temp_frequency;
                            let formatted_value = format_u32_with_separators(r.receiver[0].frequency as u32);
                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                        }
                        r.receiver[1].band = r.receiver[0].band;
                        r.receiver[0].band = temp_band;
                        let formatted_value = format_u32_with_separators(r.receiver[1].frequency as u32);
                        app_widgets.vfo_b_frequency.set_label(&formatted_value);
                        unsafe {
                            RXANBPSetTuneFrequency(0, r.receiver[0].frequency as f64);
                            RXANBPSetTuneFrequency(1, r.receiver[1].frequency as f64);
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.ctun_button.connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let style_context = button.style_context();
                        r.receiver[rx].ctun = button.is_active();
                        if r.receiver[rx].ctun {
                            r.receiver[rx].ctun_frequency = r.receiver[rx].frequency;
                            r.receiver[rx].set_ctun(true);
                        } else {
                            r.receiver[rx].ctun_frequency = 0.0;
                            r.receiver[rx].set_ctun(false);
                            let formatted_value = format_u32_with_separators(r.receiver[rx].frequency as u32);
                            if rx == 0 {
                                app_widgets.vfo_a_frequency.set_label(&formatted_value);
                            } else {
                                app_widgets.vfo_b_frequency.set_label(&formatted_value);
                            }
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.split_button.connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.split = button.is_active();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.rx2_button.connect_clicked(move |button| {
                        let mut update = false;
                        {
                            let mut r = radio_mutex_clone.radio.lock().unwrap();
                            let app_widgets = rc_app_widgets_clone_clone.borrow();
                            r.rx2_enabled = button.is_active();
                            if r.rx2_enabled {
                                app_widgets.spectrum_2_display.set_visible(true);
                                app_widgets.waterfall_2_display.set_visible(true);
                                app_widgets.meter_2_display.set_visible(true);
                            } else {
                                app_widgets.spectrum_2_display.set_visible(false);
                                app_widgets.waterfall_2_display.set_visible(false);
                                app_widgets.meter_2_display.set_visible(false);
                            }
                            if r.receiver[1].active {
                                r.receiver[1].active = false;
                                r.receiver[0].active = true;
                                update = true;
                            }
                        }
                        if update {
                            update_ui(&radio_mutex_clone.clone(), &rc_app_widgets_clone_clone.clone());
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.step_dropdown.connect_selected_notify(move |step_dropdown| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let index = step_dropdown.selected();
                        let mut step = 1000.0;
                        match index {
                            0 => step = 1.0,
                            1 => step = 10.0,
                            2 => step = 25.0,
                            3 => step = 50.0,
                            4 => step = 100.0,
                            5 => step = 250.0,
                            6 => step = 500.0,
                            7 => step = 1000.0,
                            8 => step = 5000.0,
                            9 => step = 9000.0,
                            10 => step = 10000.0,
                            11 => step = 100000.0,
                            12 => step = 250000.0,
                            13 => step = 500000.0,
                            14 => step = 1000000.0,
                            _ => step = 1000.0,
                        }
                        if r.receiver[0].active {
                            r.receiver[0].step_index = index as usize;
                            r.receiver[0].step = step;
                        } else {
                            r.receiver[1].step_index = index as usize;
                            r.receiver[1].step = step;
                        }
                    });

                    let middle_button_pressed = Rc::new(RefCell::new(false));
                    let spectrum_click_gesture = Rc::new(GestureClick::new());
                    spectrum_click_gesture.set_button(0); // all buttons
                    let spectrum_click_gesture_clone = spectrum_click_gesture.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    let press_state = middle_button_pressed.clone();
                    spectrum_click_gesture_clone.connect_pressed(move |gesture, controller, x, _y| {
                        let da = gesture.widget().unwrap();
                        let width = da.allocated_width();
                        if gesture.current_button() == 2 { // middle button
                            *press_state.borrow_mut() = true;
                        } else if gesture.current_button() == 1 { // left button
                            if !spectrum_waterfall_clicked(&radio_mutex_clone, &rc_app_widgets_clone_clone, 0, x, width, gesture.current_button()) {
                                update_ui(&radio_mutex_clone.clone(), &rc_app_widgets_clone_clone.clone());
                            }
                        } else if gesture.current_button() == 3 { // right button
                            // add a notch?
                            let mut r = radio_mutex_clone.radio.lock().unwrap();
                            let rx = if r.receiver[0].active { 0 } else { 1 };
                            let notch = Notch::new(rx as i32, r.receiver[rx].frequency as f64, 500.0, 1);
                            r.add_notch_to_vector(notch);
                            r.add_notch(notch);
                        }
                    });
                    let press_state = middle_button_pressed.clone();
                    spectrum_click_gesture_clone.connect_released(move |gesture, controller, x, _y| {
                        if gesture.current_button() == 2 { // middle button
                            *press_state.borrow_mut() = false;
                        }
                    });
                    app_widgets.spectrum_display.add_controller(<GestureClick as Clone>::clone(&spectrum_click_gesture).upcast::<EventController>());

                    let middle_button_pressed = Rc::new(RefCell::new(false));
                    let spectrum_2_click_gesture = Rc::new(GestureClick::new());
                    spectrum_2_click_gesture.set_button(0); // all buttons
                    let spectrum_2_click_gesture_clone = spectrum_2_click_gesture.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    let press_state = middle_button_pressed.clone();
                    spectrum_2_click_gesture_clone.connect_pressed(move |gesture, controller, x, _y| {
                        let da = gesture.widget().unwrap();
                        let width = da.allocated_width();
                        if gesture.current_button() == 2 { // middle button
                            *press_state.borrow_mut() = true;
                        } else if !spectrum_waterfall_clicked(&radio_mutex_clone, &rc_app_widgets_clone_clone, 1, x, width, gesture.current_button()) {
                            update_ui(&radio_mutex_clone.clone(), &rc_app_widgets_clone_clone.clone());
                        }
                    });
                    let press_state = middle_button_pressed.clone();
                    spectrum_2_click_gesture_clone.connect_released(move |gesture, controller, x, _y| {
                        if gesture.current_button() == 2 { // middle button
                            *press_state.borrow_mut() = false;
                        }
                    });
                    app_widgets.spectrum_2_display.add_controller(<GestureClick as Clone>::clone(&spectrum_2_click_gesture).upcast::<EventController>());


                    let last_spectrum_x = Rc::new(Cell::new(0.0));
                    let last_spectrum_y = Rc::new(Cell::new(0.0));

                    let cursor_nsresize = Cursor::from_name("ns-resize", None);
                    let cursor_nrsize = Cursor::from_name("n-resize", None);
                    let cursor_sresize = Cursor::from_name("s-resize", None);

                    let motion_event_controller_spectrum = EventControllerMotion::new();
                    app_widgets.spectrum_display.add_controller(motion_event_controller_spectrum.clone());
                    let last_spectrum_x_clone = last_spectrum_x.clone();
                    let last_spectrum_y_clone = last_spectrum_y.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    motion_event_controller_spectrum.connect_motion(move |controller, x, y| {
                        last_spectrum_x_clone.set(x);
                        last_spectrum_y_clone.set(y);
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        if x < 80.0 {
                            let height = app_widgets.spectrum_display.height();
                            let top = height / 4;
                            let bottom = height - top;
                            if y < top.into() {
                                app_widgets.spectrum_display.set_cursor(cursor_nrsize.as_ref());
                            } else if y > bottom.into() {
                                app_widgets.spectrum_display.set_cursor(cursor_sresize.as_ref());
                            } else {
                                app_widgets.spectrum_display.set_cursor(cursor_nsresize.as_ref());
                            }
                        } else {
                            app_widgets.spectrum_display.set_cursor(None); // default
                        }
                    });

                    //let last_spectrum_x = Rc::new(Cell::new(0.0));
                    //let last_spectrum_y = Rc::new(Cell::new(0.0));

                    let cursor_nsresize = Cursor::from_name("ns-resize", None);
                    let cursor_nrsize = Cursor::from_name("n-resize", None);
                    let cursor_sresize = Cursor::from_name("s-resize", None);

                    let motion_event_controller_spectrum_2 = EventControllerMotion::new();
                    app_widgets.spectrum_2_display.add_controller(motion_event_controller_spectrum_2.clone());
                    let last_spectrum_x_clone = last_spectrum_x.clone();
                    let last_spectrum_y_clone = last_spectrum_y.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    motion_event_controller_spectrum_2.connect_motion(move |controller, x, y| {
                        last_spectrum_x_clone.set(x);
                        last_spectrum_y_clone.set(y);
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        if x < 80.0 {
                            let height = app_widgets.spectrum_2_display.height();
                            let top = height / 4;
                            let bottom = height - top;
                            if y < top.into() {
                                app_widgets.spectrum_2_display.set_cursor(cursor_nrsize.as_ref());
                            } else if y > bottom.into() {
                                app_widgets.spectrum_2_display.set_cursor(cursor_sresize.as_ref());
                            } else {
                                app_widgets.spectrum_2_display.set_cursor(cursor_nsresize.as_ref());
                            }
                        } else {
                            app_widgets.spectrum_2_display.set_cursor(None); // default
                        }
                    });

                    let scroll_controller_spectrum = EventControllerScroll::new(
                        EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::KINETIC
                    );
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    let last_spectrum_x_clone = last_spectrum_x.clone();
                    let last_spectrum_y_clone = last_spectrum_y.clone();
                    let middle_button_state = middle_button_pressed.clone();
                    scroll_controller_spectrum.connect_scroll(move |controller, _dx, dy| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let mut increment = 1.0;
                        if dy > 0.0 {
                            increment = -1.0;
                        }
                        let height = app_widgets.spectrum_display.height();
                        let top = height / 4;
                        let bottom = height - top;

                        if last_spectrum_x_clone.get() < 80.0 {
                            let mut r = radio_mutex_clone.radio.lock().unwrap();
                            if r.is_transmitting() {
                                if last_spectrum_y_clone.get() < top.into() {
                                    r.transmitter.spectrum_high += increment;
                                } else if last_spectrum_y_clone.get() > bottom.into() {
                                    r.transmitter.spectrum_low += increment;
                                } else {
                                    r.transmitter.spectrum_high += increment;
                                    r.transmitter.spectrum_low += increment;
                                }
                            } else {
                                let b = r.receiver[0].band.to_usize();
                                if last_spectrum_y_clone.get() < top.into() {
                                    r.receiver[0].band_info[b].spectrum_high += increment;
                                } else if last_spectrum_y_clone.get() > bottom.into() {
                                    r.receiver[0].band_info[b].spectrum_low += increment;
                                } else {
                                    r.receiver[0].band_info[b].spectrum_low += increment;
                                    r.receiver[0].band_info[b].spectrum_high += increment;
                                }
                            }
                        } else {
                            spectrum_waterfall_scroll(&radio_mutex_clone, &rc_app_widgets_clone_clone, 0, dy);
                        }
                        Propagation::Proceed
                    });
                    app_widgets.spectrum_display.add_controller(scroll_controller_spectrum.clone());

                    let scroll_controller_spectrum_2 = EventControllerScroll::new(
                        EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::KINETIC
                    );
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    let last_spectrum_x_clone = last_spectrum_x.clone();
                    let last_spectrum_y_clone = last_spectrum_y.clone();
                    let middle_button_state = middle_button_pressed.clone();
                    scroll_controller_spectrum_2.connect_scroll(move |controller, _dx, dy| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let mut increment = 1.0;
                        if dy > 0.0 {
                            increment = -1.0;
                        }
                        let height = app_widgets.spectrum_2_display.height();
                        let top = height / 4;
                        let bottom = height - top;

                        if last_spectrum_x_clone.get() < 80.0 {
                            let mut r = radio_mutex_clone.radio.lock().unwrap();
                            if r.is_transmitting() {
                                if last_spectrum_y_clone.get() < top.into() {
                                    r.transmitter.spectrum_high += increment;
                                } else if last_spectrum_y_clone.get() > bottom.into() {
                                    r.transmitter.spectrum_low += increment;
                                } else {
                                    r.transmitter.spectrum_high += increment;
                                    r.transmitter.spectrum_low += increment;
                                }
                            } else {
                                let b = r.receiver[1].band.to_usize();
                                if last_spectrum_y_clone.get() < top.into() {
                                    r.receiver[1].band_info[b].spectrum_high += increment;
                                } else if last_spectrum_y_clone.get() > bottom.into() {
                                    r.receiver[1].band_info[b].spectrum_low += increment;
                                } else {
                                    r.receiver[1].band_info[b].spectrum_low += increment;
                                    r.receiver[1].band_info[b].spectrum_high += increment;
                                }
                            }
                        } else {
                            spectrum_waterfall_scroll(&radio_mutex_clone, &rc_app_widgets_clone_clone, 1, dy);
                        }
                        Propagation::Proceed
                    });
                    app_widgets.spectrum_2_display.add_controller(scroll_controller_spectrum_2.clone());


                    let scroll_controller_waterfall = EventControllerScroll::new(
                        EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::KINETIC
                    );
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    scroll_controller_waterfall.connect_scroll(move |controller, _dx, dy| {
                        spectrum_waterfall_scroll(&radio_mutex_clone, &rc_app_widgets_clone_clone, 0, dy);
                        Propagation::Proceed
                    });
                    app_widgets.waterfall_display.add_controller(scroll_controller_waterfall.clone());

                    let scroll_controller_waterfall_2 = EventControllerScroll::new(
                        EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::KINETIC
                    );
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    scroll_controller_waterfall_2.connect_scroll(move |controller, _dx, dy| {
                        spectrum_waterfall_scroll(&radio_mutex_clone, &rc_app_widgets_clone_clone, 1, dy);
                        Propagation::Proceed
                    });
                    app_widgets.waterfall_2_display.add_controller(scroll_controller_waterfall_2.clone());

                    let waterfall_click_gesture = Rc::new(GestureClick::new());
                    waterfall_click_gesture.set_button(0); // all buttons
                    let waterfall_click_gesture_clone = waterfall_click_gesture.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    waterfall_click_gesture_clone.connect_pressed(move |gesture, _, x, _y| {
                        let da = gesture.widget().unwrap();
                        let width = da.allocated_width();
                        if !spectrum_waterfall_clicked(&radio_mutex_clone, &rc_app_widgets_clone_clone, 0, x, width, gesture.current_button()) {
                            update_ui(&radio_mutex_clone, &rc_app_widgets_clone_clone);
                        }
                    });
                    app_widgets.waterfall_display.add_controller(<GestureClick as Clone>::clone(&waterfall_click_gesture).upcast::<EventController>());

                    let waterfall_2_click_gesture = Rc::new(GestureClick::new());
                    waterfall_2_click_gesture.set_button(0); // all buttons
                    let waterfall_2_click_gesture_clone = waterfall_2_click_gesture.clone();
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    waterfall_2_click_gesture_clone.connect_pressed(move |gesture, _, x, _y| {
                        let da = gesture.widget().unwrap();
                        let width = da.allocated_width();
                        if !spectrum_waterfall_clicked(&radio_mutex_clone, &rc_app_widgets_clone_clone, 1, x, width, gesture.current_button()) {
                            update_ui(&radio_mutex_clone, &rc_app_widgets_clone_clone);
                        }
                    });
                    app_widgets.waterfall_2_display.add_controller(<GestureClick as Clone>::clone(&waterfall_2_click_gesture).upcast::<EventController>());


                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.zoom_adjustment.connect_value_changed(move |adjustment| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let mut p = 0.0;
                        {
                            let mut r = radio_mutex_clone.radio.lock().unwrap();
                            let rx = if r.receiver[0].active { 0 } else { 1 };
                            r.receiver[rx].zoom = adjustment.value() as i32;
                            let channel = r.receiver[rx].channel;
                            let width = r.receiver[rx].spectrum_width;
                            r.receiver[rx].init_analyzer(channel, width);
                            if adjustment.value() == 1.0 {
                                r.receiver[rx].pan = p as i32;
                            } else {
                                // try to keep the current frequency in the zoomed area
                                let frequency_low = r.receiver[rx].frequency - (r.receiver[rx].sample_rate/2) as f64;
                                let frequency_high = r.receiver[rx].frequency + (r.receiver[rx].sample_rate/2) as f64;
                                let frequency_range = frequency_high - frequency_low;
                                let width = r.receiver[rx].spectrum_width * r.receiver[rx].zoom;
                                let mut f = r.receiver[rx].frequency;
                                let hz_per_pixel = frequency_range / width as f64;
                                if r.receiver[rx].ctun {
                                    f = r.receiver[rx].ctun_frequency;
                                }
                                p = (f - frequency_low) / hz_per_pixel;
                                p = (p / width as f64) * 100.0;
                                r.receiver[rx].pan = p as i32;
                            }
                        }
                        app_widgets.pan_adjustment.set_value(p as f64);
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.pan_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        if r.receiver[rx].zoom > 1 {
                            r.receiver[rx].pan = adjustment.value() as i32;
                        } else {
                            r.receiver[rx].pan = 0;
                            adjustment.set_value(r.receiver[rx].pan.into());
                        }
                    });

                    let r = radio_mutex.radio.lock().unwrap();
                    let rx = if r.receiver[0].active { 0 } else { 1 };
                    let band = r.receiver[rx].band.to_usize();
                    let mode = r.receiver[rx].mode;
                    let filter = r.receiver[rx].filter;
                    let low = r.receiver[rx].filter_low;
                    let high = r.receiver[rx].filter_high;
                    drop(r);

                    app_widgets.filter_grid.set_active_values(low, high);

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.band_grid.set_callback(move|index| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        let b = r.receiver[rx].band.to_usize();
                        if b != index { // band has changed
                            // save current band info
                            r.receiver[rx].band_info[b].current = r.receiver[rx].frequency;
                            r.receiver[rx].band_info[b].ctun = r.receiver[rx].ctun_frequency;

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
                        app_widgets.filter_grid.update_filter_buttons(r.receiver[rx].band_info[index].mode.to_usize());
                        app_widgets.filter_grid.set_active_index(r.receiver[rx].band_info[index].filter.to_usize());

                        if b != index { // band has changed
                            r.receiver[rx].mode = r.receiver[rx].band_info[index].mode.to_usize();
                            app_widgets.mode_grid.set_active_index(r.receiver[rx].mode);
                            let (mut low, mut high) = app_widgets.filter_grid.get_filter_values(r.receiver[rx].band_info[index].mode.to_usize(), r.receiver[rx].band_info[index].filter.to_usize());
                            app_widgets.filter_grid.set_active_values(low, high);
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
                            r = radio_mutex_clone.radio.lock().unwrap();
                        }
                        unsafe {
                            RXANBPSetTuneFrequency(rx as i32, r.receiver[rx].frequency as f64);
                        } 
                    }, band);


                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.mode_grid.set_callback(move|index| {
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        r.receiver[rx].mode = index; 
                        app_widgets.filter_grid.update_filter_buttons(index);

                        let (mut low, mut high) = app_widgets.filter_grid.get_filter_values(index, r.receiver[rx].filter);
                        app_widgets.filter_grid.set_active_values(low, high);
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
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.filter_grid.set_callback(move|index| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        r.receiver[rx].filter = index;
                        let (mut low, mut high) = app_widgets.filter_grid.get_filter_values(r.receiver[rx].mode, r.receiver[rx].filter);
                        if r.receiver[rx].mode == Modes::CWL.to_usize() {
                            low += -r.receiver[rx].cw_pitch;
                            high += -r.receiver[rx].cw_pitch;
                        } else if r.receiver[rx].mode == Modes::CWU.to_usize() {
                            low += r.receiver[rx].cw_pitch;
                            high += r.receiver[rx].cw_pitch;
                        }

                        app_widgets.filter_grid.set_active_values(low, high);
                        r.receiver[rx].filter_low = low;
                        r.receiver[rx].filter_high = high;
                        r.receiver[rx].set_filter();
                        r.transmitter.filter_low = low;
                        r.transmitter.filter_high = high;
                        r.transmitter.set_filter();
                    }, filter);

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.nr_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        let active = button.is_active();

                        if button.is_active() {
                            // NR is now enabled
                            r.receiver[rx].nr = true;
                            r.receiver[rx].set_nr(); // turn on
                        } else if r.receiver[rx].nr {
                            // NR was active
                            r.receiver[rx].nr = false;
                            r.receiver[rx].set_nr(); // turn off
                            // enable NR2
                            r.receiver[rx].nr2 = true;
                            r.receiver[rx].set_nr2(); // turn on
                            button.set_label("NR2");
                            button.set_active(true);
                        } else if r.receiver[rx].nr2 {
                            // NR2 was active
                            r.receiver[rx].nr2 = false;
                            r.receiver[rx].set_nr2(); // turn off
                            // enable NR3
                            r.receiver[rx].nr3 = true;
                            r.receiver[rx].set_nr3(); // turn on
                            button.set_label("NR3");
                            button.set_active(true);
                        } else if r.receiver[rx].nr3 {
                            // NR2 was active
                            r.receiver[rx].nr3 = false;
                            r.receiver[rx].set_nr3(); // turn off
                            // enable NR4
                            r.receiver[rx].nr4 = true;
                            r.receiver[rx].set_nr4(); // turn on
                            button.set_label("NR4");
                            button.set_active(true);
                        } else {
                            r.receiver[rx].nr4 = false;
                            r.receiver[rx].set_nr4(); // turn off
                            button.set_label("NR");
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.nb_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        let active = button.is_active();

                        if button.is_active() {
                            // NR is now enabled
                            r.receiver[rx].nb = true;
                            r.receiver[rx].set_nb(); // turn on
                        } else if r.receiver[rx].nb {
                            // NR was active
                            r.receiver[rx].nb = false;
                            r.receiver[rx].set_nb(); // turn off
                            // enable NR2
                            r.receiver[rx].nb2 = true;
                            r.receiver[rx].set_nb2(); // turn on
                            button.set_label("NB2");
                            button.set_active(true);
                        } else {
                            r.receiver[rx].nb2 = false;
                            r.receiver[rx].set_nb2(); // turn off
                            button.set_label("NB");
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.anf_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        r.receiver[rx].anf = button.is_active();
                        r.receiver[rx].set_anf();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.snb_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        r.receiver[rx].snb = button.is_active();
                        r.receiver[rx].set_snb();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.mox_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        r.mox = button.is_active();
                        if r.mox && app_widgets.tun_button.is_active() {
                           app_widgets.tun_button.set_active(false);
                           r.tune = false;
                           r.transmitter.set_tuning(r.tune, r.cw_keyer_sidetone_frequency);
                        }
                        r.updated = true;
                        r.set_state();
                        if r.mox {
                            if r.split {
                                app_widgets.vfo_b_frequency.remove_css_class("vfo-b-label");
                                app_widgets.vfo_b_frequency.add_css_class("vfo-tx-label");
                            } else {
                                app_widgets.vfo_a_frequency.remove_css_class("vfo-a-label");
                                app_widgets.vfo_a_frequency.add_css_class("vfo-tx-label");
                            }
                        } else if r.split {
                            app_widgets.vfo_b_frequency.remove_css_class("vfo-tx-label");
                            app_widgets.vfo_b_frequency.add_css_class("vfo-b-label");
                        } else {
                            app_widgets.vfo_a_frequency.remove_css_class("vfo-tx-label");
                            app_widgets.vfo_a_frequency.add_css_class("vfo-a-label");
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.tun_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        r.tune = button.is_active();
                        if r.tune && app_widgets.mox_button.is_active() {
                           app_widgets.mox_button.set_active(false);
                           r.mox = false;
                        }
                        r.transmitter.set_tuning(r.tune, r.cw_keyer_sidetone_frequency);
                        r.updated = true;
                        r.set_state();
                        if r.tune {
                            if r.split {
                                app_widgets.vfo_b_frequency.remove_css_class("vfo-b-label");
                                app_widgets.vfo_b_frequency.add_css_class("vfo-tx-label");
                            } else {
                                app_widgets.vfo_a_frequency.remove_css_class("vfo-a-label");
                                app_widgets.vfo_a_frequency.add_css_class("vfo-tx-label");
                            }
                        } else if r.split {
                            app_widgets.vfo_b_frequency.remove_css_class("vfo-tx-label");
                            app_widgets.vfo_b_frequency.add_css_class("vfo-b-label");
                        } else {
                            app_widgets.vfo_a_frequency.remove_css_class("vfo-tx-label");
                            app_widgets.vfo_a_frequency.add_css_class("vfo-a-label");
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.afgain_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        r.receiver[rx].afgain = (adjustment.value() / 100.0) as f32;
                        r.receiver[rx].set_afgain();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.agc_dropdown.connect_selected_notify(move |dropdown| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        let index = dropdown.selected();
                        r.receiver[rx].agc = AGC::from_i32(index as i32).expect("Invalid AGC");
                        AGC::set_agc(&r.receiver[rx], r.receiver[rx].channel);
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.agcgain_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        r.receiver[rx].agcgain = adjustment.value() as f32;
                        r.receiver[rx].set_agcgain();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.attenuation_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        if r.dev == 6 { // HEMES_LITE
                            let b = r.receiver[0].band.to_usize();
                            r.receiver[0].band_info[b].attenuation = adjustment.value() as i32;
                        } else {
                            let rx = if r.receiver[0].active { 0 } else { 1 };
                            let b = r.receiver[rx].band.to_usize();
                            r.receiver[rx].band_info[b].attenuation = adjustment.value() as i32;
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.squelch_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let sq = adjustment.value();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        if r.receiver[rx].mode == Modes::FMN.to_usize() {
                            r.receiver[rx].fm_squelch_threshold = sq;
                        } else {
                            r.receiver[rx].am_squelch_threshold = sq;
                        }
                        r.receiver[rx].set_squelch_threshold();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.micgain_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.transmitter.micgain = adjustment.value() as f32;
                        r.transmitter.set_micgain();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.drive_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        r.transmitter.drive = adjustment.value() as f32;
                        r.updated = true;
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.cwpitch_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };
                        r.receiver[rx].cw_pitch = adjustment.value() as f64;
                        r.receiver[rx].set_filter();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.low_adjustment.connect_value_changed(move |adjustment| {
                        let mut lock = radio_mutex_clone.radio.try_lock();
                        if let Ok(ref mut mutex) = lock {
                            let mut r = lock.unwrap();
                            let rx = if r.receiver[0].active { 0 } else { 1 };
                            let app_widgets = rc_app_widgets_clone_clone.borrow();
                            app_widgets.filter_grid.set_filter_low(adjustment.value(), r.receiver[rx].mode, r.receiver[rx].filter);
                            r.receiver[rx].filter_low = adjustment.value();
                            r.receiver[rx].set_filter();
                            r.transmitter.filter_low = adjustment.value();
                            r.transmitter.set_filter();
                        } else {
                            // already locked - must be updating the filters
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.high_adjustment.connect_value_changed(move |adjustment| {
                        let mut lock = radio_mutex_clone.radio.try_lock();
                        if let Ok(ref mut mutex) = lock {
                            let mut r = lock.unwrap();
                            let rx = if r.receiver[0].active { 0 } else { 1 };
                            let app_widgets = rc_app_widgets_clone_clone.borrow();
                            app_widgets.filter_grid.set_filter_high(adjustment.value(), r.receiver[rx].mode, r.receiver[rx].filter );
                            r.receiver[rx].filter_high = adjustment.value();
                            r.receiver[rx].set_filter();
                            r.transmitter.filter_high = adjustment.value();
                            r.transmitter.set_filter();
                        } else {
                            // already locked - must be updating the filters
                        }
                    });

                    // initialize ui
                    {
                        let mut r = radio_mutex.radio.lock().unwrap();
                        let rx = if r.receiver[0].active { 0 } else { 1 };

                        app_widgets.filter_grid.update_filter_buttons(r.receiver[rx].mode);
                        r.audio[0].init();
                        r.audio[1].init();
                        r.receiver[rx].set_mode();
                        r.transmitter.init();


                        if !r.rx2_enabled {
                            app_widgets.spectrum_2_display.set_visible(false);
                            app_widgets.waterfall_2_display.set_visible(false);
                        }

                        let mut f = r.receiver[0].frequency;
                        if r.receiver[0].ctun {
                            f = r.receiver[0].ctun_frequency;
                        }
                        let formatted_value = format_u32_with_separators(f as u32);
                        app_widgets.vfo_a_frequency.set_label(&formatted_value);


                        f = r.receiver[1].frequency;
                        if r.receiver[1].ctun {
                            f = r.receiver[1].ctun_frequency;
                        }
                        let formatted_value = format_u32_with_separators(f as u32);
                        app_widgets.vfo_b_frequency.set_label(&formatted_value);


                        app_widgets.nr_button.set_active(r.receiver[rx].nr | r.receiver[rx].nr2 | r.receiver[rx].nr3 | r.receiver[rx].nr4 );
                        if r.receiver[rx].nr4 {
                            app_widgets.nr_button.set_label("NR4");
                        } else if r.receiver[rx].nr3 {
                            app_widgets.nr_button.set_label("NR3");
                        } else if r.receiver[rx].nr2 {
                            app_widgets.nr_button.set_label("NR2");
                        } else {
                            app_widgets.nr_button.set_label("NR");
                        }

                        app_widgets.nb_button.set_active(r.receiver[rx].nb | r.receiver[rx].nb2);
                        if r.receiver[rx].nb2 {
                            app_widgets.nb_button.set_label("NB2");
                        } else {
                            app_widgets.nb_button.set_label("NB");
                        }

                        app_widgets.anf_button.set_active(r.receiver[rx].anf);

                        app_widgets.snb_button.set_active(r.receiver[rx].snb);

                        //initialize the notch vector
                        r.notch = 0;
                        for i in 0..r.notches.len() {
                            let notch = r.notches[i];
                            r.add_notch(notch);
                        }

                        // enable the notches
                        unsafe {
                            RXANBPSetTuneFrequency(0, r.receiver[0].frequency as f64);
                            RXANBPSetTuneFrequency(1, r.receiver[1].frequency as f64);
                            RXANBPSetNotchesRun(0, 1);
                            RXANBPSetNotchesRun(1, 1);
                        }

                    }   

                    let rc_spectrum_clone2 = rc_spectrum_clone.clone();
                    app_widgets.spectrum_display.set_draw_func(move |_da, cr, width, height| {
                        let spectrum = rc_spectrum_clone2.borrow_mut();
                        spectrum.draw(cr, width, height);
                    });

                    let rc_waterfall_clone2 = rc_waterfall_clone.clone();
                    app_widgets.waterfall_display.set_draw_func(move |_da, cr, width, height| {
                        let waterfall = rc_waterfall_clone2.borrow_mut();
                        waterfall.draw(cr, width, height);
                    });

                    let rc_spectrum_2_clone2 = rc_spectrum_2_clone.clone();
                    app_widgets.spectrum_2_display.set_draw_func(move |_da, cr, width, height| {
                        let spectrum = rc_spectrum_2_clone2.borrow_mut();
                        spectrum.draw(cr, width, height);
                    });

                    let rc_waterfall_2_clone2 = rc_waterfall_2_clone.clone();
                    app_widgets.waterfall_2_display.set_draw_func(move |_da, cr, width, height| {
                        let waterfall = rc_waterfall_2_clone2.borrow_mut();
                        waterfall.draw(cr, width, height);
                    });

                    let rc_meter_1_clone2 = rc_meter_1_clone.clone();
                    app_widgets.meter_1_display.set_draw_func(move |_da, cr, width, height| {
                        let meter = rc_meter_1_clone2.borrow_mut();
                        meter.draw(cr);
                    });

                    let rc_meter_2_clone2 = rc_meter_2_clone.clone();
                    app_widgets.meter_2_display.set_draw_func(move |_da, cr, width, height| {
                        let meter = rc_meter_2_clone2.borrow_mut();
                        meter.draw(cr);
                    });

                    //let rc_meter_tx_clone2 = rc_meter_tx_clone.clone();
                    //app_widgets.meter_tx_display.set_draw_func(move |_da, cr, width, height| {
                    //    let mut meter = rc_meter_tx_clone2.borrow_mut();
                    //    meter.draw(cr);
                    //});

                    match device.protocol {
                        1 => {
                            let mut p1 = Protocol1::new(device);
                            let radio_mutex_clone = radio_mutex.clone();
                            thread::spawn(move || {
                                p1.run(&radio_mutex_clone);
                            });
                        },
                        2 => {
                            let mut p2 = Protocol2::new(device);
                            let radio_mutex_clone = radio_mutex.clone();
                            thread::spawn(move || {
                                p2.run(&radio_mutex_clone);
                            });
                        },
                        _ => eprintln!("Invalid protocol"),
                    }

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.main_window.connect_close_request(move |_| {
                        let r = radio_mutex_clone.radio.lock().unwrap();
                        r.save(device);
                        Propagation::Proceed
                    });

                    let mut update_interval = 100.0;
                    let r = radio_mutex.radio.lock().unwrap();
                    update_interval = 1000.0 / r.receiver[0].spectrum_fps;
                    drop(r);


                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone2 = rc_app_widgets_clone.clone();
                    let rc_spectrum_clone2 = rc_spectrum_clone.clone();
                    let rc_spectrum_2_clone2 = rc_spectrum_2_clone.clone();
                    let spectrum_timeout_id = timeout_add_local(Duration::from_millis(update_interval as u64), move || {
                        let mut rx2 = false;
                        let mut is_transmitting = false;
                        let r = radio_mutex_clone.radio.lock().unwrap();
                        rx2 = r.rx2_enabled;
                        is_transmitting = r.is_transmitting();
                        drop(r);
                        spectrum_update(&radio_mutex_clone, &rc_app_widgets_clone2, &rc_spectrum_clone2);
                        if rx2 && !is_transmitting {
                            spectrum_2_update(&radio_mutex_clone, &rc_app_widgets_clone2, &rc_spectrum_2_clone2);
                        }
                        Continue
                    });

                    {
                        let r = radio_mutex.radio.lock().unwrap();
                        update_interval = 1000.0 / r.receiver[0].waterfall_fps;
                    }
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone2 = rc_app_widgets_clone.clone();
                    let rc_waterfall_clone2 = rc_waterfall_clone.clone();
                    let rc_waterfall_2_clone2 = rc_waterfall_2_clone.clone();
                    let waterfall_timeout_id = timeout_add_local(Duration::from_millis(update_interval as u64), move || {
                        let mut rx2 = false;
                        let mut is_transmitting = false;
                        {
                            let r = radio_mutex_clone.radio.lock().unwrap();
                            rx2 = r.rx2_enabled;
                            is_transmitting = r.is_transmitting();
                        }
                        waterfall_update(&radio_mutex_clone, &rc_app_widgets_clone2, &rc_waterfall_clone2);
                        if rx2 && !is_transmitting {
                            waterfall_2_update(&radio_mutex_clone, &rc_app_widgets_clone2, &rc_waterfall_2_clone2);
                        }
                        Continue
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone2 = rc_app_widgets_clone.clone();
                    let rc_meter_1_clone2 = rc_meter_1_clone.clone();
                    let rc_meter_tx_clone2 = rc_meter_tx_clone.clone();
                    let meter_1_timeout_id = timeout_add_local(Duration::from_millis(update_interval as u64), move || {
                        meter_1_update(&radio_mutex_clone, &rc_app_widgets_clone2, &rc_meter_1_clone2);
                        Continue
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone2 = rc_app_widgets_clone.clone();
                    let rc_meter_2_clone2 = rc_meter_2_clone.clone();
                    let meter_2_timeout_id = timeout_add_local(Duration::from_millis(update_interval as u64), move || {
                        meter_2_update(&radio_mutex_clone, &rc_app_widgets_clone2, &rc_meter_2_clone2);
                        Continue
                    });

                    let update_interval = 100.0;
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone2 = rc_app_widgets_clone.clone();
                    let rc_meter_tx_clone2 = rc_meter_tx_clone.clone();
                    let meter_tx_timeout_id = timeout_add_local(Duration::from_millis(update_interval as u64), move || {
                        meter_tx_update(&radio_mutex_clone, &rc_app_widgets_clone2, &rc_meter_tx_clone2);
                        Continue
                    });


                    let mut r = radio_mutex.radio.lock().unwrap();
                    r.spectrum_timeout_id = Some(spectrum_timeout_id);
                    r.waterfall_timeout_id = Some(waterfall_timeout_id);
                    r.meter_1_timeout_id = Some(meter_1_timeout_id);
                    r.meter_2_timeout_id = Some(meter_2_timeout_id);
                    drop(r);

                    // protocol 2 needs to send a keep alive message
                    if device.protocol == 2 {
                        let radio_mutex_clone = radio_mutex.clone();
                        let keepalive_timeout_id = timeout_add_local(Duration::from_millis(250), move || {
                            let mut r = radio_mutex_clone.radio.lock().unwrap();
                            r.keepalive = true;
                            if !r.received {
                                eprintln!("no data received from radio!");
                            } else {
                                r.received = false;
                            }
                            Continue
                        });
                    }


                    let r = radio_mutex.radio.lock().unwrap();
                    let cat_enabled = r.cat_enabled;
                    drop(r);

                    let cat = CAT::new("127.0.0.1:19001".to_string());
                    let (tx, rx): (mpsc::Sender<CatMessage>, mpsc::Receiver<CatMessage>) = mpsc::channel();
                    let stop_cat_flag = Arc::new(AtomicBool::new(false));
                    let tx_clone = tx.clone();
                    let stop_flag = Arc::clone(&stop_cat_flag);
                    let cat_clone = cat.clone();
                    if cat_enabled {
                        let radio_mutex_clone = radio_mutex.clone();
                        let mut cat_clone_clone = cat_clone.clone();
                        let stop_flag_clone = stop_flag.clone();
                        thread::spawn(move || {
                            cat_clone_clone.run(&radio_mutex_clone, &tx_clone, stop_flag_clone);
                        });
                    }

                    // handle CAT button
                    let radio_mutex_clone = radio_mutex.clone();
                    let tx_clone = tx.clone();
                    let cat_clone = cat.clone();
                    app_widgets.cat_button.connect_clicked(move |button| {
                        if button.is_active() {
                            let mut r = radio_mutex_clone.radio.lock().unwrap();
                            r.cat_enabled = true;
                            drop(r);
                            stop_flag.store(false, Ordering::SeqCst);
                            let radio_mutex_clone_clone = radio_mutex_clone.clone();
                            let mut cat_clone_clone = cat_clone.clone();
                            let tx_clone_clone = tx_clone.clone();
                            let stop_flag_clone = stop_flag.clone();
                            thread::spawn(move || {
                                cat_clone_clone.run(&radio_mutex_clone_clone, &tx_clone_clone, stop_flag_clone);
                            });
                        } else {
                            stop_flag.store(true, Ordering::SeqCst);
                        }
                    });

                    // handle CAT messages
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone2 = rc_app_widgets_clone.clone();
                    glib::timeout_add_local(Duration::from_millis(100), clone!(@strong radio_mutex_clone, @strong rc_app_widgets_clone=> move || {
                        match rx.try_recv() {
                            Ok(msg) => {
                                // Message received, update the UI
                                match msg {
                                    CatMessage::UpdateMox(state) => {
                                        eprintln!("Received UpdateMox({})", state);
                                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                                        let app_widgets = rc_app_widgets_clone.borrow();
                                        r.mox = state;
                                        r.updated = true;
                                        r.set_state();
                                        if r.mox {
                                            if r.split {
                                                app_widgets.vfo_b_frequency.remove_css_class("vfo-b-label");
                                                app_widgets.vfo_b_frequency.add_css_class("vfo-tx-label");
                                            } else {
                                                app_widgets.vfo_a_frequency.remove_css_class("vfo-a-label");
                                                app_widgets.vfo_a_frequency.add_css_class("vfo-tx-label");
                                            }
                                        } else if r.split {
                                            app_widgets.vfo_b_frequency.remove_css_class("vfo-tx-label");
                                            app_widgets.vfo_b_frequency.add_css_class("vfo-b-label");
                                        } else {
                                            app_widgets.vfo_a_frequency.remove_css_class("vfo-tx-label");
                                            app_widgets.vfo_a_frequency.add_css_class("vfo-a-label");
                                        }
                                    },
                                    CatMessage::UpdateFrequencyA() => {
                                        let r = radio_mutex_clone.radio.lock().unwrap();
                                        let app_widgets = rc_app_widgets_clone.borrow();
                                        if r.receiver[0].ctun {
                                            let formatted_value = format_u32_with_separators(r.receiver[0].ctun_frequency as u32);
                                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                                        } else {
                                            let formatted_value = format_u32_with_separators(r.receiver[0].frequency as u32);
                                            app_widgets.vfo_a_frequency.set_label(&formatted_value);
                                        }    
                                    },
                                    CatMessage::UpdateFrequencyB() => {
                                        let r = radio_mutex_clone.radio.lock().unwrap();
                                        let app_widgets = rc_app_widgets_clone.borrow();
                                        if r.receiver[1].ctun {
                                            let formatted_value = format_u32_with_separators(r.receiver[1].ctun_frequency as u32);
                                            app_widgets.vfo_b_frequency.set_label(&formatted_value);
                                        } else {
                                            let formatted_value = format_u32_with_separators(r.receiver[1].frequency as u32);
                                            app_widgets.vfo_b_frequency.set_label(&formatted_value);
                                        }    
                                    },
                                }
                                // Continue the polling timeout (return Continue(true))
                                glib::ControlFlow::Continue
                            }
                            Err(TryRecvError::Empty) => {
                                // No message yet, keep polling
                                glib::ControlFlow::Continue
                            }
                            Err(TryRecvError::Disconnected) => {
                                // The Sender (tx) has been dropped, meaning the thread finished.
                                eprintln!("Thread disconnected!");
                                // Stop the polling timeout (return Break)
                                glib::ControlFlow::Break
                            }
                        }
                    }));

                    let r = radio_mutex.radio.lock().unwrap();
                    let midi_enabled = r.midi_enabled;
                    drop(r);
                    let midi = MIDI::new("Studio 2A:Studio 2A MIDI 1 28:0".to_string());
                    let (tx, rx): (mpsc::Sender<MidiMessage>, mpsc::Receiver<MidiMessage>) = mpsc::channel();
                    let stop_midi_flag = Arc::new(AtomicBool::new(false));
                    let tx_clone = tx.clone();
                    let stop_flag = Arc::clone(&stop_midi_flag);
                    let midi_clone = midi.clone();
                    if midi_enabled {
                        let radio_mutex_clone = radio_mutex.clone();
                        let midi_clone_clone = midi_clone.clone();
                        let stop_flag_clone = stop_flag.clone();
                        thread::spawn(move || {
                            midi_clone_clone.run(&radio_mutex_clone, &tx_clone, stop_flag_clone);
                        });
                    }

                    // handle MIDI button
                    let radio_mutex_clone = radio_mutex.clone();
                    let tx_clone = tx.clone();
                    let midi_clone = midi.clone();
                    app_widgets.midi_button.connect_clicked(move |button| {
                        if button.is_active() {
                            let mut r = radio_mutex_clone.radio.lock().unwrap();
                            r.midi_enabled = true;
                            drop(r);
                            stop_flag.store(false, Ordering::SeqCst);
                            let radio_mutex_clone_clone = radio_mutex_clone.clone();
                            let midi_clone_clone = midi_clone.clone();
                            let tx_clone_clone = tx_clone.clone();
                            let stop_flag_clone = stop_flag.clone();
                            thread::spawn(move || {
                                midi_clone_clone.run(&radio_mutex_clone_clone, &tx_clone_clone, stop_flag_clone);
                            });
                        } else {
                            stop_flag.store(true, Ordering::SeqCst);
                        }
                    });

                    // handle CAT messages
                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone2 = rc_app_widgets_clone.clone();
                    glib::timeout_add_local(Duration::from_millis(100), clone!(@strong radio_mutex_clone, @strong rc_app_widgets_clone=> move || {
                        match rx.try_recv() {
                            Ok(msg) => {
                                // Message received, update the UI
                                match msg {
                                    MidiMessage::StepFrequencyA(increment) => {
                                        eprintln!("MidiMessage::StepFrequencyA {}", increment);
                                        spectrum_waterfall_scroll(&radio_mutex_clone, &rc_app_widgets_clone2, 0, -increment as f64);
                                    },
                                    MidiMessage::StepFrequencyA(increment) => {
                                        eprintln!("MidiMessage::StepFrequencyB {}", increment);
                                    },
                                    _ => {
                                    }
                                }
                                // Continue the polling timeout (return Continue(true))
                                glib::ControlFlow::Continue
                            }
                            Err(TryRecvError::Empty) => { 
                                // No message yet, keep polling
                                glib::ControlFlow::Continue
                            }
                            Err(TryRecvError::Disconnected) => {
                                // The Sender (tx) has been dropped, meaning the thread finished.
                                eprintln!("Thread disconnected!");
                                // Stop the polling timeout (return Break)
                                glib::ControlFlow::Break
                            }
                        }
                    }));
                } else {
                    // try again
                }
            },
            None => {app_clone.quit();},
        }
        Propagation::Proceed
    });

    discovery_dialog.present();
    discovery_dialog.grab_focus();

    let app_widgets = rc_app_widgets.borrow();
    app_widgets.main_window.present();
}

fn spectrum_update(radio_mutex: &RadioMutex,  rc_app_widgets: &Rc<RefCell<AppWidgets>>, rc_spectrum: &Rc<RefCell<Spectrum>>) {
    let app_widgets = rc_app_widgets.borrow();
    let (flag, pixels) = radio_mutex.update_spectrum(app_widgets.spectrum_display.width());
    if flag != 0 {
        let mut spectrum = rc_spectrum.borrow_mut();
        spectrum.update(app_widgets.spectrum_display.width(), app_widgets.spectrum_display.height(), radio_mutex, &pixels);
        app_widgets.spectrum_display.queue_draw();
    }
}

fn spectrum_2_update(radio_mutex: &RadioMutex,  rc_app_widgets: &Rc<RefCell<AppWidgets>>, rc_spectrum: &Rc<RefCell<Spectrum>>) {
    let r = radio_mutex.radio.lock().unwrap();
    let is_transmitting = r.is_transmitting();
    drop(r);

    if !is_transmitting {
        let app_widgets = rc_app_widgets.borrow();
        let (flag, pixels) = radio_mutex.update_spectrum_2(app_widgets.spectrum_2_display.width());
        if flag != 0 {
            let mut spectrum = rc_spectrum.borrow_mut();
            spectrum.update(app_widgets.spectrum_2_display.width(), app_widgets.spectrum_2_display.height(), radio_mutex, &pixels);
            app_widgets.spectrum_2_display.queue_draw();
        }
    }
}

fn waterfall_update(radio_mutex: &RadioMutex,  rc_app_widgets: &Rc<RefCell<AppWidgets>>, rc_waterfall: &Rc<RefCell<Waterfall>>) {
    let r = radio_mutex.radio.lock().unwrap();
    let is_transmitting = r.is_transmitting();
    drop(r);

    if !is_transmitting {
        let app_widgets = rc_app_widgets.borrow();
        let (flag, pixels) = radio_mutex.update_waterfall(app_widgets.waterfall_display.width());
        if flag != 0 {
            let mut waterfall = rc_waterfall.borrow_mut();
            waterfall.update(app_widgets.waterfall_display.width(), app_widgets.waterfall_display.height(), radio_mutex, &pixels);
            app_widgets.waterfall_display.queue_draw();
        }
    }
}

fn waterfall_2_update(radio_mutex: &RadioMutex,  rc_app_widgets: &Rc<RefCell<AppWidgets>>, rc_waterfall_2: &Rc<RefCell<Waterfall>>) {
    let r = radio_mutex.radio.lock().unwrap();
    let is_transmitting = r.is_transmitting();
    drop(r);

    if !is_transmitting {
        let app_widgets = rc_app_widgets.borrow();
        let (flag, pixels) = radio_mutex.update_waterfall_2(app_widgets.waterfall_2_display.width());
        if flag != 0 {
            let mut waterfall = rc_waterfall_2.borrow_mut();
            waterfall.update(app_widgets.waterfall_2_display.width(), app_widgets.waterfall_2_display.height(), radio_mutex, &pixels);
            app_widgets.waterfall_2_display.queue_draw();
        }
    }
}

fn meter_1_update(radio_mutex: &RadioMutex,  rc_app_widgets: &Rc<RefCell<AppWidgets>>, rc_meter: &Rc<RefCell<Meter>>) {
    let app_widgets = rc_app_widgets.borrow();
    let mut meter = rc_meter.borrow_mut();
    let mut r = radio_mutex.radio.lock().unwrap();
    if r.is_transmitting() {
        unsafe {
            r.transmitter.alc = GetTXAMeter(r.transmitter.channel,txaMeterType_TXA_ALC_AV as i32);
        }
    } else {
        unsafe {
            r.s_meter_dbm = GetRXAMeter(r.receiver[0].channel,rxaMeterType_RXA_S_AV as i32);
        }
        meter.update_rx(r.s_meter_dbm, false);
        app_widgets.meter_1_display.queue_draw();
    }
}

fn meter_2_update(radio_mutex: &RadioMutex,  rc_app_widgets: &Rc<RefCell<AppWidgets>>, rc_meter: &Rc<RefCell<Meter>>) {
    let app_widgets = rc_app_widgets.borrow();
    let mut meter = rc_meter.borrow_mut();
    let mut r = radio_mutex.radio.lock().unwrap();
    if !r.is_transmitting() {
        unsafe {
            r.s_meter_dbm = GetRXAMeter(r.receiver[1].channel,rxaMeterType_RXA_S_AV as i32);
        }
        meter.update_rx(r.s_meter_dbm, false);
        app_widgets.meter_2_display.queue_draw();
    }
}

fn meter_tx_update(radio_mutex: &RadioMutex,  rc_app_widgets: &Rc<RefCell<AppWidgets>>, rc_meter: &Rc<RefCell<Meter>>) {
    let app_widgets = rc_app_widgets.borrow();
    let r = radio_mutex.radio.lock().unwrap();
    let is_transmitting = r.is_transmitting();
    let forward = r.transmitter.alex_forward_power;
    let reverse = r.transmitter.alex_reverse_power;
    let c1 = r.transmitter.c1;
    let c2 = r.transmitter.c2;
    let alc = r .transmitter.alc;
    let input_level = r.transmitter.input_level;
    drop(r);

    // calculate the SWR
    let fwd_power = forward as f32;
    let rev_power = reverse as f32;

    let v_fwd = (fwd_power / 4095.0) * c1;
    let fwd = (v_fwd * v_fwd) / c2;

    let v_rev = (rev_power / 4095.0) * c1;
    let rev = (v_rev * v_rev) / c2;

    let mut swr = (1.0 + (rev / fwd).sqrt())  / (1.0 - (rev / fwd).sqrt());
    if swr < 0.0 {
        swr = 1.0;
    }
    if swr.is_nan() {
        swr = 1.0;
    }

    if is_transmitting {
        let formatted_power = format!("Power: {:.1} W", fwd);
        app_widgets.tx_power.set_label(&formatted_power);
        let formatted_swr = format!("SWR: {:.1}:1", swr);
        app_widgets.tx_swr.set_label(&formatted_swr);
        let formatted_alc = format!("ALC: {:.3}", alc);
        app_widgets.tx_alc.set_label(&formatted_alc);
        app_widgets.input_level.set_fraction(input_level.into());
    } else {
        app_widgets.input_level.set_fraction(input_level.into());
    }
}

fn spectrum_waterfall_clicked(radio_mutex: &RadioMutex, rc_app_widgets: &Rc<RefCell<AppWidgets>>, rx: usize, x: f64, width: i32, button: u32) -> bool {
    let mut r = radio_mutex.radio.lock().unwrap();
    if rx == 0 {
        if !r.receiver[0].active {
            r.receiver[0].active = true;
            r.receiver[1].active = false;
            return false;
        }
    } else if !r.receiver[1].active {
        r.receiver[1].active = true;
        r.receiver[0].active = false;
        return false;
    }

    let app_widgets = rc_app_widgets.borrow();
        
    let frequency_low = r.receiver[rx].frequency - (r.receiver[rx].sample_rate/2) as f64;
    let frequency_high = r.receiver[rx].frequency + (r.receiver[rx].sample_rate/2) as f64;
    let frequency_range = frequency_high - frequency_low;
                
    let display_frequency_range = frequency_range / r.receiver[rx].zoom as f64;
    let display_frequency_offset = ((frequency_range - display_frequency_range) / 100.0) * r.receiver[rx].pan as f64;
    let display_frequency_low = frequency_low + display_frequency_offset;
    let display_frequency_high = frequency_high + display_frequency_offset;
    let display_hz_per_pixel = display_frequency_range / width as f64;
        
        
    let mut f1 = display_frequency_low + (x as f64 * display_hz_per_pixel);
    f1 = (f1 as u32 / r.receiver[rx].step as u32 * r.receiver[rx].step as u32) as f64;
    if r.receiver[rx].mode == Modes::CWL.to_usize() {
        f1 += r.receiver[rx].cw_pitch;
    } else if r.receiver[rx].mode == Modes::CWU.to_usize() {
        f1 -= r.receiver[rx].cw_pitch;
    }
    r.receiver[rx].set_frequency(f1);
    let formatted_value = format_u32_with_separators(f1 as u32);
    if rx == 0 {
        app_widgets.vfo_a_frequency.set_label(&formatted_value);
    } else {
        app_widgets.vfo_b_frequency.set_label(&formatted_value);
    }
    true
}

fn spectrum_waterfall_scroll(radio_mutex: &RadioMutex, rc_app_widgets: &Rc<RefCell<AppWidgets>>, rx: usize, dy: f64) {
    let mut r = radio_mutex.radio.lock().unwrap();
    let app_widgets = rc_app_widgets.borrow();

    let frequency_low = r.receiver[rx].frequency - (r.receiver[rx].sample_rate/2) as f64;
    let frequency_high = r.receiver[rx].frequency + (r.receiver[rx].sample_rate/2) as f64;
    let mut f1 = if r.receiver[rx].ctun {
                     r.receiver[rx].ctun_frequency
                  } else {
                     r.receiver[rx].frequency
                  };
    f1 -= r.receiver[rx].step * dy as f64;
    r.receiver[rx].set_frequency(f1);
    let formatted_value = format_u32_with_separators(f1 as u32);

    if rx == 0 {
        app_widgets.vfo_a_frequency.set_label(&formatted_value);
    } else {
        app_widgets.vfo_b_frequency.set_label(&formatted_value);
    }
}

fn update_ui(radio_mutex: &RadioMutex, rc_app_widgets: &Rc<RefCell<AppWidgets>>) {
    let r = radio_mutex.radio.lock().unwrap();
    let rx = if r.receiver[0].active { 0 } else { 1 };
    let step_index = r.receiver[rx].step_index;
    let band = r.receiver[rx].band;
    let mode = r.receiver[rx].mode;
    let filter = r.receiver[rx].filter;
    let nr = r.receiver[rx].nr;
    let nr2 = r.receiver[rx].nr2;
    let nr3 = r.receiver[rx].nr3;
    let nr4 = r.receiver[rx].nr4;
    let nb = r.receiver[rx].nb;
    let nb2 = r.receiver[rx].nb2;
    let anf = r.receiver[rx].anf;
    let snb = r.receiver[rx].snb;
    let afgain = r.receiver[rx].afgain;
    let agc = r.receiver[rx].agc;
    let agcgain = r.receiver[rx].agcgain;
    let ctun = r.receiver[rx].ctun;
    let zoom = r.receiver[rx].zoom;
    let pan = r.receiver[rx].pan;
    let cw_pitch = r.receiver[rx].cw_pitch;

    let mut b = r.receiver[rx].band.to_usize();
    let mut attenuation = r.receiver[rx].band_info[b].attenuation;
    if r.dev == 6 {
        b = r.receiver[0].band.to_usize();
        attenuation = r.receiver[0].band_info[b].attenuation;
    } else {
        b = r.receiver[rx].band.to_usize();
        attenuation = r.receiver[rx].band_info[b].attenuation;
    }
    let am_squelch_threshold = r.receiver[rx].am_squelch_threshold;
    let fm_squelch_threshold = r.receiver[rx].fm_squelch_threshold;
    drop(r);

    let mut app_widgets = rc_app_widgets.borrow_mut();

    // update step index
    app_widgets.step_dropdown.set_selected(step_index as u32);

    // update band
    if rx==0 {
        app_widgets.band_frame.set_label(Some("RX1 Band"));
        app_widgets.mode_frame.set_label(Some("RX1 Mode"));
        app_widgets.filter_frame.set_label(Some("RX1 Filter"));
    } else {
        app_widgets.band_frame.set_label(Some("RX2 Band"));
        app_widgets.mode_frame.set_label(Some("RX2 Mode"));
        app_widgets.filter_frame.set_label(Some("RX2 Filter"));
    }
    app_widgets.band_grid.set_active_index(b);
 

    // update mode
    app_widgets.mode_grid.set_active_index(mode);

    // update filter
    app_widgets.filter_grid.set_active_index(filter);

    // update NR/NR2
    app_widgets.nr_button.set_active(nr | nr2 | nr3 | nr4);
    if nr4 {
        app_widgets.nr_button.set_label("NR4");
    } else if nr3 {
        app_widgets.nr_button.set_label("NR3");
    } else if nr2 {
        app_widgets.nr_button.set_label("NR2");
    } else {
        app_widgets.nr_button.set_label("NR");
    }

    // update NB/NB2
    app_widgets.nb_button.set_active(nb | nb2);
    if nb2 {
        app_widgets.nb_button.set_label("NB2");
    } else {
        app_widgets.nb_button.set_label("NB");
    }

    // update ANF
    app_widgets.anf_button.set_active(anf);

    // update SNB
    app_widgets.snb_button.set_active(snb);

    // update AFGain
    app_widgets.afgain_adjustment.set_value((afgain * 100.0).into());

    // update AGC
    app_widgets.agc_dropdown.set_selected(agc as u32);

    // update AGCGain
    app_widgets.agcgain_adjustment.set_value(agcgain.into());

    // cw pitch
    app_widgets.cwpitch_adjustment.set_value(cw_pitch.into());

    // update CTUN
    app_widgets.ctun_button.set_active(ctun);

    // Zoom and Pan
    app_widgets.zoom_adjustment.set_value(zoom.into());
    app_widgets.pan_adjustment.set_value(pan.into());

    // Attenuation
    app_widgets.attenuation_adjustment.set_value(attenuation.into());

    // Squelch
    if mode == Modes::FMN.to_usize() {
        app_widgets.squelch_adjustment.set_value(fm_squelch_threshold);
    } else {
        app_widgets.squelch_adjustment.set_value(am_squelch_threshold);
    }

}
