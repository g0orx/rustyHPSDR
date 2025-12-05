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

use std::io::{self, BufReader, BufWriter, BufRead, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::radio::RadioMutex;

const ADDRESS: &str = "127.0.0.1:19001";
const RIG_ID: &str = "019"; // TS-2000

const DEBUG_CAT: bool = false;

#[derive(Debug)]
pub enum CatMessage {
    UpdateMox(bool),
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct CAT {
    pub running: bool,
}

impl CAT {
 
    pub fn new() -> Self {
        let running = false;
        CAT {
            running,
        }
    }

    pub fn run(&mut self, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> io::Result<()> {
if DEBUG_CAT {eprintln!("CAT::run");}
        // lsten for a connection
        let listener = TcpListener::bind(ADDRESS)?;
        let stream = listener.incoming().next().expect("Failed to accept connection")?;
        let reader_stream = stream.try_clone()?;
        let mut writer_stream = stream.try_clone()?;
        let mut reader = BufReader::new(reader_stream);
        let mut writer = BufWriter::new(writer_stream);
        let mut received_data = [0; 1024];
        self.running = true;

        while self.running {
            match reader.read(&mut received_data) {
                Ok(0) => {
                    // client closed
                    eprintln!("cat: client closed connection");
                    self.running = false;
                    }
                Ok(bytes_read) => {
                    let reply = self.parse_commands(&String::from_utf8_lossy(&received_data[..bytes_read]), radio_mutex, tx.clone());
if DEBUG_CAT {eprintln!("CAT::run: reply {:?}", reply);}
                    for i in 0..reply.len() {
                        if reply[i].len() > 0 {
                            writer.write_all(reply[i].as_bytes())?;
                            writer.flush()?;
                        }
                    }
                    }
                Err(e) => {
                    eprintln!("cat error {}", e);
                    self.running = false;
                    }
            }
        }
if DEBUG_CAT {eprintln!("CAT::run: exiting");}
        Ok(())
    }

    pub fn stop(&mut self, radio_mutex: &RadioMutex) {
        self.running = false;
    }

    fn parse_commands(&self, input: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> Vec<String> {
        let cmd = input.trim_end_matches(';').to_uppercase(); // cmd does not include the ;
        let commands: Vec<&str> = input
            .split(';')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim())
            .collect();

        let mut reply = Vec::new();

        for (i, cmd) in commands.iter().enumerate() {
            let command_code = &cmd[..2];
            let suffix = &cmd[2..];
if DEBUG_CAT {eprintln!("CAT::parse_command: {} = {} {}", cmd, command_code, suffix);}
            reply.push(match command_code {
                "AI" => self.AI_cmd(suffix, radio_mutex, tx.clone()),
                "ID" => self.ID_cmd(suffix, radio_mutex, tx.clone()),
                "IF" => self.IF_cmd(suffix, radio_mutex, tx.clone()),
                "FA" => self.FA_cmd(suffix, radio_mutex, tx.clone()),
                "FB" => self.FB_cmd(suffix, radio_mutex, tx.clone()),
                "KS" => self.KS_cmd(suffix, radio_mutex, tx.clone()),
                "MD" => self.MD_cmd(suffix, radio_mutex, tx.clone()),
                "RX" => self.RX_cmd(suffix, radio_mutex, tx.clone()),
                "VX" => self.VX_cmd(suffix, radio_mutex, tx.clone()),
                "TX" => self.TX_cmd(suffix, radio_mutex, tx.clone()),
                "ZZ" => self.parse_zz_command(&cmd, radio_mutex, tx.clone()), // extended commands
                _ => "?;".to_string(), // Unknown command response
            });
        }
        reply
    }

    fn parse_zz_command(&self, command: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        let cmd = &command[..4];
        let suffix = &command[4..];
        "?;".to_string()
    }

    fn AI_cmd(&self, suffix: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        // TODO
        "?;".to_string()
    }

    fn FA_cmd(&self, suffix: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        let mut r = radio_mutex.radio.lock().unwrap();
        let mut reply = "".to_string();
        if suffix == "" {
            // return current frequenxcy
            if r.receiver[0].ctun {
                reply = format!("FA{:011};", r.receiver[0].ctun_frequency);
            } else {
                reply = format!("FA{:011};", r.receiver[0].frequency);
            }
        } else {
            // set the frequency
            let f = suffix.parse::<f32>().unwrap();
            if r.receiver[0].ctun {
                r.receiver[0].ctun_frequency = f;
            } else {
                r.receiver[0].frequency = f;
            }
        }
        reply
    }
 
    fn FB_cmd(&self, suffix: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        let mut r = radio_mutex.radio.lock().unwrap();
        let mut reply = "".to_string();
        if suffix == "" {
            // return current frequenxcy
            if r.receiver[1].ctun {
                reply = format!("FB{:011};", r.receiver[1].ctun_frequency);
            } else {
                reply = format!("FB{:011};", r.receiver[1].frequency);
            }
        } else {
            // set the frequency
            let f = suffix.parse::<f32>().unwrap();
            if r.receiver[1].ctun {
                r.receiver[1].ctun_frequency = f;
            } else {
                r.receiver[1].frequency = f;
            }
        }
        reply
    }
 
    fn ID_cmd(&self, suffix: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        format!("ID{};", RIG_ID)
    }

    fn IF_cmd(&self, suffix: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        
        let r = radio_mutex.radio.lock().unwrap();
        format!("IF{:011}{:04}{:+06}{}{}{}{:02}{}{}{}{}{}{}{:02}{};",
            if r.receiver[0].ctun {
                r.receiver[0].ctun_frequency
            } else {
                r.receiver[0].frequency
            },
            r.receiver[0].step,
            0, // RIT / XIT
            0, // RIT ON/OFF
            0, // XIT ON/OFF
            0, // Memory Bank Number
            0, // Channel Number
            r.is_transmitting() as u8,
            r.receiver[0].mode,
            0, // VFO A/B
            0, // Scan Status
            r.split as u8,
            0, // Tone / CTCSS
            0, // Tone Frequency
            0) // Offset function (FM Mode)
    }

    fn KS_cmd(&self, suffix: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        let r = radio_mutex.radio.lock().unwrap();
        let mut reply = "".to_string();
        if suffix == "" {
            reply = format!("KS{:03};", r.cw_keyer_speed);
        } else {
            // set keyer speed
        }
        reply
    }

    fn MD_cmd(&self, suffix: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        let mut r = radio_mutex.radio.lock().unwrap();
        let mut reply = "".to_string();
        if suffix == "" {
            reply = format!("MD{};", r.receiver[0].mode);
        } else {
            // set the mode
        }
        reply
    }

    fn RX_cmd(&self, suffix: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        let mut r = radio_mutex.radio.lock().unwrap();
        let mut reply = "".to_string();
        r.external_mox = false;
        if tx.send(CatMessage::UpdateMox(false)).is_err() {
            eprintln!("TX_cmd: Main thread receiver was dropped.");
        }
        reply
    }

    fn VX_cmd(&self, suffix: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        let r = radio_mutex.radio.lock().unwrap();
        let mut reply = "".to_string();
        if suffix == "" {
            reply = format!("VX{};", r.vox);
        } else {
            // set vox
        }
        reply
    }

    fn TX_cmd(&self, suffix: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<CatMessage>) -> String {
        let mut r = radio_mutex.radio.lock().unwrap();
        let mut reply = "".to_string();
        r.external_mox = true;
        if tx.send(CatMessage::UpdateMox(true)).is_err() {
            eprintln!("TX_cmd: Main thread receiver was dropped.");
        }
        reply
    }

}

