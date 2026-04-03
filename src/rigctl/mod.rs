/*
    Copyright (C) 2025/2026  John Melton G0ORX/N6LYT

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


const DEBUG_RIGCTL: bool = false;

pub enum RIGCTLMessage {
    ClientConnected(),
    ClientDisconnected(),
    UpdateMox(bool),
    UpdateFrequencyA(f64),
}

impl Default for RIGCTLMessage {
    fn default() -> Self {
        RIGCTLMessage::UpdateMox(false)
    }
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RIGCTL {
    pub address: String,
}

impl RIGCTL {

    pub fn new(network_address: String) -> Self {
if DEBUG_RIGCTL {eprintln!("RIGCTL::new");}
        let address = network_address;
        RIGCTL {
            address,
        }
    }

    pub fn set_address(&mut self, network_address: String) {
        self.address = network_address;
    }

    pub fn run(&mut self, radio_mutex: &RadioMutex, tx: &mpsc::Sender<RIGCTLMessage>, stop_flag: Arc<AtomicBool>) -> io::Result<()> {
if DEBUG_RIGCTL {eprintln!("RIGCTL::run");}
        // lsten for a connection
        let listener = TcpListener::bind(&self.address)?;


        while !stop_flag.load(Ordering::Relaxed) {

if DEBUG_RIGCTL {eprintln!("RIGCTL::listening");}
        let stream = listener.incoming().next().expect("Failed to accept connection")?;
        let reader_stream = stream.try_clone()?;
        let mut writer_stream = stream.try_clone()?;
        let mut reader = BufReader::new(reader_stream);
        let mut writer = BufWriter::new(writer_stream);
        let mut received_data = [0; 1024];

        let _ = tx.send(RIGCTLMessage::ClientConnected());
if DEBUG_RIGCTL {eprintln!("RIGCTL::running");}
        while !stop_flag.load(Ordering::Relaxed) {
            match reader.read(&mut received_data) {
                Ok(0) => {
                    // client closed
                    eprintln!("RIGCTL: client closed connection");
                    let _ = tx.send(RIGCTLMessage::ClientDisconnected());
                    break;
                    }
                Ok(bytes_read) => {
                    let raw_input = String::from_utf8_lossy(&received_data[..bytes_read]);
                    for line in raw_input.lines() {
                        let cmd = line.trim();
                        if cmd.is_empty() { continue; }

                        let reply = self.parse_commands(cmd, radio_mutex, tx.clone());
                        let clean_reply = reply.replace("\r", "");
                        if DEBUG_RIGCTL { eprintln!("RIGCTL::run: cmd={} reply={:?}", line, clean_reply); }
                        if !clean_reply.is_empty() {
                            writer.write_all(clean_reply.as_bytes())?;
                            writer.flush()?;
                        }
                    }
                    }
                Err(e) => {
                    eprintln!("RIGCTL error {}", e);
                    break;
                    }
            }
        }
        }
if DEBUG_RIGCTL {eprintln!("RIGCTL::run: exiting");}
        Ok(())
    }

    fn parse_commands(&self, input: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<RIGCTLMessage>) -> String {
    // line.lines() already stripped the \n, so just trim whitespace
    let cmd = input.trim();
    if cmd.is_empty() { return "".to_string(); }

    let is_extended = cmd.starts_with('\\') || cmd.starts_with('+');
    let clean_cmd = if is_extended { &cmd[1..] } else { cmd };

    match clean_cmd {
        "dump_state" | "_" => {

            format!(
"0
1
2
100000.0 54000000.0 0x1ef 0 -1 0x0 0x0
0 0 0 0 0 0 0
100000.0 54000000.0 0x1ef 0 -1 0x0 0x0
0 0 0 0 0 0 0
0 0
0 0
0
0
0
0


0x0
0x0
0x0
0x0
0x0
0
")
        }
        "get_powerstat" => {
            format!("1\n")
        }
        "get_freq" | "f" => {
            let r = radio_mutex.radio.lock().unwrap();
            let freq = if r.receiver[0].ctun { r.receiver[0].ctun_frequency } else { r.receiver[0].frequency };
            format!("{}\n", freq as u32)
        }
        "get_ptt" | "t" => {
            let r = radio_mutex.radio.lock().unwrap();
            format!("{}\n", if r.is_transmitting() { 1 } else { 0 })
        }
        "get_vfo" | "v" => {
            // return current vfo - currently always VFOA
            "VFOA\n".to_string()
        }
        "set_vfo" | "V" => {
            // set the current vfo - currently always set VFOA
            "RPRT 0\n".to_string()
        }
        c if c.starts_with("set_freq") || c.starts_with('F') => {
            // Use split_whitespace to get the actual number regardless of prefix length
            let val = c.split_whitespace().last().unwrap_or("0");
            if let Ok(f) = val.parse::<f64>() {
                let _ = tx.send(RIGCTLMessage::UpdateFrequencyA(f));
                "RPRT 0\n".to_string()
            } else {
                "RPRT -1\n".to_string()
            }
        }
        c if c.starts_with("set_ptt") || c.starts_with('T') => {
            let val = c.split_whitespace().last().unwrap_or("0");
            let _ = tx.send(RIGCTLMessage::UpdateMox(val == "1"));
            "RPRT 0\n".to_string()
        }
        "chk_vfo" => "CHKVFO 0\n".to_string(),
        _ => "RPRT 0\n".to_string(), // Better to return 0 (Success) than -4 (Error) for stability
    }
}

/*
    fn parse_commands(&self, input: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<RIGCTLMessage>) -> String {
        let cmd = input.trim_end_matches('\n');
if DEBUG_RIGCTL {eprintln!("RIGCTL::parse_commands: cmd={}",cmd);}
        let is_extended = cmd.starts_with("\\");
        let clean_cmd = if is_extended { &cmd[1..] } else { cmd };
if DEBUG_RIGCTL {eprintln!("RIGCTL::parse_commands: clean_cmd={}",clean_cmd);}
        match clean_cmd {
            "dump_state" | "_" => {
                let reply = format!(
"0
1
2
100000
54000000
0x1ef
0
-1
0x0
0x0
");
                reply 
            }
            "get_powerstat" => {
                let reply = format!(
"1
");
                reply
            }
            "get_freq" | "f" => {
                let mut r = radio_mutex.radio.lock().unwrap();
                let mut reply = "".to_string();
                // return current frequency
                if r.receiver[0].ctun {
                    reply = format!(
"{}
", r.receiver[0].ctun_frequency as u32);
                } else {
                    reply = format!(
"{}
", r.receiver[0].frequency as u32);
                }
                reply
            }
            "get_ptt" | "t" => {
                let mut r = radio_mutex.radio.lock().unwrap();
                let mut reply = "".to_string();

                reply = format!(
"{}
", r.is_transmitting());
                reply
            }
            "get_vfo" | "v" => {
                let mut r = radio_mutex.radio.lock().unwrap();
                let mut reply = "".to_string();

                reply = format!(
"VFOA
");
                reply
            }
            c if c.starts_with("set_freq") || c.starts_with('F') => {
                let r = radio_mutex.radio.lock().unwrap();
                let mut reply = "".to_string();
                let val = c.split_whitespace().last().unwrap_or("0");
                if let Ok(f) = c[2..].trim().parse::<f64>() {
                    if tx.send(RIGCTLMessage::UpdateFrequencyA(f)).is_err() {
                        eprintln!("TX_cmd: Main thread receiver was dropped.");
                    }
                    reply =
"RPRT 0
".to_string();
                }
                reply
            }
            c if c.starts_with("set_ptt") || c.starts_with('T') => {
                let r = radio_mutex.radio.lock().unwrap();
                let mut reply = "".to_string();
                let val = c.split_whitespace().last().unwrap_or("0");
                if tx.send(RIGCTLMessage::UpdateMox(val=="1")).is_err() {
                    eprintln!("TX_cmd: Main thread receiver was dropped.");
                }
                reply =
"RPRT 0
".to_string();
                reply
            }
            _ => { // unimplemented command
                let reply = format!(
"RPRT -4
");   // unimplemented
                reply
            }
        }
    }
*/

}
