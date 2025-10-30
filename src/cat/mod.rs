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


const RIG_ID: &str = "019";
const VFO_A_FREQ: &str = "142000000"; // Internal VFO state placeholder
const VFO_A_MODE: &str = "1"; // Internal mode state placeholder (1=USB)

struct CAT {
}

impl CAT {

    fn parse_command(&self, command: &str) -> String {
        let cmd = command.trim_end_matches(';').to_uppercase();
        let command_code = &cmd[..2];
        return match command_code {
            "ID" => format!("ID{};", RIG_ID),
            "FA" => format!("FA{};", VFO_A_FREQ), // Read VFO A frequency
            "MD" => format!("MD{};", VFO_A_MODE), // Read Mode
            // 'IF' command is complex (returns a long status string), so we send a minimal placeholder
            "IF" => "IF142000000+00000000100000000000000000000000000;".to_string(),
            "ZZ" => self.parse_zz_command(&cmd),
            _ => "?;".to_string(), // Unknown command response
        };
    }

    fn parse_zz_command(&self, command: &str) -> String {
        let command_code = &command[..4];
        return match command_code {
            _ => "?;".to_string(), // Unknown command response
        }
    }

}

