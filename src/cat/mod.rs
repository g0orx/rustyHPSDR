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

use std::io::{self, BufReader, BufWriter, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self};
use serde::{Deserialize, Serialize};
use crate::radio::RadioMutex;

const RIG_ID: &str = "019"; // Kenwood TS-2000
const DEBUG_CAT: bool = false;

pub enum CatMessage {
    UpdateMox(bool),
    UpdateFrequencyA(),
    UpdateFrequencyB(),
}

impl Default for CatMessage {
    fn default() -> Self {
        CatMessage::UpdateFrequencyA()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CAT {
    pub address: String,
}

impl CAT {
 
    pub fn new(network_address: String) -> Self {
if DEBUG_CAT {eprintln!("CAT::new");}
        let address = network_address;
        CAT {
            address,
        }
    }

    pub fn set_address(&mut self, network_address: String) {
        self.address = network_address;
    }

    pub fn run(&mut self, radio_mutex: &RadioMutex, tx: &mpsc::Sender<CatMessage>, stop_flag: Arc<AtomicBool>) -> io::Result<()> {
if DEBUG_CAT {eprintln!("CAT::run");}
        // lsten for a connection
        let listener = TcpListener::bind(&self.address)?;


        while !stop_flag.load(Ordering::Relaxed) {

if DEBUG_CAT {eprintln!("CAT::listening");}
        let stream = listener.incoming().next().expect("Failed to accept connection")?;
        let reader_stream = stream.try_clone()?;
        let mut writer_stream = stream.try_clone()?;
        let mut reader = BufReader::new(reader_stream);
        let mut writer = BufWriter::new(writer_stream);
        let mut received_data = [0; 1024];

if DEBUG_CAT {eprintln!("CAT::running");}
        while !stop_flag.load(Ordering::Relaxed) {
            match reader.read(&mut received_data) {
                Ok(0) => {
                    // client closed
                    eprintln!("cat: client closed connection");
                    break;
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
                    break;
                    }
            }
        }
        }
if DEBUG_CAT {eprintln!("CAT::run: exiting");}
        Ok(())
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
            let f = suffix.parse::<f64>().unwrap();
            if r.receiver[0].ctun {
                r.receiver[0].ctun_frequency = f;
            } else {
                r.receiver[0].frequency = f;
            }
            if tx.send(CatMessage::UpdateFrequencyA()).is_err() {
                eprintln!("TX_cmd: Main thread receiver was dropped.");
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
            let f = suffix.parse::<f64>().unwrap();
            if r.receiver[1].ctun {
                r.receiver[1].ctun_frequency = f;
            } else {
                r.receiver[1].frequency = f;
            }
            if tx.send(CatMessage::UpdateFrequencyB()).is_err() {
                eprintln!("TX_cmd: Main thread receiver was dropped.");
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
