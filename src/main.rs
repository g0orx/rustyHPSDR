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

use glib::ControlFlow::Continue;
use glib::timeout_add_local;
use gtk::prelude::*;
use gtk::{Adjustment, Application, ApplicationWindow, Builder, Button, DrawingArea, DropDown, Frame, Grid, Label, ProgressBar, ToggleButton};
use gtk::{EventController, EventControllerMotion, EventControllerScroll, EventControllerScrollFlags, GestureClick};
use gtk::gdk::Cursor;
use gtk::glib::Propagation;

use std::cell::{Cell, RefCell};
use std::env;
use std::ffi::CString;
use std::fs;
use std::os::raw::c_char;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rustyHPSDR::agc::*;
use rustyHPSDR::bands::*;
use rustyHPSDR::modes::*;
use rustyHPSDR::filters::*;
use rustyHPSDR::discovery::create_discovery_dialog;
use rustyHPSDR::discovery::device_name;
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

struct AppWidgets {
    pub main_window: ApplicationWindow,
    pub configure_button: Button,
    pub vfo_a_frequency: Label,
    pub vfo_b_frequency: Label,
    pub a_to_b_button: Button,
    pub b_to_a_button: Button,
    pub a_swap_b_button: Button,
    pub split_button: ToggleButton,
    pub ctun_button: ToggleButton,
    pub rx2_button: ToggleButton,
    pub step_dropdown: DropDown,
    pub meter_1_display: DrawingArea,
    pub meter_2_display: DrawingArea,
    //pub meter_tx_display: DrawingArea,
    pub spectrum_display: DrawingArea,
    pub waterfall_display: DrawingArea,
    pub spectrum_2_display: DrawingArea,
    pub waterfall_2_display: DrawingArea,
    pub zoom_adjustment: Adjustment,
    pub pan_adjustment: Adjustment,
    pub nr_button: ToggleButton,
    pub nb_button: ToggleButton,
    pub anf_button: ToggleButton,
    pub snb_button: ToggleButton,
    pub mox_button: ToggleButton,
    pub tun_button: ToggleButton,
    pub afgain_adjustment: Adjustment,
    pub agc_dropdown: DropDown,
    pub agcgain_adjustment: Adjustment,
    pub attenuation_adjustment: Adjustment,
    pub micgain_adjustment: Adjustment,
    pub drive_adjustment: Adjustment,
    pub band_frame: Frame,
    pub mode_frame: Frame,
    pub filter_frame: Frame,
    pub band_grid: BandGrid,
    pub mode_grid: ModeGrid,
    pub filter_grid: FilterGrid,
    pub cwpitch_adjustment: Adjustment,
    pub low_adjustment: Adjustment,
    pub high_adjustment: Adjustment,
    pub tx_power: Label,
    pub tx_swr: Label,
    pub tx_alc: Label,
    //pub input_level: Label,
    pub input_level: ProgressBar,
}

impl AppWidgets {
    fn from_builder(builder: &Builder) -> Self {
        let main_window: ApplicationWindow = builder
            .object("main_window")
            .expect("Could not get object `main_window` from builder.");

        let configure_button: Button = builder
            .object("configure_button")
            .expect("Could not get configure_button from builder");

        let vfo_a_frequency: Label = builder
            .object("vfo_a_frequency")
            .expect("Could not get vfo_a_frequency from builder");

        let vfo_b_frequency: Label = builder
            .object("vfo_b_frequency")
            .expect("Could not get vfo_b_frequency from builder");

        let a_to_b_button: Button = builder
            .object("a_to_b_button")
            .expect("Could not get a_to_b_button from builder");

        let b_to_a_button: Button = builder
            .object("b_to_a_button")
            .expect("Could not get b_to_a_button from builder");

        let a_swap_b_button: Button = builder
            .object("a_swap_b_button")
            .expect("Could not get a_swap_b_button from builder");

        let split_button: ToggleButton = builder
            .object("split_button")
            .expect("Could not get split_button from builder");

        let ctun_button: ToggleButton = builder
            .object("ctun_button")
            .expect("Could not get ctun_button from builder");

        let rx2_button: ToggleButton = builder
            .object("rx2_button")
            .expect("Could not get rx2_button from builder");

        let step_dropdown = builder
            .object("step_dropdown")
            .expect("Could not get step_dropdown from builder");

        let meter_1_display: DrawingArea = builder
            .object("meter_1_display")
            .expect("Could not get meter_1_display from builder");

        let meter_2_display: DrawingArea = builder
            .object("meter_2_display")
            .expect("Could not get meter_2_display from builder");

        //let meter_tx_display: DrawingArea = builder
        //    .object("meter_tx_display")
        //    .expect("Could not get meter_tx_display from builder");

        let spectrum_display: DrawingArea = builder
            .object("spectrum_display")
            .expect("Could not get spectrum_display from builder");

        let waterfall_display: DrawingArea = builder
            .object("waterfall_display")
            .expect("Could not get waterfall_display from builder");

        let spectrum_2_display: DrawingArea = builder
            .object("spectrum_2_display")
            .expect("Could not get spectrum_2_display from builder");

        let waterfall_2_display: DrawingArea = builder
            .object("waterfall_2_display")
            .expect("Could not get waterfall_2_display from builder");

        let band_grid: Grid = builder
            .object("band_grid")
            .expect("Could not get band_grid from builder");

        let mode_grid: Grid = builder
            .object("mode_grid")
            .expect("Could not get mode_grid from builder");

        let filter_grid: Grid = builder
            .object("filter_grid")
            .expect("Could not get filter_grid from builder");

        let zoom_adjustment: Adjustment = builder
            .object("zoom_adjustment")
            .expect("Could not get zoom_adjustment from builder");

        let pan_adjustment: Adjustment = builder
            .object("pan_adjustment")
            .expect("Could not get pan_adjustment from builder");

        let nr_button: ToggleButton = builder
            .object("nr_button")
            .expect("Could not get nr_button from builder");

        let nb_button: ToggleButton = builder
            .object("nb_button")
            .expect("Could not get nb_button from builder");

        let anf_button: ToggleButton = builder
            .object("anf_button")
            .expect("Could not get anf_button from builder");

        let snb_button: ToggleButton = builder
            .object("snb_button")
            .expect("Could not get snb_button from builder");

        let mox_button: ToggleButton = builder
            .object("mox_button")
            .expect("Could not get mox_button from builder");

        let tun_button: ToggleButton = builder
            .object("tun_button")
            .expect("Could not get tun_button from builder");

        let afgain_adjustment: Adjustment = builder
            .object("afgain_adjustment")
            .expect("Could not get afgain_adjustment from builder");

        let agc_dropdown: DropDown = builder
            .object("agc_dropdown")
            .expect("Could not get agc_dropdown from builder");

        let agcgain_adjustment: Adjustment = builder
            .object("agcgain_adjustment")
            .expect("Could not get agcgain_adjustment from builder");

        let attenuation_adjustment: Adjustment = builder
            .object("attenuation_adjustment")
            .expect("Could not get attenuation_adjustment from builder");

        let micgain_adjustment: Adjustment = builder
            .object("micgain_adjustment")
            .expect("Could not get micgain_adjustment from builder");

        let drive_adjustment: Adjustment = builder
            .object("drive_adjustment")
            .expect("Could not get drive_adjustment from builder");

        let cwpitch_adjustment: Adjustment = builder
            .object("cwpitch_adjustment")
            .expect("Could not get cwpitch_adjustment from builder");

        let low_adjustment: Adjustment = builder
            .object("low_adjustment")
            .expect("Could not get low_adjustment from builder");

        let high_adjustment: Adjustment = builder
            .object("high_adjustment")
            .expect("Could not get high_adjustment from builder");

        let band_frame: Frame = builder
            .object("band_frame")
            .expect("Could not get band_frame from builder");

        let mode_frame: Frame = builder
            .object("mode_frame")
            .expect("Could not get mode_frame from builder");

        let filter_frame: Frame = builder
            .object("filter_frame")
            .expect("Could not get filter_frame from builder");

        let band_grid = BandGrid::new(builder);
        let mode_grid = ModeGrid::new(builder);
        let filter_grid = FilterGrid::new(builder);

        let tx_power: Label = builder
            .object("tx_power")
            .expect("Could not get tx_power from builder");

        let tx_swr: Label = builder
            .object("tx_swr")
            .expect("Could not get tx_swr from builder");

        let tx_alc: Label = builder
            .object("tx_alc")
            .expect("Could not get tx_alc from builder");

        //let input_level: Label = builder
        let input_level: ProgressBar = builder
            .object("input_level")
            .expect("Could not get input_level from builder");

        AppWidgets {
            main_window,
            configure_button,
            vfo_a_frequency,
            vfo_b_frequency,
            a_to_b_button,
            b_to_a_button,
            a_swap_b_button,
            split_button,
            ctun_button,
            rx2_button,
            step_dropdown,
            meter_1_display,
            meter_2_display,
            //meter_tx_display,
            spectrum_display,
            waterfall_display,
            spectrum_2_display,
            waterfall_2_display,
            zoom_adjustment,
            pan_adjustment,
            nr_button,
            nb_button,
            anf_button,
            snb_button,
            mox_button,
            tun_button,
            afgain_adjustment,
            agc_dropdown,
            agcgain_adjustment,
            attenuation_adjustment,
            micgain_adjustment,
            drive_adjustment,
            cwpitch_adjustment,
            low_adjustment,
            high_adjustment,
            band_frame,
            mode_frame,
            filter_frame,
            band_grid,
            mode_grid,
            filter_grid,
            tx_power,
            tx_swr,
            tx_alc,
            input_level,
        }
    }
}

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
    let c_path_ptr: *const c_char = c_string.as_ptr();
    unsafe {
        WDSPwisdom(c_path_ptr);
    }

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
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }

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
                        }
                        app_widgets.attenuation_adjustment.set_value(r.adc[r.receiver[rx].adc].attenuation.into());
                        app_widgets.micgain_adjustment.set_value(r.transmitter.micgain.into());
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
                        let configure_dialog = create_configure_dialog(&app_widgets.main_window, &radio_mutex_clone);
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
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
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
                        let mut update = false;
                        if r.receiver[1].active {
                            r.receiver[1].active = false;
                            r.receiver[0].active = true;
                            update = true;
                        }
                        drop(r);
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
                            let mut rx: usize = 0;
                            if r.receiver[1].active {
                                rx = 1;
                            }
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
                    let cursor_uparrow = Cursor::from_name("up-arrow", None);
                    let cursor_downarrow = Cursor::from_name("down-arrow", None);

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
                                app_widgets.spectrum_display.set_cursor(cursor_uparrow.as_ref());
                            } else if y > bottom.into() {
                                app_widgets.spectrum_display.set_cursor(cursor_downarrow.as_ref());
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
                    let cursor_uparrow = Cursor::from_name("up-arrow", None);
                    let cursor_downarrow = Cursor::from_name("down-arrow", None);

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
                                app_widgets.spectrum_2_display.set_cursor(cursor_uparrow.as_ref());
                            } else if y > bottom.into() {
                                app_widgets.spectrum_2_display.set_cursor(cursor_downarrow.as_ref());
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
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
                        r.receiver[rx].zoom = adjustment.value() as i32;
                        let channel = r.receiver[rx].channel;
                        let width = r.receiver[rx].spectrum_width;
                        r.receiver[rx].init_analyzer(channel, width);
                        let mut p = 0.0;
                        if adjustment.value() == 1.0 {
                            r.receiver[rx].pan = p as i32;
                        } else {
                            // try to keep the current frequency in the zoomed area
                            let frequency_low = r.receiver[rx].frequency - (r.receiver[rx].sample_rate/2) as f32;
                            let frequency_high = r.receiver[rx].frequency + (r.receiver[rx].sample_rate/2) as f32;
                            let frequency_range = frequency_high - frequency_low;
                            let width = r.receiver[rx].spectrum_width * r.receiver[rx].zoom;
                            let mut f = r.receiver[rx].frequency;
                            let hz_per_pixel = frequency_range / width as f32;
                            if r.receiver[rx].ctun {
                                f = r.receiver[rx].ctun_frequency;
                            }
                            p = (f - frequency_low) / hz_per_pixel;
                            p = (p / width as f32) * 100.0;
                            r.receiver[rx].pan = p as i32;
                        }
                        drop(r);
                        app_widgets.pan_adjustment.set_value(p as f64);
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.pan_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
                        if r.receiver[rx].zoom > 1 {
                            r.receiver[rx].pan = adjustment.value() as i32;
                        } else {
                            r.receiver[rx].pan = 0;
                            adjustment.set_value(r.receiver[rx].pan.into());
                        }
                    });

                    let r = radio_mutex.radio.lock().unwrap();
                    let mut rx = 0;
                    if r.receiver[1].active {
                        rx = 1;
                    }
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
                        // save current band info
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }

                        let b = r.receiver[rx].band.to_usize();
                        if b != index { // band has changed
                            r.receiver[rx].band_info[b].current = r.receiver[rx].frequency;

                            // get new band info
                            r.receiver[rx].band = Bands::from_usize(index).expect("invalid band index");
                            r.receiver[rx].frequency = r.receiver[rx].band_info[index].current;
                            if r.receiver[rx].ctun {
                                r.receiver[rx].ctun_frequency = r.receiver[rx].frequency;
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
                            let (low, high) = app_widgets.filter_grid.get_filter_values(r.receiver[rx].band_info[index].mode.to_usize(), r.receiver[rx].band_info[index].filter.to_usize());
                            app_widgets.filter_grid.set_active_values(low, high);
                            r.receiver[rx].filter_low = low;
                            r.receiver[rx].filter_high = high;
                            if r.receiver[rx].mode == Modes::CWL.to_usize() {
                                r.receiver[rx].filter_low = -r.receiver[rx].cw_pitch - low;
                                r.receiver[rx].filter_high = -r.receiver[rx].cw_pitch + high;
                            } else if r.receiver[rx].mode == Modes::CWU.to_usize() {
                                r.receiver[rx].filter_low = r.receiver[rx].cw_pitch - low;
                                r.receiver[rx].filter_high = r.receiver[rx].cw_pitch + high;
                            }
                            r.receiver[rx].set_mode();

                            r.transmitter.filter_low = low;
                            r.transmitter.filter_high = high;
                            r.transmitter.mode = r.receiver[rx].band_info[index].mode.to_usize();
                            r.transmitter.set_mode();
                            r.transmitter.set_filter();

                            let formatted_value = format_u32_with_separators(r.receiver[rx].frequency as u32);
                            if rx == 0 {
                                app_widgets.vfo_a_frequency.set_label(&formatted_value);
                            } else {
                                app_widgets.vfo_b_frequency.set_label(&formatted_value);
                            }
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
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
                        r.receiver[rx].mode = index; 
                        app_widgets.filter_grid.update_filter_buttons(index);

                        let (low, high) = app_widgets.filter_grid.get_filter_values(index, r.receiver[rx].filter);
                        app_widgets.filter_grid.set_active_values(low, high);
                        r.receiver[rx].filter_low = low;
                        r.receiver[rx].filter_high = high;

                        if r.receiver[rx].mode == Modes::CWL.to_usize() {
                            r.receiver[rx].filter_low = -r.receiver[rx].cw_pitch - low;
                            r.receiver[rx].filter_high = -r.receiver[rx].cw_pitch + high;
                        } else if r.receiver[rx].mode == Modes::CWU.to_usize() {
                            r.receiver[rx].filter_low = r.receiver[rx].cw_pitch - low;
                            r.receiver[rx].filter_high = r.receiver[rx].cw_pitch + high;
                        }
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
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
                        let app_widgets = rc_app_widgets_clone_clone.borrow();
                        r.receiver[rx].filter = index;
                        let (low, high) = app_widgets.filter_grid.get_filter_values(r.receiver[rx].mode, r.receiver[rx].filter);
                        app_widgets.filter_grid.set_active_values(low, high);
                        r.receiver[rx].filter_low = low;
                        r.receiver[rx].filter_high = high;

                        if r.receiver[rx].mode == Modes::CWL.to_usize() {
                            r.receiver[rx].filter_low = -r.receiver[rx].cw_pitch - low;
                            r.receiver[rx].filter_high = -r.receiver[rx].cw_pitch + high;
                        } else if r.receiver[rx].mode == Modes::CWU.to_usize() {
                            r.receiver[rx].filter_low = r.receiver[rx].cw_pitch - low;
                            r.receiver[rx].filter_high = r.receiver[rx].cw_pitch + high;
                        }
                        r.receiver[rx].set_filter();

                        r.transmitter.filter_low = low;
                        r.transmitter.filter_high = high;
                        r.transmitter.set_filter();
                    }, filter);

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.nr_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
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
                        } else {
                            r.receiver[rx].nr2 = false;
                            r.receiver[rx].set_nr2(); // turn off
                            button.set_label("NR");
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.nb_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
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
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
                        r.receiver[rx].anf = button.is_active();
                        r.receiver[rx].set_anf();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.snb_button.clone().connect_clicked(move |button| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
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
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
                        r.receiver[rx].afgain = (adjustment.value() / 100.0) as f32;
                        r.receiver[rx].set_afgain();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.agc_dropdown.connect_selected_notify(move |dropdown| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
                        let index = dropdown.selected();
                        r.receiver[rx].agc = AGC::from_i32(index as i32).expect("Invalid AGC");
                        AGC::set_agc(&r.receiver[rx], r.receiver[rx].channel);
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.agcgain_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
                        r.receiver[rx].agcgain = adjustment.value() as f32;
                        r.receiver[rx].set_agcgain();
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    app_widgets.attenuation_adjustment.connect_value_changed(move |adjustment| {
                        let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
                        let adc = r.receiver[rx].adc;
                        r.adc[adc].attenuation = adjustment.value() as i32;
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
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }
                        r.receiver[rx].cw_pitch = adjustment.value() as f32;
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.low_adjustment.connect_value_changed(move |adjustment| {
                        //let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut lock = radio_mutex_clone.radio.try_lock();
                        if let Ok(ref mut mutex) = lock {
                            // update the filter low setting
                            let app_widgets = rc_app_widgets_clone_clone.borrow();
                            app_widgets.filter_grid.set_filter_low(adjustment.value() as f32);

                            // update the receiver low setting
                        } else {
                            // already locked - must be updating the filters
                        }
                    });

                    let radio_mutex_clone = radio_mutex.clone();
                    let rc_app_widgets_clone_clone = rc_app_widgets_clone.clone();
                    app_widgets.high_adjustment.connect_value_changed(move |adjustment| {
                        //let mut r = radio_mutex_clone.radio.lock().unwrap();
                        let mut lock = radio_mutex_clone.radio.try_lock();
                        if let Ok(ref mut mutex) = lock {
                            // update the filter high setting
                            let app_widgets = rc_app_widgets_clone_clone.borrow();
                            app_widgets.filter_grid.set_filter_high(adjustment.value() as f32);
                            // update the receiver high setting
                        } else {
                            // already locked - must be updating the filters
                        }
                    });

                    // initialize ui
                    {
                        let mut r = radio_mutex.radio.lock().unwrap();
                        let mut rx = 0;
                        if r.receiver[1].active {
                            rx = 1;
                        }

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


                        app_widgets.nr_button.set_active(r.receiver[rx].nr | r.receiver[rx].nr2);
                        if r.receiver[rx].nr2 {
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
                        let r = radio_mutex_clone.radio.lock().unwrap();
                        rx2 = r.rx2_enabled;
                        is_transmitting = r.is_transmitting();
                        drop(r);
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
    let r = radio_mutex.radio.lock().unwrap();
    let is_transmitting = r.is_transmitting();
    drop(r);

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
    let meter = rc_meter.borrow_mut();
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
        //let formatted_level = format!("Input Level: {:.2}", input_level);
        //app_widgets.input_level.set_label(&formatted_level);
        app_widgets.input_level.set_fraction(input_level);
    } else {
        //let formatted_level = format!("Input Level: {:.2}", input_level);
        //app_widgets.input_level.set_label(&formatted_level);
        app_widgets.input_level.set_fraction(input_level);
    }

    //if is_transmitting {
    //    meter.update_tx(forward, reverse);
    //    app_widgets.meter_tx_display.queue_draw();
   // }
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
        
    let frequency_low = r.receiver[rx].frequency - (r.receiver[rx].sample_rate/2) as f32;
    let frequency_high = r.receiver[rx].frequency + (r.receiver[rx].sample_rate/2) as f32;
    let frequency_range = frequency_high - frequency_low;
                
    let display_frequency_range = frequency_range / r.receiver[rx].zoom as f32;
    let display_frequency_offset = ((frequency_range - display_frequency_range) / 100.0) * r.receiver[rx].pan as f32;
    let display_frequency_low = frequency_low + display_frequency_offset;
    let display_frequency_high = frequency_high + display_frequency_offset;
    let display_hz_per_pixel = display_frequency_range / width as f32;
        
        
    let f1 = display_frequency_low + (x as f32 * display_hz_per_pixel);
    let f1 = (f1 as u32 / r.receiver[rx].step as u32 * r.receiver[rx].step as u32) as f32;
        
    if r.receiver[rx].ctun {
        r.receiver[rx].ctun_frequency = f1;
        r.receiver[rx].set_ctun_frequency();
        let formatted_value = format_u32_with_separators(r.receiver[rx].ctun_frequency as u32);
        if rx == 0 {
            app_widgets.vfo_a_frequency.set_label(&formatted_value);
        } else {
            app_widgets.vfo_b_frequency.set_label(&formatted_value);
        }
    } else {
        r.receiver[rx].frequency = f1;
        let formatted_value = format_u32_with_separators(r.receiver[rx].frequency as u32);
        if rx == 0 {
            app_widgets.vfo_a_frequency.set_label(&formatted_value);
        } else {
            app_widgets.vfo_b_frequency.set_label(&formatted_value);
        }
    }

    unsafe {
        RXANBPSetTuneFrequency(rx as i32, r.receiver[rx].frequency as f64);
    }

    true
}

fn spectrum_waterfall_scroll(radio_mutex: &RadioMutex, rc_app_widgets: &Rc<RefCell<AppWidgets>>, rx: usize, dy: f64) {
    let mut r = radio_mutex.radio.lock().unwrap();
    let app_widgets = rc_app_widgets.borrow();

    let frequency_low = r.receiver[rx].frequency - (r.receiver[rx].sample_rate/2) as f32;
    let frequency_high = r.receiver[rx].frequency + (r.receiver[rx].sample_rate/2) as f32;
    if r.receiver[rx].ctun {
        r.receiver[rx].ctun_frequency -= r.receiver[rx].step * dy as f32;
        if r.receiver[rx].ctun_frequency < frequency_low {
            r.receiver[rx].ctun_frequency = frequency_low;
        } else if r.receiver[rx].ctun_frequency > frequency_high {
            r.receiver[rx].ctun_frequency = frequency_high;
        }
        let formatted_value = format_u32_with_separators(r.receiver[rx].ctun_frequency as u32);
        if rx == 0 {
            app_widgets.vfo_a_frequency.set_label(&formatted_value);
        } else {
            app_widgets.vfo_b_frequency.set_label(&formatted_value);
        }
        r.receiver[rx].set_ctun_frequency();
    } else {
        r.receiver[rx].frequency -= r.receiver[rx].step * dy as f32;
        let formatted_value = format_u32_with_separators(r.receiver[rx].frequency as u32);
        if rx == 0 {
            app_widgets.vfo_a_frequency.set_label(&formatted_value);
        } else {
            app_widgets.vfo_b_frequency.set_label(&formatted_value);
        }
    }

    unsafe {
        RXANBPSetTuneFrequency(rx as i32, r.receiver[rx].frequency as f64);
    }
}

fn update_ui(radio_mutex: &RadioMutex, rc_app_widgets: &Rc<RefCell<AppWidgets>>) {
    let r = radio_mutex.radio.lock().unwrap();
    let mut rx = 0;
    if r.receiver[1].active {
        rx = 1;
    }
    let step_index = r.receiver[rx].step_index;
    let band = r.receiver[rx].band;
    let mode = r.receiver[rx].mode;
    let filter = r.receiver[rx].filter;
    let nr = r.receiver[rx].nr;
    let nr2 = r.receiver[rx].nr2;
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
    drop(r);

    let app_widgets = rc_app_widgets.borrow();

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
    let band_button = app_widgets.band_grid.get_button(band.to_usize());
    band_button.emit_by_name::<()>("clicked", &[]);

    // update mode
    let mode_button = app_widgets.mode_grid.get_button(mode);
    mode_button.emit_by_name::<()>("clicked", &[]);

    // update filter
    let filter_button = app_widgets.filter_grid.get_button(filter);
    filter_button.emit_by_name::<()>("clicked", &[]);

    // update NR/NR2
    app_widgets.nr_button.set_active(nr | nr2);
    if nr2 {
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

    app_widgets.zoom_adjustment.set_value(zoom.into());
    app_widgets.pan_adjustment.set_value(pan.into());
}
