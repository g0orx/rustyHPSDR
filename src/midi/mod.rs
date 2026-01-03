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
use std::error::Error;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self};
use serde::{Deserialize, Serialize};

use midir::{Ignore, MidiInput, MidiInputPort, MidiIO};
use crate::radio::RadioMutex;


#[derive(Clone, Debug)]
enum MidiAction {
    MidiNone,
    MidiVfoA,
    MidiVfoB,
}

#[derive(Clone, Debug)]
pub enum MidiMessage {
    UpdateMox(bool),
    StepFrequencyA(i32),
    StepFrequencyB(i32),
}

impl Default for MidiMessage {
    fn default() -> Self {
        MidiMessage::StepFrequencyA(0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MidiType {
    MidiUnknown,
    MidiButton, // NOTE ON/OFF
    MidiKnob,   //  Value between 0 and 127
    MidiWheel,  // direction and speed
}

#[derive(Clone, Debug)]
pub struct MidiFunction {
    midi_type: MidiType,
    midi_action: MidiAction,
}

impl Default for MidiFunction {
    fn default() -> Self {
        Self {
            midi_type: MidiType::MidiUnknown,
            midi_action: MidiAction::MidiNone,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MIDI {
    pub device: String,
    pub functions: Vec<MidiFunction>,
}

impl MIDI {

    pub fn new(d: String) ->Self {
eprintln!("MIDI::new {}", d);
        let device = d;
        let functions = vec![MidiFunction::default(); 256];
        MIDI {
            device,
            functions,
        }
    }

    pub fn run(&self, radio_mutex: &RadioMutex, tx: &mpsc::Sender<MidiMessage>, stop_flag: Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {

eprintln!("MIDI::run {}", self.device);
        let midi_in = MidiInput::new("rustyHPSDR")?;
        //midi_in.ignore(Ignore::None);
        let in_ports = midi_in.ports();
        let mut port_index = 999;
        for (i, p) in in_ports.iter().enumerate() {
            println!("{}", midi_in.port_name(p).unwrap());
            if midi_in.port_name(p).unwrap() == self.device {
                port_index = i;
            }
        }
        if port_index == 999 {
            eprintln!("Could not find midi port {}", self.device);
        } else {
            let in_port = &in_ports[port_index];
            let functions = self.functions.clone();
            let tx_clone = tx.clone();
            let _conn_in = midi_in.connect(
                    &in_port,
                    "rustyHPSDR-read-input",
                    move |_, message, _| {
                        // message[0] is the function
                        // message[1] is the id
                        // message[2] is the value (note that for Note On/Off it is 127 for On and 0 for Off
                        eprintln!("{} {} {}", message[0], message[1], message[2]);
                        let index = message[1] as usize;
                        match message[0] & 0xF0 {
                            0x80 => {
                                        eprintln!("Note OFF {}", message[1]);
                                        if functions[index].midi_type != MidiType::MidiUnknown {
                                            eprintln!("{:?}", functions[index]);
                                        } else {
                                            eprintln!("Unknown!");
                                        }
                                    },
                            0x90 => {
                                        eprintln!("Note ON {}", message[1]);
                                        if functions[index].midi_type != MidiType::MidiUnknown {
                                            eprintln!("{:?}", functions[index]);
                                        } else {
                                            eprintln!("Unknown!");
                                        }
                                    },
                            0xB0 => {
                                        eprintln!("Control {}", message[2]);
                                        eprintln!("scroll {}", message[2] as i32 - 64);
                                        if functions[index].midi_type != MidiType::MidiUnknown {
                                            eprintln!("{:?}", functions[index]);
                                        } else {
                                            eprintln!("Unknown!");
                                            if tx_clone.send(MidiMessage::StepFrequencyA(message[2] as i32 - 64)).is_err() {
                                                eprintln!("TX_cmd: Main thread receiver was dropped.");
                                            }
                                        }
                                    },
                            _ => {
                                 },
                        }
                        
                    },
                    (),
                )?;
             while !stop_flag.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_secs(1));
             }
        }

        Ok(())
        
    }

}
