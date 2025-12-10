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

use gtk::{Adjustment, Application, ApplicationWindow, Builder, Button, DrawingArea, DropDown, Frame, Grid, Label, ProgressBar, ToggleButton};

use crate::bands::*;
use crate::modes::*;
use crate::filters::*;

pub struct AppWidgets {
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
    pub squelch_adjustment: Adjustment,
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
    pub input_level: ProgressBar,
}

impl AppWidgets {

    pub fn from_builder(builder: &Builder) -> Self {
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

        let squelch_adjustment: Adjustment = builder
            .object("squelch_adjustment")
            .expect("Could not get squelch_adjustment from builder");

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
            squelch_adjustment,
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

