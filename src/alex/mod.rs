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

pub const ALEX_RX_ANTENNA_NONE: u32 =    0x00000000;
pub const ALEX_RX_ANTENNA_XVTR: u32 =    0x00000100;
pub const ALEX_RX_ANTENNA_EXT1: u32 =    0x00000200;
pub const ALEX_RX_ANTENNA_EXT2: u32 =    0x00000400;
pub const ALEX_RX_ANTENNA_OUT1: u32 =    0x00000800;

pub const ALEX_ANTENNA_1: u32 =          0x01000000;
pub const ALEX_ANTENNA_2: u32 =          0x02000000;
pub const ALEX_ANTENNA_3: u32 =          0x04000000;

pub const ALEX_ATTENUATION_0dB: u32 =    0x00000000;
pub const ALEX_ATTENUATION_10dB: u32 =   0x00004000;
pub const ALEX_ATTENUATION_20dB: u32 =   0x00002000;
pub const ALEX_ATTENUATION_30dB: u32 =   0x00006000;

pub const ALEX_30_20_LPF: u32 =          0x00100000;
pub const ALEX_60_40_LPF: u32 =          0x00200000;
pub const ALEX_80_LPF: u32 =             0x00400000;
pub const ALEX_160_LPF: u32 =            0x00800000;
pub const ALEX_6_BYPASS_LPF: u32 =       0x20000000;
pub const ALEX_12_10_LPF: u32 =          0x40000000;
pub const ALEX_17_15_LPF: u32 =          0x80000000;

pub const ALEX_13MHZ_HPF: u32 =          0x00000002;
pub const ALEX_20MHZ_HPF: u32 =          0x00000004;
pub const ALEX_9_5MHZ_HPF: u32 =         0x00000010;
pub const ALEX_6_5MHZ_HPF: u32 =         0x00000020;
pub const ALEX_1_5MHZ_HPF: u32 =         0x00000040;
pub const ALEX_BYPASS_HPF: u32 =         0x00001000;

pub const ALEX_6M_PREAMP: u32 =          0x00000008;

