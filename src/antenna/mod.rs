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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Antenna {
    ANT1=0,
    ANT2,
    ANT3,
    EXT1,
    EXT2,
    XVTR
}

impl Antenna {

    pub fn from_usize(value: usize) -> Option<Self> {
        match value {
            0 => Some(Antenna::ANT1),
            1 => Some(Antenna::ANT2),
            2 => Some(Antenna::ANT3),
            3 => Some(Antenna::EXT1),
            4 => Some(Antenna::EXT2),
            5 => Some(Antenna::XVTR),
            _ => None,
        }
    }

    pub fn to_usize(&self) -> usize {
        *self as usize
    }

}
