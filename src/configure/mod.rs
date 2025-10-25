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
use gtk::{Adjustment, ApplicationWindow, Box, Builder, Button, CheckButton, ComboBoxText, DropDown, Frame, Label, ListBox, ListBoxRow, Orientation, PositionType, Scale, StringList, ToggleButton, Window};

use crate::bands::Bands;
use crate::radio::{Keyer, RadioModels, RadioMutex};
use crate::receiver::{AudioOutput};
use crate::audio::*;

pub fn create_configure_dialog(parent: &ApplicationWindow, radio_mutex: &RadioMutex) -> Window {


    let ui_xml = include_str!("../ui/configure.xml");
    let builder = Builder::from_string(ui_xml);

    let window: Window = builder
            .object("configure_window")
            .expect("Could not get object `configure_window` from builder.");

    window.set_transient_for(Some(parent)); // keeps it on top
    
    // Microphone
    let r = radio_mutex.radio.lock().unwrap();
        let remote_output1 = r.receiver[0].remote_output;
        let local_output1 = r.receiver[0].local_output;
        let output_device1 = r.receiver[0].output_device.clone();

        let remote_output2 = r.receiver[1].remote_output;
        let local_output2 = r.receiver[1].local_output;
        let output_device2 = r.receiver[1].output_device.clone();

        let input_device = r.transmitter.input_device.clone();
        let local_input = r.transmitter.local_input;
    drop(r);

    let input_devices = Audio::list_pcm_devices(true);

    /*
    let remote_input_check_button: CheckButton = builder
            .object("remote_input_check_button")
            .expect("Could not get object `remote_input_check_button` from builder.");
    remote_input_check_button.set_active(remote_input);
    let radio_mutex_clone = radio_mutex.clone();
    remote_input_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.audio[0].remote_input = is_active;
    });
    */

    let local_input_check_button: CheckButton = builder
            .object("local_input_check_button")
            .expect("Could not get object `local_input_check_button` from builder.");
    local_input_check_button.set_active(local_input);
    let radio_mutex_clone = radio_mutex.clone();
    local_input_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.local_input = is_active;
        r.transmitter.local_input_changed = true;
    });

    let input_dropdown: DropDown = builder
            .object("input_dropdown")
            .expect("Could not get object `input_dropdown` from builder.");
    let string_list_model = StringList::new(&[]);

    input_dropdown.set_model(Some(&string_list_model));

    for i in 0..input_devices.len() {
        string_list_model.append(&input_devices[i]);
        if input_devices[i] == input_device {
            input_dropdown.set_selected(i as u32);
        }
    }

    let radio_mutex_clone = radio_mutex.clone();
    input_dropdown.connect_selected_notify(move |dropdown| {
        let i = dropdown.selected();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.input_device = input_devices[i as usize].clone();
        if r.transmitter.local_input {
            r.transmitter.input_device_changed = true;
        }
    });

    let output_devices = Audio::list_pcm_devices(false);

    let rx0_remote_output_check_button: CheckButton = builder
            .object("rx0_remote_output_check_button")
            .expect("Could not get object `rx0_remote_output_check_button` from builder.");
    rx0_remote_output_check_button.set_active(remote_output1);
    let radio_mutex_clone = radio_mutex.clone();
    rx0_remote_output_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].remote_output = is_active;
    });

    let rx0_local_output_check_button: CheckButton = builder
            .object("rx0_local_output_check_button")
            .expect("Could not get object `rx0_local_output_check_button` from builder.");
    rx0_local_output_check_button.set_active(local_output1);
    let radio_mutex_clone = radio_mutex.clone();
    rx0_local_output_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].local_output = is_active;
        r.receiver[0].local_output_changed = true;
    });

    let rx0_output_dropdown: DropDown = builder
            .object("rx0_output_dropdown")
            .expect("Could not get object `rx0_output_dropdown` from builder.");
    let rx0_string_list_model = StringList::new(&[]);
    rx0_output_dropdown.set_model(Some(&rx0_string_list_model));
    for i in 0..output_devices.len() {
        rx0_string_list_model.append(&output_devices[i]);
        if output_devices[i] == output_device1 {
            rx0_output_dropdown.set_selected(i as u32);
        }
    }

    let radio_mutex_clone = radio_mutex.clone();
    let output_devices_clone = output_devices.clone();
    rx0_output_dropdown.connect_selected_notify(move |dropdown| {
        let i = dropdown.selected();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].output_device = output_devices_clone[i as usize].clone();
        if r.receiver[0].local_output {
            r.receiver[0].local_output_device_changed = true;
        }
     });

    let rx1_remote_output_check_button: CheckButton = builder
            .object("rx1_remote_output_check_button")
            .expect("Could not get object `rx1_remote_output_check_button` from builder.");
    rx1_remote_output_check_button.set_active(remote_output1);
    let radio_mutex_clone = radio_mutex.clone();
    rx1_remote_output_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[1].remote_output = is_active;
    });

    let rx1_local_output_check_button: CheckButton = builder
            .object("rx1_local_output_check_button")
            .expect("Could not get object `rx1_local_output_check_button` from builder.");
    rx1_local_output_check_button.set_active(local_output1);
    let radio_mutex_clone = radio_mutex.clone();
    rx1_local_output_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[1].local_output = is_active;
        r.receiver[1].local_output_changed = true;
    });

    let rx1_output_dropdown: DropDown = builder
            .object("rx1_output_dropdown")
            .expect("Could not get object `rx1_output_dropdown` from builder.");
    let rx1_string_list_model = StringList::new(&[]);
    rx1_output_dropdown.set_model(Some(&rx1_string_list_model));

    for i in 0..output_devices.len() {
        rx1_string_list_model.append(&output_devices[i]);
        if output_devices[i] == output_device1 {
            rx1_output_dropdown.set_selected(i as u32);
        }
    }

    let radio_mutex_clone = radio_mutex.clone();
    let output_devices_clone = output_devices.clone();
    rx1_output_dropdown.connect_selected_notify(move |dropdown| {
        let i = dropdown.selected();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[1].output_device = output_devices_clone[i as usize].clone();
        if r.receiver[1].local_output {
            r.receiver[1].local_output_device_changed = true;
        }
     });


    let r = radio_mutex.radio.lock().unwrap();
    let model = r.model;
    let adc_0_rx_antenna = r.adc[0].rx_antenna;
    let adc_0_dither = r.adc[0].dither;
    let adc_0_random = r.adc[0].random;
    drop(r);

    if model != RadioModels::HermesLite || model != RadioModels::HermesLite2 {
    let adc_0_antenna_dropdown: DropDown = builder
        .object("adc0_antenna_dropdown")
        .expect("Could not get object `adc0_antenna_dropdown` from builder.");
    adc_0_antenna_dropdown.set_selected(adc_0_rx_antenna);
    let radio_mutex_clone = radio_mutex.clone();
    adc_0_antenna_dropdown.connect_selected_notify(move |dropdown| {
        let antenna = dropdown.selected();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.adc[0].rx_antenna = antenna;
        r.updated = true;
    });

    let adc0_dither_check_button: CheckButton = builder
        .object("adc0_dither_check_button")
        .expect("Could not get object `adc0_dither_check_button` from builder.");
    adc0_dither_check_button.set_active(adc_0_dither);
    let radio_mutex_clone = radio_mutex.clone();
    adc0_dither_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.adc[0].dither = is_active;
        r.updated = true;
    });

    let adc0_random_check_button: CheckButton = builder
        .object("adc0_random_check_button")
        .expect("Could not get object `adc0_random_check_button` from builder.");
    adc0_random_check_button.set_active(adc_0_random);
    let radio_mutex_clone = radio_mutex.clone();
    adc0_random_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.adc[0].random = is_active;
        r.updated = true;
    });


    let r = radio_mutex.radio.lock().unwrap();
    let adcs = r.adc.len();
    drop(r);
    if adcs == 2 {
        let r = radio_mutex.radio.lock().unwrap();
        let adc_1_rx_antenna = r.adc[1].rx_antenna;
        let adc_1_dither = r.adc[1].dither;
        let adc_1_random = r.adc[1].random;
        drop(r);
        let adc_1_antenna_dropdown: DropDown = builder
            .object("adc1_antenna_dropdown")
            .expect("Could not get object `adc1_antenna_dropdown` from builder.");
        adc_1_antenna_dropdown.set_selected(adc_1_rx_antenna);
        let radio_mutex_clone = radio_mutex.clone();
        adc_1_antenna_dropdown.connect_selected_notify(move |dropdown| {
            let antenna = dropdown.selected();
            let mut r = radio_mutex_clone.radio.lock().unwrap();
            r.adc[1].rx_antenna = antenna;
            r.updated = true;
        });

        let adc1_dither_check_button: CheckButton = builder
            .object("adc1_dither_check_button")
            .expect("Could not get object `adc1_dither_check_button` from builder.");
        adc1_dither_check_button.set_active(adc_1_dither);
        let radio_mutex_clone = radio_mutex.clone();
        adc1_dither_check_button.connect_toggled(move |button| {
            let is_active = button.is_active();
            let mut r = radio_mutex_clone.radio.lock().unwrap();
            r.adc[1].dither = is_active;
            r.updated = true;
        });

        let adc1_random_check_button: CheckButton = builder
            .object("adc1_random_check_button")
            .expect("Could not get object `adc1_random_check_button` from builder.");
        adc1_random_check_button.set_active(adc_1_random);
        let radio_mutex_clone = radio_mutex.clone();
        adc1_random_check_button.connect_toggled(move |button| {
            let is_active = button.is_active();
            let mut r = radio_mutex_clone.radio.lock().unwrap();
            r.adc[1].random = is_active;
            r.updated = true;
        });

    } else {
        let adc1_frame: Frame = builder
                .object("adc-1-frame")
                .expect("Could not get object `adc-1-frame` from builder.");
        adc1_frame.set_visible(false);
    }
    }


    let r = radio_mutex.radio.lock().unwrap();
    let mic_boost = r.mic_boost;
    let mic_ptt = r.mic_ptt;
    let mic_bias_ring = r.mic_bias_ring;
    let mic_bias_enable = r.mic_bias_enable;
    drop(r);

    let mic_boost_check_button: CheckButton = builder
            .object("mic_boost_check_button")
            .expect("Could not get object `mic_boost_check_button` from builder.");
    mic_boost_check_button.set_active(mic_boost);
    let radio_mutex_clone = radio_mutex.clone();
    mic_boost_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.mic_boost = is_active;
        r.updated = true;
    });

    let ptt_enable_check_button: CheckButton = builder
            .object("ptt_enable_check_button")
            .expect("Could not get object `ptt_enable_check_button` from builder.");
    ptt_enable_check_button.set_active(mic_ptt);
    let radio_mutex_clone = radio_mutex.clone();
    ptt_enable_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.mic_ptt = is_active;
        r.updated = true;
    });

    let mic_bias_ring_toggle_button: ToggleButton = builder
            .object("mic_bias_ring_toggle_button")
            .expect("Could not get object `mic_bias_ring_toggle_button` from builder.");
    mic_bias_ring_toggle_button.set_active(mic_bias_ring);
    let radio_mutex_clone = radio_mutex.clone();
    mic_bias_ring_toggle_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.mic_bias_ring = is_active;
        r.updated = true;
    });

    let mic_bias_tip_toggle_button: ToggleButton = builder
            .object("mic_bias_tip_toggle_button")
            .expect("Could not get object `mic_bias_tip_toggle_button` from builder.");
    mic_bias_tip_toggle_button.set_active(!mic_bias_ring);
    let radio_mutex_clone = radio_mutex.clone();
    mic_bias_tip_toggle_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.mic_bias_ring = !is_active;
        r.updated = true;
    });


    let mic_bias_enable_check_button: CheckButton = builder
            .object("mic_bias_enable_check_button")
            .expect("Could not get object `mic_bias_enable_check_button` from builder.");
    mic_bias_enable_check_button.set_active(mic_bias_enable);
    let radio_mutex_clone = radio_mutex.clone();
    mic_bias_enable_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.mic_bias_enable = is_active;
        r.updated = true;
    });

    // Display
    let r = radio_mutex.radio.lock().unwrap();
    let spectrum_average_time = r.receiver[0].spectrum_average_time;
    drop(r);
    let spectrum_average_adjustment: Adjustment = builder
            .object("spectrum_average_adjustment")
            .expect("Could not get object `spectrum_average_adjustment` from builder.");
    spectrum_average_adjustment.set_value(spectrum_average_time.into());
    let radio_mutex_clone = radio_mutex.clone();
    spectrum_average_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].spectrum_average_time = adjustment.value() as f32;
        r.receiver[0].update_spectrum_average(r.receiver[0].channel);
    }); 
    let r = radio_mutex.radio.lock().unwrap();
    let waterfall_average_time = r.receiver[0].waterfall_average_time;
    drop(r);
    let waterfall_average_adjustment: Adjustment = builder
            .object("waterfall_average_adjustment")
            .expect("Could not get object `waterfall_average_adjustment` from builder.");
    waterfall_average_adjustment.set_value(waterfall_average_time.into());
    let radio_mutex_clone = radio_mutex.clone();
    waterfall_average_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].waterfall_average_time = adjustment.value() as f32;
        r.receiver[0].update_waterfall_average(r.receiver[0].channel);
    }); 


    // PA Calibration

    let r = radio_mutex.radio.lock().unwrap();
    let pa_2200_value = r.transmitter.pa_calibration[Bands::Band2200.to_usize()];
    drop(r);
    let pa_2200_adjustment: Adjustment = builder
            .object("pa_2200_adjustment")
            .expect("Could not get object `pa_2200_adjustment` from builder.");
    pa_2200_adjustment.set_value(pa_2200_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_2200_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band2200.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_630_value = r.transmitter.pa_calibration[Bands::Band630.to_usize()];
    drop(r);
    let pa_630_adjustment: Adjustment = builder
            .object("pa_630_adjustment")
            .expect("Could not get object `pa_630_adjustment` from builder.");
    pa_630_adjustment.set_value(pa_630_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_630_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band630.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_160_value = r.transmitter.pa_calibration[Bands::Band160.to_usize()];
    drop(r);
    let pa_160_adjustment: Adjustment = builder
            .object("pa_160_adjustment")
            .expect("Could not get object `pa_160_adjustment` from builder.");
    pa_160_adjustment.set_value(pa_160_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_160_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band160.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_80_value = r.transmitter.pa_calibration[Bands::Band80.to_usize()];
    drop(r);
    let pa_80_adjustment: Adjustment = builder
            .object("pa_80_adjustment")
            .expect("Could not get object `pa_80_adjustment` from builder.");
    pa_80_adjustment.set_value(pa_80_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_80_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band80.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_60_value = r.transmitter.pa_calibration[Bands::Band60.to_usize()];
    drop(r);
    let pa_60_adjustment: Adjustment = builder
            .object("pa_60_adjustment")
            .expect("Could not get object `pa_60_adjustment` from builder.");
    pa_60_adjustment.set_value(pa_60_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_60_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band60.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_40_value = r.transmitter.pa_calibration[Bands::Band40.to_usize()];
    drop(r);
    let pa_40_adjustment: Adjustment = builder
            .object("pa_40_adjustment")
            .expect("Could not get object `pa_40_adjustment` from builder.");
    pa_40_adjustment.set_value(pa_40_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_40_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band40.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_30_value = r.transmitter.pa_calibration[Bands::Band30.to_usize()];
    drop(r);
    let pa_30_adjustment: Adjustment = builder
            .object("pa_30_adjustment")
            .expect("Could not get object `pa_30_adjustment` from builder.");
    pa_30_adjustment.set_value(pa_30_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_30_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band30.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_20_value = r.transmitter.pa_calibration[Bands::Band20.to_usize()];
    drop(r);
    let pa_20_adjustment: Adjustment = builder
            .object("pa_20_adjustment")
            .expect("Could not get object `pa_20_adjustment` from builder.");
    pa_20_adjustment.set_value(pa_20_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_20_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band20.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_17_value = r.transmitter.pa_calibration[Bands::Band17.to_usize()];
    drop(r);
    let pa_17_adjustment: Adjustment = builder
            .object("pa_17_adjustment")
            .expect("Could not get object `pa_17_adjustment` from builder.");
    pa_17_adjustment.set_value(pa_17_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_17_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band17.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_15_value = r.transmitter.pa_calibration[Bands::Band15.to_usize()];
    drop(r);
    let pa_15_adjustment: Adjustment = builder
            .object("pa_15_adjustment")
            .expect("Could not get object `pa_15_adjustment` from builder.");
    pa_15_adjustment.set_value(pa_15_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_15_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band15.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_12_value = r.transmitter.pa_calibration[Bands::Band12.to_usize()];
    drop(r);
    let pa_12_adjustment: Adjustment = builder
            .object("pa_12_adjustment")
            .expect("Could not get object `pa_12_adjustment` from builder.");
    pa_12_adjustment.set_value(pa_12_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_12_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band12.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_10_value = r.transmitter.pa_calibration[Bands::Band10.to_usize()];
    drop(r);
    let pa_10_adjustment: Adjustment = builder
            .object("pa_10_adjustment")
            .expect("Could not get object `pa_10_adjustment` from builder.");
    pa_10_adjustment.set_value(pa_10_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_10_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band10.to_usize()] = adjustment.value() as f32;
    }); 

    let r = radio_mutex.radio.lock().unwrap();
    let pa_6_value = r.transmitter.pa_calibration[Bands::Band6.to_usize()];
    drop(r);
    let pa_6_adjustment: Adjustment = builder
            .object("pa_6_adjustment")
            .expect("Could not get object `pa_6_adjustment` from builder.");
    pa_6_adjustment.set_value(pa_6_value.into());
    let radio_mutex_clone = radio_mutex.clone();
    pa_6_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.transmitter.pa_calibration[Bands::Band6.to_usize()] = adjustment.value() as f32;
    }); 

    // Radio

    let r = radio_mutex.radio.lock().unwrap();
        let model = r.model;
    drop(r);
    let model_dropdown: DropDown = builder
            .object("model_dropdown")
            .expect("Could not get object `model_dropdown` from builder.");
    model_dropdown.set_selected(model.to_u32());
    let radio_mutex_clone = radio_mutex.clone();
    model_dropdown.connect_selected_notify(move |dropdown| {
        let model = dropdown.selected();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        //r.set_title();
        r.updated = true;
    });

    let radio_sample_rate: DropDown = builder
            .object("sample_rate_dropdown")
            .expect("Could not get object `sample_rate_dropdown` from builder.");
    let r = radio_mutex.radio.lock().unwrap();
        let protocol = r.protocol;
        let sample_rate = r.sample_rate;
    drop(r);
    if protocol == 2 {
        radio_sample_rate.set_visible(false); // Only used if Protocol 1
    } else {
        let radio_mutex_clone = radio_mutex.clone();
        let mut rate = 0;
        match sample_rate {
            48000 => rate = 0,
            96000 => rate = 1,
            192000 => rate = 2,
            384000 => rate = 3,
            _ => rate = 0,
        }
        radio_sample_rate.set_selected(rate);
        radio_sample_rate.connect_selected_notify(move |dropdown| {
            let rate = dropdown.selected();
            let mut sample_rate: i32 = 48000;
            match rate {
                0 =>  sample_rate = 48000,
                1 =>  sample_rate = 96000,
                2 =>  sample_rate = 192000,
                3 =>  sample_rate = 384000,
                4 =>  sample_rate = 768000,
                5 =>  sample_rate = 1536000,
                _ => sample_rate = 48000,
            }
            let mut r = radio_mutex_clone.radio.lock().unwrap();
            r.sample_rate_changed(sample_rate);
        });

    }


    let r = radio_mutex.radio.lock().unwrap();
        let cw_keyer_mode = r.cw_keyer_mode;
        let cw_keyer_internal = r.cw_keyer_internal;
        let cw_keys_reversed = r.cw_keys_reversed;
        let cw_breakin = r.cw_breakin;
    drop(r);

    let keyer_mode_dropdown: DropDown = builder
            .object("keyer_mode_dropdown")
            .expect("Could not get object `keyer_mode_dropdown` from builder.");
    keyer_mode_dropdown.set_selected(cw_keyer_mode.to_u32());
    let radio_mutex_clone = radio_mutex.clone();
    keyer_mode_dropdown.connect_selected_notify(move |dropdown| {
        let mode = dropdown.selected();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.cw_keyer_mode = Keyer::from_u32(mode).expect("Invalid CW Keyer Mode");
        r.updated = true;
    });

    let cw_keyer_internal_check_button: CheckButton = builder
            .object("cw_keyer_internal_check_button")
            .expect("Could not get object `cw_keyer_internal_check_button` from builder.");
    cw_keyer_internal_check_button.set_active(cw_keyer_internal);
    let radio_mutex_clone = radio_mutex.clone();
    cw_keyer_internal_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.cw_keyer_internal = is_active;
        r.updated = true;
    });

    let cw_keys_reversed_check_button: CheckButton = builder
            .object("cw_keys_reversed_check_button")
            .expect("Could not get object `cw_keys_reversed_check_button` from builder.");
    cw_keys_reversed_check_button.set_active(cw_keys_reversed);
    let radio_mutex_clone = radio_mutex.clone();
    cw_keys_reversed_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.cw_keys_reversed = is_active;
        r.updated = true;
    });

    let cw_breakin_check_button: CheckButton = builder
            .object("cw_breakin_check_button")
            .expect("Could not get object `cw_breakin_check_button` from builder.");
    cw_breakin_check_button.set_active(cw_breakin);
    let radio_mutex_clone = radio_mutex.clone();
    cw_breakin_check_button.connect_toggled(move |button| {
        let is_active = button.is_active();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.cw_breakin = is_active;
        r.updated = true;
    });

    // Noise
    let r = radio_mutex.radio.lock().unwrap();
        let taps = r.receiver[0].nr_taps;
        let delay = r.receiver[0].nr_delay;
        let gain = r.receiver[0].nr_gain;
        let leak = r.receiver[0].nr_leak;
    drop(r);

    let nr_taps_adjustment: Adjustment = builder
            .object("nr_taps_adjustment")
            .expect("Could not get object `nr_taps_adjustment` from builder.");
    nr_taps_adjustment.set_value(taps.into());
    let radio_mutex_clone = radio_mutex.clone();
    nr_taps_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].nr_taps = adjustment.value() as i32;
        r.receiver[0].update_Nrvals();
    }); 

    let nr_delay_adjustment: Adjustment = builder
            .object("nr_delay_adjustment")
            .expect("Could not get object `nr_delay_adjustment` from builder.");
    nr_delay_adjustment.set_value(delay.into());
    let radio_mutex_clone = radio_mutex.clone();
    nr_delay_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].nr_delay = adjustment.value() as i32;
        r.receiver[0].update_Nrvals();
    }); 

    let nr_gain_adjustment: Adjustment = builder
            .object("nr_gain_adjustment")
            .expect("Could not get object `nr_gain_adjustment` from builder.");
    nr_gain_adjustment.set_value(gain.into());
    let radio_mutex_clone = radio_mutex.clone();
    nr_gain_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].nr_gain = adjustment.value() as f32;
        r.receiver[0].update_Nrvals();
    }); 

    let nr_leak_adjustment: Adjustment = builder
            .object("nr_leak_adjustment")
            .expect("Could not get object `nr_leak_adjustment` from builder.");
    nr_leak_adjustment.set_value(leak.into());
    let radio_mutex_clone = radio_mutex.clone();
    nr_leak_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].nr_leak = adjustment.value() as f32;
        r.receiver[0].update_Nrvals();
    }); 

    let r = radio_mutex.radio.lock().unwrap();
        let taps = r.receiver[0].anf_taps;
        let delay = r.receiver[0].anf_delay;
        let gain = r.receiver[0].anf_gain;
        let leak = r.receiver[0].anf_leak;
    drop(r);

    let anf_taps_adjustment: Adjustment = builder
            .object("anf_taps_adjustment")
            .expect("Could not get object `anf_taps_adjustment` from builder.");
    anf_taps_adjustment.set_value(taps.into());
    let radio_mutex_clone = radio_mutex.clone();
    anf_taps_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].anf_taps = adjustment.value() as i32;
        r.receiver[0].update_Anfvals();
    });

    let anf_delay_adjustment: Adjustment = builder
            .object("anf_delay_adjustment")
            .expect("Could not get object `anf_delay_adjustment` from builder.");
    anf_delay_adjustment.set_value(delay.into());
    let radio_mutex_clone = radio_mutex.clone();
    anf_delay_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].anf_delay = adjustment.value() as i32;
        r.receiver[0].update_Anfvals();
    });

    let anf_gain_adjustment: Adjustment = builder
            .object("anf_gain_adjustment")
            .expect("Could not get object `anf_gain_adjustment` from builder.");
    anf_gain_adjustment.set_value(gain.into());
    let radio_mutex_clone = radio_mutex.clone();
    anf_gain_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].anf_gain = adjustment.value() as f32;
        r.receiver[0].update_Anfvals();
    });

    let anf_leak_adjustment: Adjustment = builder
            .object("anf_leak_adjustment")
            .expect("Could not get object `anf_leak_adjustment` from builder.");
    anf_leak_adjustment.set_value(leak.into());
    let radio_mutex_clone = radio_mutex.clone();
    anf_leak_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].anf_leak = adjustment.value() as f32;
        r.receiver[0].update_Anfvals();
    });

    let r = radio_mutex.radio.lock().unwrap();
        let position = r.receiver[0].agc_position;
    drop(r);
    let pre_agc_check_button: CheckButton = builder
            .object("pre_agc_check_button")
            .expect("Could not get object `pre_agc_check_button` from builder.");
    pre_agc_check_button.set_active(position == 0);
    let radio_mutex_clone = radio_mutex.clone();
    pre_agc_check_button.connect_toggled(move |button| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        if button.is_active() {
            let rx = r.active_receiver;
            r.receiver[rx].agc_position = 0;
        }
    });

    let r = radio_mutex.radio.lock().unwrap();
        let position = r.receiver[0].agc_position;
    drop(r);
    let post_agc_check_button: CheckButton = builder
            .object("post_agc_check_button")
            .expect("Could not get object `post_agc_check_button` from builder.");
    post_agc_check_button.set_active(position == 1);
    let radio_mutex_clone = radio_mutex.clone();
    post_agc_check_button.connect_toggled(move |button| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        if button.is_active() {
            let rx = r.active_receiver;
            r.receiver[rx].agc_position = 1;
        }
    });

    // Notch
    let r = radio_mutex.radio.lock().unwrap();
    //let notch = r.notch;
    //let notches = &r.notches;
    //drop(r);
    let notch_list: ListBox = builder
            .object("notch_list")
            .expect("Could not get object `notch_list` from builder.");

    for i in 0..r.notches.len() {
        let row = ListBoxRow::new();
        let hbox = gtk::Box::new(Orientation::Horizontal, 10);
        let id = format!("{:?}", i);
        let label_id = Label::new(Some(&id));
        label_id.set_xalign(0.0); // Align text to the left
        hbox.append(&label_id);
        let frequency = format!("{:?}", r.notches[i].frequency);
        let label_frequency = Label::new(Some(&frequency));
        label_frequency.set_xalign(0.0); // Align text to the left
        hbox.append(&label_frequency);
        let width = format!("{:?}", r.notches[i].width);
        let label_width = Label::new(Some(&width));
        label_width.set_xalign(0.0); // Align text to the left
        hbox.append(&label_width);
        let active = format!("{:?}", r.notches[i].active);
        let label_active = Label::new(Some(&active));
        label_active.set_xalign(0.0); // Align text to the left
        hbox.append(&label_active);
        row.set_child(Some(&hbox));

        notch_list.append(&row);
    }
    drop(r);

    // Receiver
    let r = radio_mutex.radio.lock().unwrap();
    let rx_0_adc = r.receiver[0].adc;
    let rx_1_adc = r.receiver[1].adc;
    let adcs = r.adc.len();
    drop(r);

    let rx_0_adc_adjustment: Adjustment = builder
            .object("rx0_adc_adjustment")
            .expect("Could not get object `rx0_adc_adjustment` from builder.");
    rx_0_adc_adjustment.set_value(rx_0_adc as f64);
    let radio_mutex_clone = radio_mutex.clone();
    rx_0_adc_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].adc = adjustment.value() as usize;
        r.updated = true;
    });
    if adcs < 2 {
        let rx_0_adc_frame: Frame = builder
            .object("rx0_adc_frame")
            .expect("Could not get object `rx0_adc_frame` from builder.");
        rx_0_adc_frame.set_visible(false);
    }

    let rx0_sample_rate: DropDown = builder
            .object("rx0_sample_rate_dropdown")
            .expect("Could not get object `rx0_sample_rate_dropdown` from builder.");
    let r = radio_mutex.radio.lock().unwrap();
        let protocol = r.protocol;
        let sample_rate = r.receiver[0].sample_rate;
    drop(r);
    if protocol == 1 {
        rx0_sample_rate.set_visible(false);
    } else {
        let radio_mutex_clone = radio_mutex.clone();
        let mut rate = 0;
        match sample_rate {
                48000 =>  rate = 0,
                96000 =>  rate = 1,
                192000 =>  rate = 2,
                384000 =>  rate = 3,
                768000 =>  rate = 4,
                1536000 =>  rate = 5,
                _ => rate = 0,
        }
        rx0_sample_rate.set_selected(rate);
        rx0_sample_rate.connect_selected_notify(move |dropdown| {
            let rate = dropdown.selected();
            let mut sample_rate: i32 = 48000;
            match rate {
                0 =>  sample_rate = 48000,
                1 =>  sample_rate = 96000,
                2 =>  sample_rate = 192000,
                3 =>  sample_rate = 384000,
                4 =>  sample_rate = 768000,
                5 =>  sample_rate = 1536000,
                _ => sample_rate = 48000,
            }
            let mut r = radio_mutex_clone.radio.lock().unwrap();
            r.receiver[0].sample_rate_changed(sample_rate);
        });
    }

    let rx0_audio: DropDown = builder
            .object("rx0_audio_dropdown")
            .expect("Could not get object `rx0_audio_dropdown` from builder.");
    let r = radio_mutex.radio.lock().unwrap();
        let audio_output = r.receiver[0].audio_output;
    drop(r);
    let radio_mutex_clone = radio_mutex.clone();
    rx0_audio.set_selected(audio_output.to_u32());
    rx0_audio.connect_selected_notify(move |dropdown| {
        let output = dropdown.selected();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[0].audio_output = AudioOutput::from_u32(output);
    });

    let r = radio_mutex.radio.lock().unwrap();
    let adcs = r.adc.len();
    drop(r);
    let rx_1_adc_adjustment: Adjustment = builder
            .object("rx1_adc_adjustment")
            .expect("Could not get object `rx1_adc_adjustment` from builder.");
    rx_1_adc_adjustment.set_value(rx_1_adc as f64);
    let radio_mutex_clone = radio_mutex.clone();
    rx_1_adc_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[1].adc = adjustment.value() as usize;
        r.updated = true;
    });
    if adcs < 2 {
        let rx_1_adc_frame: Frame = builder
            .object("rx1_adc_frame")
            .expect("Could not get object `rx1_adc_frame` from builder.");
        rx_1_adc_frame.set_visible(false);
    }

    let rx1_sample_rate: DropDown = builder
            .object("rx1_sample_rate_dropdown")
            .expect("Could not get object `rx1_sample_rate_dropdown` from builder.");
    let r = radio_mutex.radio.lock().unwrap();
        let protocol = r.protocol;
        let sample_rate = r.receiver[1].sample_rate;
    drop(r);
    if protocol == 1 {
        rx1_sample_rate.set_visible(false);
    } else {
        let radio_mutex_clone = radio_mutex.clone();
        let mut rate = 0;
        match sample_rate {
                48000 =>  rate = 0,
                96000 =>  rate = 1,
                192000 =>  rate = 2,
                384000 =>  rate = 3,
                768000 =>  rate = 4,
                1536000 =>  rate = 5,
                _ => rate = 0,
        }
        rx1_sample_rate.set_selected(rate);
        rx1_sample_rate.connect_selected_notify(move |dropdown| {
            let rate = dropdown.selected();
            let mut sample_rate: i32 = 48000;
            match rate {
                0 =>  sample_rate = 48000,
            1 =>  sample_rate = 96000,
                2 =>  sample_rate = 192000,
                3 =>  sample_rate = 384000,
                4 =>  sample_rate = 768000,
                5 =>  sample_rate = 1536000,
                _ => sample_rate = 48000,
            }
            let mut r = radio_mutex_clone.radio.lock().unwrap();
            r.receiver[1].sample_rate_changed(sample_rate);
        });
    }

  
    let rx1_audio: DropDown = builder
            .object("rx1_audio_dropdown")
            .expect("Could not get object `rx1_audio_dropdown` from builder.");
    let r = radio_mutex.radio.lock().unwrap();
        let audio_output = r.receiver[1].audio_output;
    drop(r);
    let radio_mutex_clone = radio_mutex.clone();
    rx1_audio.set_selected(audio_output.to_u32());
    rx1_audio.connect_selected_notify(move |dropdown| {
        let output = dropdown.selected();
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        r.receiver[1].audio_output = AudioOutput::from_u32(output);
    });



    // Equalizer
    let r = radio_mutex.radio.lock().unwrap();
    let rx = r.active_receiver;
    let enabled = r.receiver[rx].equalizer_enabled;
    let preamp = r.receiver[rx].equalizer_preamp as f64;
    let low = r.receiver[rx].equalizer_low as f64;
    let mid = r.receiver[rx].equalizer_mid as f64;
    let high = r.receiver[rx].equalizer_high as f64;
    drop(r);

    let equalizer_enabled_check_button: CheckButton = builder
            .object("equalizer_enabled_check_button")
            .expect("Could not get object `equalizer_enabled_check_button` from builder.");
    equalizer_enabled_check_button.set_active(enabled);
    let radio_mutex_clone = radio_mutex.clone();
    equalizer_enabled_check_button.connect_toggled(move |button| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        r.receiver[rx].equalizer_enabled = button.is_active();
        r.receiver[rx].enable_equalizer();
    });

    let preamp_scale: Scale = builder
            .object("preamp_scale")
            .expect("Could not get object `preamp_scale` from builder.");
    preamp_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    preamp_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    preamp_scale.add_mark(15.0, PositionType::Left, Some("15dB"));

    let low_scale: Scale = builder
            .object("low_scale")
            .expect("Could not get object `low_scale` from builder.");
    low_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    low_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    low_scale.add_mark(15.0, PositionType::Left, Some("15dB"));


    let mid_scale: Scale = builder
            .object("mid_scale")
            .expect("Could not get object `mid_scale` from builder.");
    mid_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    mid_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    mid_scale.add_mark(15.0, PositionType::Left, Some("15dB"));

    let high_scale: Scale = builder
            .object("high_scale")
            .expect("Could not get object `high_scale` from builder.");
    high_scale.add_mark(-12.0, PositionType::Left, Some("-12dB"));
    high_scale.add_mark(0.0, PositionType::Left, Some("0dB"));
    high_scale.add_mark(15.0, PositionType::Left, Some("15dB"));

    let preamp_adjustment: Adjustment = builder
            .object("preamp_adjustment")
            .expect("Could not get object `preamp_adjustment` from builder.");
    preamp_adjustment.set_value(preamp);
    let radio_mutex_clone = radio_mutex.clone();
    preamp_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        r.receiver[rx].equalizer_preamp = adjustment.value() as f32;
        r.receiver[rx].set_equalizer_values();
    });
    let low_adjustment: Adjustment = builder
            .object("low_adjustment")
            .expect("Could not get object `low_adjustment` from builder.");
    low_adjustment.set_value(low);
    let radio_mutex_clone = radio_mutex.clone();
    low_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        r.receiver[rx].equalizer_low = adjustment.value() as f32;
        r.receiver[rx].set_equalizer_values();
    });
    let mid_adjustment: Adjustment = builder
            .object("mid_adjustment")
            .expect("Could not get object `mid_adjustment` from builder.");
    mid_adjustment.set_value(mid);
    let radio_mutex_clone = radio_mutex.clone();
    mid_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        r.receiver[rx].equalizer_mid = adjustment.value() as f32;
        r.receiver[rx].set_equalizer_values();
    });
    let high_adjustment: Adjustment = builder
            .object("high_adjustment")
            .expect("Could not get object `high_adjustment` from builder.");
    high_adjustment.set_value(high);
    let radio_mutex_clone = radio_mutex.clone();
    high_adjustment.connect_value_changed(move |adjustment| {
        let mut r = radio_mutex_clone.radio.lock().unwrap();
        let rx = r.active_receiver;
        r.receiver[rx].equalizer_high = adjustment.value() as f32;
        r.receiver[rx].set_equalizer_values();
    });

    let ok_button: Button = builder
            .object("ok_button")
            .expect("Could not get object `ok_button` from builder.");
    let window_for_ok = window.clone();
    ok_button.connect_clicked(move |_| {
        window_for_ok.close();
    });

    window
}

