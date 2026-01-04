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
use gtk::cairo::Context;
use gdk_pixbuf::{Colorspace, Pixbuf};

use crate::radio::RadioMutex;

#[derive(Clone)]
pub struct Waterfall {
    rx: usize,
    pixbuf: Pixbuf,
    updated: bool,
}

impl Waterfall {

    pub fn new(id: usize, width: i32, height: i32) -> Self {
        let rx = id;
        let pixbuf = Pixbuf::new(Colorspace::Rgb, false, 8, width, height).unwrap();
        let updated = false;
        Self {
            rx,
            pixbuf,
            updated,
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        if width!=0 && height !=0 {
            let new_pixbuf = Pixbuf::new(Colorspace::Rgb, false, 8, width, height).unwrap();
            self.pixbuf = new_pixbuf;
        }
        self.updated = false;
    }

    pub fn update(&mut self, _width:i32, _height: i32, radio_mutex: &RadioMutex, new_pixels: &Vec<f32>) {
        let mut r = radio_mutex.radio.lock().unwrap();
        let mut average = 0.0;
        unsafe {
            let pixels = self.pixbuf.pixels();
            let width = self.pixbuf.width() as usize;
            let height = self.pixbuf.height() as usize;
            let rowstride = self.pixbuf.rowstride() as usize;
            //let channels = self.pixbuf.n_channels() as usize;

            for y in (0..height - 1).rev() { // Iterate in reverse order
                let src_offset = y * rowstride;
                let dest_offset = (y + 1) * rowstride;
                if dest_offset + rowstride <= pixels.len() {
                    pixels.copy_within(src_offset..src_offset + rowstride, dest_offset);
                }
            }

            // fill in the top line with the latest spectrum data
            //let waterfall_width = r.receiver[self.rx].spectrum_width;
            let waterfall_width = r.receiver[self.rx].waterfall_width;
            let pan = ((new_pixels.len() as f32 - waterfall_width as f32) / 100.0) * r.receiver[self.rx].pan as f32;

            let b = r.receiver[self.rx].band.to_usize();
            for x in 0..waterfall_width {
                let mut R = 0.0;
                let mut G = 0.0;
                let mut B = 0.0;

                let value: f32 = new_pixels[x as usize + pan as usize];
                if value < r.receiver[self.rx].band_info[b].spectrum_low {
                    average += r.receiver[self.rx].band_info[b].spectrum_low;
                } else {
                    average += value;
                }

                if value >= (r.receiver[self.rx].band_info[b].waterfall_low + 6.0) {
                    R = 255.0;
                    G = 255.0;
                    B = 0.0;
                }

                let ix = (x * 3) as usize;
                pixels[ix] = R as u8;
                pixels[ix + 1] = G as u8;
                pixels[ix + 2] = B as u8;
            }
            //println!("average {} max_percent {}", average / width as f32, max_percent);
            if r.waterfall_auto {
                r.receiver[self.rx].band_info[b].waterfall_low = (r.receiver[self.rx].band_info[b].waterfall_low + (average / width as f32)) / 2.0;
            }
        } // unsafe
        self.updated = true;
    }

    pub fn draw(&self, cr: &Context, _width: i32, _height: i32) {
        if self.updated {
            cr.set_source_pixbuf(&self.pixbuf, 0.0, 0.0);
            cr.paint().unwrap();
        }
    }
}
