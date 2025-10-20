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

use alsa::pcm::*;
use alsa::device_name::HintIter;
use alsa::{Direction, ValueOr, Error};

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Audio {
    pub remote_input: bool,
    pub local_input: bool,
    pub input_device: String,
#[serde(skip_serializing, skip_deserializing)]
    input: Option<PCM>,
#[serde(skip_serializing, skip_deserializing)]
    pub period_size: i64,
    pub remote_output: bool,
    pub local_output: bool,
    pub output_device: String,
#[serde(skip_serializing, skip_deserializing)]
    output: Option<PCM>,
#[serde(skip_serializing, skip_deserializing)]
    pub output_underruns: i32,
}

impl Audio {

    pub fn new() -> Audio {
        let remote_input = true;
        let local_input = false;
        let input_device = String::from("default"); 
        let input = None;
        let period_size = 0;
        let remote_output = true;
        let local_output = false;
        let output_device = String::from("default"); 
        let output = None;
        let output_underruns = 0;

        
        Audio{remote_input, local_input, input_device, input, period_size, remote_output, local_output, output_device, output, output_underruns}
    }

    pub fn init(&mut self) {
        self.input = None;
        self.output = None;
        self.output_underruns = 0;
        if self.local_input {
            let _ = self.open_input();
        }
        if self.local_output {
            let _ = self.open_output();
        }
    }

    pub fn open_input(&mut self) -> Result<(), Error> {
        println!("audio::open_input");
        let pcm = PCM::new(&self.input_device, Direction::Capture, false)?;
        {
            let hwp = HwParams::any(&pcm)?;
            println!("audio::open_input configure hardware params");
            hwp.set_channels(2).expect("create_input failed to set channels");
            hwp.set_rate(48000, ValueOr::Nearest)?;
            hwp.set_format(Format::s16())?;
            hwp.set_access(Access::RWInterleaved)?;
            pcm.hw_params(&hwp)?;
            match hwp.get_period_size() {
                Ok(s) => {
                    self.period_size = s;
                    println!("audio::open_input period_size={}", s);
                },
                Err(e) => {
                    eprintln!("Failed to get period_size");
                }
            };
        }
        self.input = Some(pcm);
        println!("audio::open_input Ok");
        Ok(())
    }

    pub fn read_input(&mut self) -> Vec<i16> {
        println!("audio::read_input");
        let mut buffer = vec![0i16; self.period_size as usize];
        let mut frames_read = 0;
        let io = match self.input.as_ref().expect("failed to get self.input").io_i16() {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Failed to get io_i16: {}", e);
                let audio_data = buffer[..frames_read].to_vec();
                return audio_data;
            }
        };
        frames_read = match io.readi(&mut buffer) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to read frames: {}", e);
                let audio_data = buffer[..frames_read].to_vec();
                return audio_data;
            }
        };
        println!("audio::read_input frames read={}", frames_read);
        
        buffer[..frames_read].to_vec()
    }

    pub fn close_input(&mut self) ->  Result<(), Error> {
        println!("audio::close_input");
        self.input = None;
        Ok(())
    }

    pub fn open_output(&mut self) -> Result<(), Error> {
        let pcm = PCM::new(&self.output_device, Direction::Playback, false)?;
        {
            let hwp = HwParams::any(&pcm)?;
            hwp.set_channels(2)?;
            hwp.set_rate(48000, ValueOr::Nearest)?;
            hwp.set_format(Format::s16())?;
            hwp.set_access(Access::RWInterleaved)?;
            pcm.hw_params(&hwp)?;
        }
        self.output = Some(pcm);
        Ok(())
    }

    pub fn close_output(&mut self) ->  Result<(), Error> {
        self.output = None;
        Ok(())
    }

    pub fn write_output(&mut self, buffer: &Vec<i16>) -> Result<(), Error> {
        match self.output.as_ref().expect("Could not access output to get delay").delay() {
            Ok(delay) => {
                let mut trim = 0;
                let max_delay = 3 * buffer.len() as i64;
                if delay > max_delay {
                    trim = delay - max_delay;
                }
                let io = self.output.as_ref().expect("write_output: failed to get io").io_i16().expect("write_output: failed to get io_i16");
                if trim > 0 {
                    let n = buffer.len() - trim as usize;
                    if n > 0  {
                        let trimmed_buffer = buffer[0..n].to_vec();
                        match io.writei(&trimmed_buffer) {
                            Ok(_frames) => {
                            }
                            Err(e) => {
                                self.output_underruns += 1;
                                match self.output.as_ref().expect("output failed for prepare").prepare() {
                                    Ok(()) => {
                                    }
                                    Err(e) => {
                                        eprintln!("write_output: prepare failed: {}", e);
                                    }
                                }
                            }
                        }

                    }
                } else {
                    match io.writei(buffer) {
                        Ok(_frames) => {
                        }
                        Err(e) => {
                            self.output_underruns += 1;
                            match self.output.as_ref().expect("output failed for prepare").prepare() {
                                Ok(()) => {
                                }
                                Err(e) => {
                                    eprintln!("write_output: prepare failed: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get delay: {}", e);
            }
        }

        Ok(())
    }

    pub fn list_pcm_devices(direction: Direction) -> Vec<String> {
        let mut devices = Vec::<String>::new();
        let hints = HintIter::new_str(None, "pcm").unwrap();
        for hint in hints {
            if hint.name.is_some() && hint.desc.is_some() && (hint.direction.is_none() || hint.direction.map(|dir| dir == direction).unwrap_or_default()) {
                devices.push(hint.name.expect("Error: cannot push name"));
            }
        }
        devices
    }
}
