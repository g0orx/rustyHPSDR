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

use crate::alex::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Adc {
    pub random: bool,
    pub dither: bool,
}

impl Default for Adc {
    fn default() -> Self {
        Self::new()
    }
}

impl Adc {

    pub fn new() -> Adc {
        let random = false;
        let dither = false;

        Adc {random, dither}
    }
}
