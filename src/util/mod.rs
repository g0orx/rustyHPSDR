
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

use std::fmt::Write;

pub fn format_u32_with_separators(value: u32) -> String {
    let mut result = String::new();
    let value_str = value.to_string();
    let len = value_str.len();

    // Iterate over the characters and insert separators
    for (i, ch) in value_str.chars().enumerate() {
        if (len - i).is_multiple_of(3) && i != 0 {
            write!(&mut result, ".").unwrap();
        }
        write!(&mut result, "{}", ch).unwrap();
    }

    result
}

