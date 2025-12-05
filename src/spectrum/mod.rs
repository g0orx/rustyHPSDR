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

use gtk::cairo::{Context, Format, ImageSurface, LineCap, LineJoin, LinearGradient};

use crate::radio::RadioMutex;
use crate::modes::*;
use crate::util::*;

#[derive(Clone)]
pub struct Spectrum {
    rx: usize,
    surface: ImageSurface,
    updated: bool,
}

impl Spectrum {

    pub fn new(id: usize, width: i32, height: i32) -> Self {
        let rx = id;
        let surface = ImageSurface::create(Format::ARgb32, width, height).expect("Failed to create surface");
        let updated = false;
        Self {
            rx,
            surface,
            updated,
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        let surface = ImageSurface::create(Format::ARgb32, width, height).expect("Failed to create surface");
        self.surface = surface;
        let cr = Context::new(&self.surface).expect("Couldn't create cairo context from surface");
        cr.set_source_rgb(0.0, 0.0, 0.0); // black
        cr.paint().expect("Failed to paint black background on surface");
        self.updated = false;
    }

    pub fn update(&mut self, width: i32, height: i32, radio_mutex: &RadioMutex, pixels: &Vec<f32>) {
        let r = radio_mutex.radio.lock().unwrap();
        let spectrum_height = height - 10; // leave space for the frequency
        let cr = Context::new(self.surface.clone()).expect("Couldn't create cairo context from surface");
        if r.receiver[self.rx].active {
            cr.set_source_rgb(0.0, 0.0, 1.0); // dark blue
        } else {
            cr.set_source_rgb(0.0, 0.0, 0.5); // blue
        }
        cr.paint().expect("Failed to paint background on surface");

        if r.is_transmitting() && self.rx==0 {
            // draw the spectrum
            let dbm_per_line: f32 = spectrum_height as f32/(r.transmitter.spectrum_high-r.transmitter.spectrum_low);     
            let spectrum_high = r.transmitter.spectrum_high;
            let spectrum_width = r.transmitter.spectrum_width;

            let mut multiplier = 3; // protocol 1
            if r.protocol == 2 {
                multiplier = 12; // protocol 2
            }               
            let pixel_len = spectrum_width * multiplier;

            let hz_per_pixel = r.transmitter.output_rate as f32 / pixel_len as f32;

            if pixels.len() == pixel_len as usize {
                cr.set_source_rgb(1.0, 1.0, 0.0);
                cr.move_to(0.0, spectrum_height as f64);
                let offset=(pixel_len as f32 / 2.0)-(spectrum_width as f32 / 2.0);
                for i in 0..spectrum_width {
                    let pixel = pixels[(i + offset as i32) as usize];
                    let y = ((spectrum_high - pixel) * dbm_per_line).floor();
                    cr.line_to(i as f64, y.into());
                }
                cr.line_to(spectrum_width as f64, spectrum_height as f64);
                cr.stroke().unwrap();
            }

            // draw the filter
            cr.set_source_rgba (0.4, 0.4, 0.4, 0.80);
            let center = spectrum_width / 2;
            let filter_left = center as f32 + (r.transmitter.filter_low / hz_per_pixel);
            let filter_right = center as f32 + (r.transmitter.filter_high / hz_per_pixel);
            cr.rectangle(filter_left.into(), 0.0, (filter_right-filter_left).into(), spectrum_height.into());
            let _ = cr.fill();

            // draw the cursor
            cr.set_source_rgb (1.0, 0.0, 0.0);
            cr.set_line_width(1.0);
            cr.move_to((width/2).into(), 0.0);
            cr.line_to((width/2).into(), spectrum_height.into());
            cr.stroke().unwrap();
        } else {
            let b = r.receiver[self.rx].band.to_usize();
            let dbm_per_line: f32 = spectrum_height as f32/(r.receiver[self.rx].band_info[b].spectrum_high-r.receiver[self.rx].band_info[b].spectrum_low);

            cr.set_source_rgb(1.0, 1.0, 0.0);
            cr.set_line_width(1.0);
            cr.set_line_cap(LineCap::Round);
            cr.set_line_join(LineJoin::Round);

            let frequency_low = r.receiver[self.rx].frequency - (r.receiver[self.rx].sample_rate/2) as f32;
            let frequency_high = r.receiver[self.rx].frequency + (r.receiver[self.rx].sample_rate/2) as f32;
            let frequency_range = frequency_high - frequency_low;
   
            let display_frequency_range = frequency_range / r.receiver[self.rx].zoom as f32;
            let display_frequency_offset = ((frequency_range - display_frequency_range) / 100.0) * r.receiver[self.rx].pan as f32;
            let display_frequency_low = frequency_low + display_frequency_offset;
            let display_frequency_high = frequency_high + display_frequency_offset;
            let display_hz_per_pixel = display_frequency_range / width as f32;

            let mut step = 25000.0;
            match r.receiver[self.rx].sample_rate {
                 48000 => step = 50000.0,
                 96000 => step = 10000.0,
                192000 => step = 20000.0,
                384000 => match r.receiver[self.rx].zoom {
                              1 => step = 25000.0,
                              2 => step = 25000.0,
                              3 => step = 10000.0,
                              4 => step = 10000.0,
                              5 => step = 10000.0,
                              6 => step = 5000.0,
                              7 => step = 5000.0,
                              8 => step = 5000.0,
                              9 => step = 5000.0,
                              10 => step = 5000.0,
                              11 => step = 2000.0,
                              12 => step = 2000.0,
                              13 => step = 2000.0,
                              14 => step = 2000.0,
                              15 => step = 2000.0,
                              16 => step = 2000.0,
                              _ => step = 25000.0,
                          },
                768000 => step = 50000.0,
               1536000 => step = 100000.0,
                     _ => step = 25000.0,
            }

            // draw the band limits
            cr.set_source_rgb(1.0, 0.0, 0.0);
            let dashes = [4.0, 4.0];
            let offset = 0.0;
            cr.set_dash(&dashes, offset);
            cr.set_line_width(4.0);
            if display_frequency_low < r.receiver[self.rx].band_info[b].low && display_frequency_high > r.receiver[self.rx].band_info[b].low {
                let x = (r.receiver[self.rx].band_info[b].low - display_frequency_low) / display_hz_per_pixel;
                cr.move_to( x.into(), 0.0);
                cr.line_to( x.into(), spectrum_height.into());
            }

            if display_frequency_low < r.receiver[self.rx].band_info[b].high && display_frequency_high > r.receiver[self.rx].band_info[b].high {
                let x = (r.receiver[self.rx].band_info[b].high - display_frequency_low) / display_hz_per_pixel;
                cr.move_to( x.into(), 0.0);
                cr.line_to( x.into(), spectrum_height.into());
            }
            cr.stroke().unwrap();
            cr.set_dash(&[], 0.0);
            cr.set_line_width(2.0);

            // draw signal levels
            for i in r.receiver[self.rx].band_info[b].spectrum_low as i32 .. r.receiver[self.rx].band_info[b].spectrum_high as i32 {
                if i % r.receiver[self.rx].spectrum_step as i32 == 0 {
                    let y = (r.receiver[self.rx].band_info[b].spectrum_high - i as f32) * dbm_per_line;
                    cr.set_source_rgb(0.5, 0.5, 0.5);
                    cr.move_to(0.0, y.into());
                    cr.line_to(width as f64, y.into());
                    cr.stroke().unwrap();
                    let text = format!("{} dBm", i);
                    cr.set_source_rgb(1.0, 1.0, 0.0);
                    cr.move_to( 5.0, (y-2.0).into());
                    let _ = cr.show_text(&text);
                }
            }

            let mut frequency = r.receiver[self.rx].frequency;
            if r.receiver[self.rx].ctun {
                frequency = r.receiver[self.rx].ctun_frequency;
            }
/*
            if r.receiver[self.rx].mode == Modes::CWL.to_usize() {
                frequency += r.receiver[self.rx].cw_pitch;
            } else if r.receiver[self.rx].mode == Modes::CWU.to_usize() {
                frequency -= r.receiver[self.rx].cw_pitch;
            }
*/
            // see if cursor and filter visible
            if display_frequency_low < frequency && display_frequency_high > frequency {
                // draw the center line frequency marker
                let x = (frequency - display_frequency_low) / display_hz_per_pixel;
                    cr.set_source_rgb(1.0, 0.0, 0.0);
                cr.set_line_width(1.0);
                cr.move_to(x.into(), 0.0);
                cr.line_to(x.into(), spectrum_height.into());
                cr.stroke().unwrap();

                // draw the filter
                cr.set_source_rgba (0.4, 0.4, 0.4, 0.80);
                let filter_left = ((frequency + r.receiver[self.rx].filter_low) - display_frequency_low) / display_hz_per_pixel;
                let filter_right = ((frequency + r.receiver[self.rx].filter_high) - display_frequency_low) / display_hz_per_pixel;
                cr.rectangle(filter_left.into(), 0.0, (filter_right-filter_left).into(), spectrum_height.into());
                let _ = cr.fill();
            }

            // draw the spectrum
            let spectrum_high = r.receiver[self.rx].band_info[b].spectrum_high;
            let spectrum_width = r.receiver[self.rx].spectrum_width;
            let pan = ((pixels.len() as f32 - spectrum_width as f32) / 100.0) * r.receiver[self.rx].pan as f32;
                cr.set_source_rgb(1.0, 1.0, 0.0);
            cr.move_to(0.0, spectrum_height as f64);
            for i in 0..spectrum_width {
                let pixel = pixels[i as usize + pan as usize];
                let mut y = ((spectrum_high - pixel) * dbm_per_line).floor();
                if y > spectrum_height as f32 {
                    y = spectrum_height as f32;
                }
                cr.line_to(i as f64, y.into());
            }
            cr.line_to(width as f64, spectrum_height as f64);
            // fill the spectrum
            let pattern = LinearGradient::new(0.0, (spectrum_height-20) as f64, 0.0, 0.0);
            let mut s9: f32 = -73.0;
            s9 = ((r.receiver[self.rx].band_info[b].spectrum_high - s9)
                          * (spectrum_height-20) as f32
                        / (r.receiver[self.rx].band_info[b].spectrum_high - r.receiver[self.rx].band_info[b].spectrum_low)).floor();
            s9 = 1.0-(s9/(spectrum_height-20) as f32);
                pattern.add_color_stop_rgb(0.0,0.0,1.0,0.0); // Green
                pattern.add_color_stop_rgb((s9/3.0).into(),1.0,0.65,0.0); // Orange
                pattern.add_color_stop_rgb(((s9/3.0)*2.0).into(),1.0,1.0,0.0); // Yellow
                pattern.add_color_stop_rgb(s9.into(),1.0,0.0,0.0); // Red
            cr.set_source(&pattern).expect("Failed to set source");
            cr.close_path();
            let _ = cr.fill_preserve();
            cr.stroke().unwrap();

            // draw the frequency markers
            let mut f: f32 = (((display_frequency_low as i32 + step as i32) / step as i32) * step as i32) as f32;
            while f < display_frequency_high {
                let x = (f - display_frequency_low) / display_hz_per_pixel;
                cr.set_source_rgb(0.5, 0.5, 0.5);
                cr.move_to( x.into(), 0.0);
                cr.line_to( x.into(), spectrum_height.into());
                cr.stroke().unwrap();
                let text = format_u32_with_separators((f / 1000.0) as u32);
                cr.set_source_rgb(1.0, 1.0, 1.0);
                let pango_layout = pangocairo::functions::create_layout(&cr);
                pango_layout.set_text(&text);
                let (text_width, _text_height) = pango_layout.pixel_size();
                cr.move_to( (x - (text_width as f32 / 2.0)).into(), height.into());
                let _ = cr.show_text(&text);
                f += step as f32;
            }

            // draw any active notches
            for i in 0..r.notch {
                let notch = r.notches[i as usize];
                if notch.frequency > display_frequency_low as f64 && notch.frequency < display_frequency_high as f64 {
                    let x = (notch.frequency - display_frequency_low as f64) / display_hz_per_pixel as f64;
                    cr.set_source_rgb(1.0, 1.0, 1.0);
                    cr.move_to( x, 0.0);
                    cr.line_to( x, spectrum_height.into());
                    cr.stroke().unwrap();
                }
            }

        }
        self.updated = true;

    }

    pub fn draw(&self, cr: &Context, _width: i32, _height: i32) {
        if self.updated {
            cr.set_source_surface(&self.surface, 0.0, 0.0).expect("failed to set source surface");
            cr.paint().expect("Failed to pant surface");
        }
    }
}
