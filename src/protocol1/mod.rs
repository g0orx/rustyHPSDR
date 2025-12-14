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

/*
    Implements the Open HPSDR - USB Data Protocol (v1.60) over Ethernet.

    Supports sample rates of 48/96/192/384kHz.
    Microphone, Audio and I/Q transmit data is at 48kHz.

    Note that the Microphone sample rate is actually at the receive I/Q sample
    rate but each sample is repeated to fit the I/Q sample rate. The replicated
    samples are ignored.
*/

use nix::sys::socket::setsockopt;
use nix::sys::socket::sockopt::{ReuseAddr, ReusePort};
use std::net::{UdpSocket};

use crate::antenna::Antenna;
use crate::audio::Audio;
use crate::receiver::AudioOutput;
use crate::discovery::Device;
use crate::modes::Modes;
use crate::radio::RadioMutex;

const OZY_BUFFER_SIZE: usize = 512;
const METIS_BUFFER_SIZE: usize = (OZY_BUFFER_SIZE * 2) + 8;
const SYNC: u8  = 0x7F;

pub struct Protocol1 {
    device: Device,
    socket: UdpSocket,
    receive_sequence: u32,
    send_sequence: u32,
    wide_sequence: u32,
    receivers: u8,
    current_receiver: u8,
    iq_samples: i32,
    n_samples: i32,
    left_sample: i32,
    right_sample: i32,
    ozy_buffer: Vec<u8>,
    ozy_buffer_offset: usize,
    ozy_command: u8,
    metis_buffer: Vec<u8>,
    metis_buffer_offset: usize,
    rx_audio: Vec<Audio>,
    tx_audio: Audio,
}

impl Protocol1 {

    pub fn new(device: Device) -> Protocol1 {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("bind failed");
        setsockopt(&socket, ReusePort, &true).unwrap();
        setsockopt(&socket, ReuseAddr, &true).unwrap();

        let receive_sequence: u32 = 0;
        let send_sequence: u32 = 0;
        let wide_sequence: u32 = 0;
        let receivers: u8 = 2; 
        let current_receiver: u8 = 0;
        let iq_samples: i32 = (512 - 8)/((receivers as i32 * 6) + 2);
        let n_samples: i32 = 0;
        let left_sample: i32 = 0;
        let right_sample: i32 = 0;
        let ozy_buffer: Vec<u8> = vec![0; OZY_BUFFER_SIZE];
        let ozy_buffer_offset: usize = 8;
        let ozy_command: u8 = 1;
        let metis_buffer: Vec<u8> = vec![0; METIS_BUFFER_SIZE];
        let metis_buffer_offset: usize = 8;

        let mut rx_audio: Vec<Audio> = Vec::new();
        for _i in 0..receivers {
            rx_audio.push(Audio::new());
        }
        let tx_audio: Audio = Audio::new();

        

        Protocol1{device,
                           socket,
                           receive_sequence,
                           send_sequence,
                           wide_sequence,
                           receivers,
                           current_receiver,
                           n_samples,
                           iq_samples,
                           left_sample,
                           right_sample,
                           ozy_buffer,
                           ozy_buffer_offset,
                           ozy_command,
                           metis_buffer,
                           metis_buffer_offset,
                           rx_audio,
                           tx_audio,
                          }
    }

    pub fn run(&mut self, radio_mutex: &RadioMutex) {

        // setup local audio nput and output if configured
        let mut r = radio_mutex.radio.lock().unwrap();
        if r.receiver[0].local_output {
            let _ = self.rx_audio[0].open_output(&r.receiver[0].output_device);
        }
        if r.receiver[1].local_output {
            let _ = self.rx_audio[1].open_output(&r.receiver[1].output_device);
        }
        if r.transmitter.local_input {
            match self.tx_audio.open_input(&r.transmitter.input_device) {
                Ok(_) => {
                    },
                Err(e) => {
                    eprintln!("Error opening input: {}", e);
                    },
             }
        }
        drop(r);

        // start the radio running
        loop {
            if self.device.device == 6 {
                self.send_ozy_buffer(radio_mutex, 0);
            } else {
                self.send_ozy_buffer(radio_mutex, 0);
            }
            if self.ozy_command == 1 {
                break;
            }
        }
        loop {
            if self.device.device == 6 {
                self.send_ozy_buffer(radio_mutex, 0);
            } else {
                self.send_ozy_buffer(radio_mutex, 0);
            }
            if self.ozy_command == 1 {
                break;
            }
        }

        self.metis_start();
        let mut buffer = vec![0; 2048];
        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((_size, src)) => {
                    match src.port() {
                        1024 => {
                                if buffer[0] == 0xEF && buffer[1] == 0xFE {
                                    let _seq = u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
                                    match buffer[2] {
                                        1 => {
                                             match buffer[3] {
                                                 6 => { // IQ samples
                                                      self.process_ozy_buffer(&buffer,8,radio_mutex);
                                                      self.process_ozy_buffer(&buffer,520,radio_mutex);
                                                      },
                                                 4 => { // Wideband samples
                                                      },
                                                 _ => println!("Unexpected EP {}", buffer[3]),
                                             }
                                             },
                                        _ => println!("Unexpected packet type {}", buffer[2]),
                                    }
                                } else {
                                    println!("Received bad header bytes");
                                }
                                },
                        _ => println!("Unknown port {}", src.port()),
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving UDP packet: {}", e);
                }
            }

            // check for local microphone
            let mut r = radio_mutex.radio.lock().unwrap();
            if r.transmitter.local_input && !r.transmitter.local_input_changed && !r.tune {
                //r.transmitter.microphone_samples = 0;
                let (mic_buffer, count) = self.tx_audio.read_input();
                if count != 0 {
                    for i in 0..mic_buffer.len() {
                        if r.transmitter.add_mic_sample(mic_buffer[i]) && r.is_transmitting() {
                            for j in 0..r.transmitter.output_samples {
                                // Dummy RX Audio samples
                                self.ozy_buffer[self.ozy_buffer_offset] = 0;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = 0;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = 0;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = 0;
                                self.ozy_buffer_offset += 1; 

                                // TX IQ samples
                                let ix = j * 2;
                                let i_sample: i16 = (r.transmitter.iq_buffer[ix as usize] * 32767.0) as i16;
                                let q_sample: i16 = (r.transmitter.iq_buffer[(ix+1) as usize]* 32767.0)  as i16;
                                self.ozy_buffer[self.ozy_buffer_offset] = ((i_sample >> 8) & 0xFF) as u8;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = (i_sample & 0xFF) as u8;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = ((q_sample >> 8) & 0xFF) as u8;
                                self.ozy_buffer_offset += 1;
                                self.ozy_buffer[self.ozy_buffer_offset] = (q_sample & 0xFF) as u8;
                                self.ozy_buffer_offset += 1;
    
                                if self.ozy_buffer_offset == OZY_BUFFER_SIZE {
                                    drop(r);
                                    self.send_ozy_buffer(radio_mutex, 0);
                                    self.ozy_buffer_offset = 8;
                                    r = radio_mutex.radio.lock().unwrap();
                                }
                            }
                        }
                    }
                }
            }

            // check for any changes that we need to handle here
            let sample_rate_changed = r.sample_rate_changed;
            r.sample_rate_changed = false;

            let rx1_local_output_changed = r.receiver[0].local_output_changed;
            let rx1_local_output_changed_to = r.receiver[0].local_output_changed_to;
            let rx1_local_output_device_changed = r.receiver[0].local_output_device_changed;
            let rx1_output_device = r.receiver[0].output_device.clone();
            let rx1_local_output = r.receiver[0].local_output;
            r.receiver[0].local_output_device_changed = false;

            let rx2_local_output_changed = r.receiver[1].local_output_changed;
            let rx2_local_output_changed_to = r.receiver[1].local_output_changed_to;
            let rx2_local_output_device_changed = r.receiver[1].local_output_device_changed;
            let rx2_local_output = r.receiver[1].local_output;
            let rx2_output_device = r.receiver[1].output_device.clone();
            let rx2_local_output = r.receiver[1].local_output;
            r.receiver[1].local_output_device_changed = false;

            let local_input = r.transmitter.local_input;
            let input_changed = r.transmitter.local_input_changed;
            let input_device_changed = r.transmitter.input_device_changed;
            let input_device = r.transmitter.input_device.clone();
            r.transmitter.local_input_changed = false;
            r.transmitter.input_device_changed = false;
            drop(r);
            if sample_rate_changed {
                self.metis_stop();
                // receiver change sample rate
                // transmitter change mic sample rate
                self.metis_start();
                // PS set sample rate
            }
            if rx1_local_output_changed {
                if rx1_local_output_changed_to {
                    let _ = self.rx_audio[0].open_output(&rx1_output_device);
                    let mut r = radio_mutex.radio.lock().unwrap();
                    r.receiver[0].local_output_changed = false;
                    r.receiver[0].local_output = true;
                } else {
                    let mut r = radio_mutex.radio.lock().unwrap();
                    r.receiver[0].local_output_changed = false;
                    r.receiver[0].local_output = false;
                    let _ = self.rx_audio[0].close_output();
                }
            }
            if rx1_local_output_device_changed && rx1_local_output {
                let _ = self.rx_audio[0].close_output();
                let _ = self.rx_audio[0].open_output(&rx1_output_device);
            }
            if rx2_local_output_changed {
                if rx2_local_output_changed_to {
                    let _ = self.rx_audio[1].open_output(&rx2_output_device);
                    let mut r = radio_mutex.radio.lock().unwrap();
                    r.receiver[1].local_output_changed = false;
                    r.receiver[1].local_output = true;
                } else {
                    let mut r = radio_mutex.radio.lock().unwrap();
                    r.receiver[1].local_output_changed = false;
                    r.receiver[1].local_output = false;
                    let _ = self.rx_audio[1].close_output();
                }
            }
            if rx2_local_output_device_changed && rx2_local_output {
                let _ = self.rx_audio[1].close_output();
                let _ = self.rx_audio[1].open_output(&rx2_output_device);
            }
            if input_changed {
                if local_input {
                    let _ = self.tx_audio.open_input(&input_device);
                } else {
                    let _ = self.tx_audio.close_input();
                }
            }
            if input_device_changed && local_input {
                let _ = self.tx_audio.close_input();
                let _ = self.tx_audio.open_input(&input_device);
            }
        }
    }

    fn process_ozy_buffer(&mut self, buffer: &Vec<u8>, offset: usize, radio_mutex: &RadioMutex)  {
        let mut r = radio_mutex.radio.lock().unwrap();
        let mic_sample_divisor = r.sample_rate / 48000;

        /*
        let mut c0: u8 = 0;
        let mut c1: u8 = 0;
        let mut c2: u8 = 0;
        let mut c3: u8 = 0;
        let mut c4: u8 = 0;
        */
        let mut i_sample = 0;
        let mut q_sample = 0;
        let mut mic_samples = 0;
        let mut mic_sample:f32 = 0.0;
        let mut b = offset;

        let mut process_rx_audio: bool = false;
        let mut process_tx_iq: bool = false;

        if buffer[b]==SYNC && buffer[b+1]==SYNC && buffer[b+2]==SYNC {
            b += 3;
        } else {
            eprintln!("SYNC error");
            drop(r);
            self.metis_stop();
            loop {
                if self.device.device == 6 {
                    self.send_ozy_buffer(radio_mutex, 0);
                } else {
                    self.send_ozy_buffer(radio_mutex, 0);
                }
                if self.ozy_command == 1 {
                    break;
                }
            }
            self.metis_start();
            return;
        }
        // collect the control bytes
        let c0 = buffer[b];
        b += 1;
        let c1 = buffer[b];
        b += 1;
        let c2 = buffer[b];
        b += 1;
        let c3 = buffer[b];
        b += 1;
        let c4 = buffer[b];
        b += 1;

        let previous_ptt = r.ptt;
        let previous_dot = r.dot;
        let previous_dash = r.dash;
        r.ptt = (c0 & 0x01) == 0x01;
        r.dash = (c0 & 0x02) == 0x02;
        r.dot = (c0 & 0x04) == 0x04;

        if r.ptt != previous_ptt || r.dot != previous_dot || r.dash != previous_dash {
eprintln!("ptt {} dot {} dash {}", r.ptt, r.dot, r.dash);
            r.set_state();
        }

        let address = (c0>>3) & 0x1F;
        match address {
            0 => {
                }
            1 => {
                r.transmitter.exciter_power=u16::from_be_bytes([c1,c2]);
                r.transmitter.alex_forward_power=u16::from_be_bytes([c3,c4]);
                }
            2 => {
                r.transmitter.alex_reverse_power=u16::from_be_bytes([c1,c2]);
                }
            3 => {
                }
            4 => {
                }
            _ => {
                eprintln!("Invalid address from Radio {}", address);
                }
        }

        // colleact the RX IQ samples and MIC Audio Samples
        for _s in 0..self.iq_samples {
            // IQ samples for each receiver
            for rx in 0..r.receivers {
                let ddc = rx as usize;
                if buffer[b] & 0x80 != 0 {
                    i_sample = u32::from_be_bytes([0xFF, buffer[b], buffer[b+1], buffer[b+2]]) as i32;
                } else {
                    i_sample = u32::from_be_bytes([0, buffer[b], buffer[b+1], buffer[b+2]]) as i32;
                }
                b += 3;
                if buffer[b] & 0x80 != 0 {
                    q_sample = u32::from_be_bytes([0xFF, buffer[b], buffer[b+1], buffer[b+2]]) as i32;
                } else {
                    q_sample = u32::from_be_bytes([0, buffer[b], buffer[b+1], buffer[b+2]]) as i32;
                }
                b += 3;

                let i = r.receiver[ddc].samples*2;
                r.receiver[ddc].iq_input_buffer[i]=i_sample as f64/16777215.0;
                r.receiver[ddc].iq_input_buffer[i+1]=q_sample as f64/16777215.0;
                r.receiver[ddc].samples += 1;
                if r.receiver[ddc].samples >= r.receiver[ddc].buffer_size {
                    r.receiver[ddc].process_iq_samples();
                    r.receiver[ddc].samples = 0;
                    process_rx_audio = true;
                }
            }
            // MIC Audio samples - always collect them and sync with local audio input if enabled
            mic_sample = (i16::from_be_bytes([buffer[b], buffer[b+1]])) as f32/ 32767.0;
            b += 2;

            // discard replicated samples
            if mic_samples == 0 {
                if r.transmitter.add_mic_sample(mic_sample) && r.is_transmitting() {
                    if !r.transmitter.local_input || r.tune {


                        for j in 0..r.transmitter.output_samples {
                            // Dummy RX Audio samples
                            self.ozy_buffer[self.ozy_buffer_offset] = 0;
                            self.ozy_buffer_offset += 1;
                            self.ozy_buffer[self.ozy_buffer_offset] = 0;
                            self.ozy_buffer_offset += 1;
                            self.ozy_buffer[self.ozy_buffer_offset] = 0;
                            self.ozy_buffer_offset += 1;
                            self.ozy_buffer[self.ozy_buffer_offset] = 0;
                            self.ozy_buffer_offset += 1;

                            // TX IQ samples
                            let ix = j * 2;
                            let i_sample: i16 = (r.transmitter.iq_buffer[ix as usize] * 32767.0) as i16;
                            let q_sample: i16 = (r.transmitter.iq_buffer[(ix+1) as usize]* 32767.0)  as i16;
                            self.ozy_buffer[self.ozy_buffer_offset] = ((i_sample >> 8) & 0xFF) as u8;
                            self.ozy_buffer_offset += 1;
                            self.ozy_buffer[self.ozy_buffer_offset] = (i_sample & 0xFF) as u8;
                            self.ozy_buffer_offset += 1;
                            self.ozy_buffer[self.ozy_buffer_offset] = ((q_sample >> 8) & 0xFF) as u8;
                            self.ozy_buffer_offset += 1;
                            self.ozy_buffer[self.ozy_buffer_offset] = (q_sample & 0xFF) as u8;
                            self.ozy_buffer_offset += 1;

                            if self.ozy_buffer_offset == OZY_BUFFER_SIZE {
                                drop(r);
                                self.send_ozy_buffer(radio_mutex, 0);
                                self.ozy_buffer_offset = 8;
                                r = radio_mutex.radio.lock().unwrap();
                            }
                        }
                    }
                }
            }
        }

        // full RX audio buffers
        let output_samples = r.receiver[0].output_samples;
        // check each receiver has same output samples?
        if process_rx_audio {
            if !r.is_transmitting() {
                for i in 0..output_samples {
                    let ix = i * 2;
                    let mut left_sample: f32 = 0.0;
                    let mut right_sample: f32 = 0.0;
                    for  rx in 0..r.receivers {
                        if rx == 0 || (rx == 1 && r.rx2_enabled) {
                        match r.receiver[rx as usize].audio_output {
                            AudioOutput::Stereo | AudioOutput::Left => {
                                left_sample += r.receiver[rx as usize].audio_buffer[ix] as f32;
                                }
                            AudioOutput::Right | AudioOutput::Mute => {
                                //left_sample = left_sample + 0;
                                }
                        }
                        match r.receiver[rx as usize].audio_output {
                            AudioOutput::Stereo | AudioOutput::Right => {
                                right_sample += r.receiver[rx as usize].audio_buffer[ix+1] as f32;
                                }
                            AudioOutput::Left | AudioOutput::Mute => {
                                //right_sample = right_sample + 0;
                                }
                        }

                        if left_sample > f32::MAX as f32 {
                            left_sample = f32::MAX as f32;
                        } else if left_sample < f32::MIN as f32 {
                            left_sample = f32::MIN as f32;
                        }

                        if right_sample > f32::MAX as f32 {
                            right_sample = f32::MAX as f32;
                        } else if right_sample < f32::MIN as f32 {
                            right_sample = f32::MIN as f32;
                        }

                        if r.receiver[rx as usize].local_output {
                            let lox=r.receiver[rx as usize].local_audio_buffer_offset * 2;
                            match r.receiver[rx as usize].audio_output {
                                AudioOutput::Stereo => {
                                    r.receiver[rx as usize].local_audio_buffer[lox]=left_sample as f32;
                                    r.receiver[rx as usize].local_audio_buffer[lox+1]=right_sample as f32;
                                },
                                AudioOutput::Left => {
                                    r.receiver[rx as usize].local_audio_buffer[lox]=left_sample as f32;
                                    r.receiver[rx as usize].local_audio_buffer[lox+1]=0.0;
                                },
                                AudioOutput::Right => {
                                    r.receiver[rx as usize].local_audio_buffer[lox]=0.0;
                                    r.receiver[rx as usize].local_audio_buffer[lox+1]=right_sample as f32;
                                },
                                AudioOutput::Mute => {
                                    r.receiver[rx as usize].local_audio_buffer[lox]=0.0;
                                    r.receiver[rx as usize].local_audio_buffer[lox+1]=0.0;
                                },
                            }
                            r.receiver[rx as usize].local_audio_buffer_offset += 1;
                            if r.receiver[rx as usize].local_audio_buffer_offset == r.receiver[rx as usize].local_audio_buffer_size {
                                r.receiver[rx as usize].local_audio_buffer_offset = 0;
                                let buffer_clone = r.receiver[rx as usize].local_audio_buffer.clone();
                                let _ = self.rx_audio[rx as usize].write_output(&buffer_clone);
                            }
                        }
                        }
                    }

                    let left = (left_sample * 32767.0) as i16;
                    let right = (right_sample * 32767.0) as i16;

                    self.ozy_buffer[self.ozy_buffer_offset] = (left >> 8) as u8;
                    self.ozy_buffer_offset += 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = left as u8;
                    self.ozy_buffer_offset += 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = (right >> 8) as u8;
                    self.ozy_buffer_offset += 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = right as u8;
                    self.ozy_buffer_offset += 1;

                    // TX IQ samples
                    self.ozy_buffer[self.ozy_buffer_offset] = 0;
                    self.ozy_buffer_offset += 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = 0;
                    self.ozy_buffer_offset += 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = 0;
                    self.ozy_buffer_offset += 1;
                    self.ozy_buffer[self.ozy_buffer_offset] = 0;
                    self.ozy_buffer_offset += 1;

                    if self.ozy_buffer_offset == OZY_BUFFER_SIZE {
                        drop(r);
                        self.send_ozy_buffer(radio_mutex, 0);
                        self.ozy_buffer_offset = 8;
                        r = radio_mutex.radio.lock().unwrap();
                    }
                }
            }
            process_rx_audio = false;
        }
    }

    fn send_ozy_buffer(&mut self, radio_mutex: &RadioMutex, rx: i32) {
        let mut c0: u8 = 0x00;
        let mut c1: u8 = 0x00;
        let mut c2: u8 = 0x00;
        let mut c3: u8 = 0x00;
        let mut c4: u8 = 0x00;

        let r = radio_mutex.radio.lock().unwrap();
        let mut frequency = r.receiver[0].frequency;
        if r.receiver[0].ctun {
            frequency = r.receiver[0].ctun_frequency;
        }
        let mut frequency_b = r.receiver[1].frequency;
        if r.receiver[1].ctun {
            frequency_b = r.receiver[1].ctun_frequency;
        }

        let mut b = r.receiver[rx as usize].band.to_usize();
        let mut attenuation = r.receiver[rx as usize].band_info[b].attenuation;
        if r.dev == 6 {
            b = r.receiver[0].band.to_usize();
            attenuation = r.receiver[0].band_info[b].attenuation;
        }

        if self.metis_buffer_offset == 8 {
            c0 = 0x00;
            c1 = match r.receiver[0].sample_rate {
                48000 => {0x00},
                96000 => {0x01},
                192000 => {0x02},
                384000 => {0x03},
                _ => {00}, 
            };
            c2 = 0x00; // TODO Class E and OC
            c3 = match r.receiver[0].band_info[b].antenna {
                Antenna::ANT1 => 0x00, // ANT 1
                Antenna::ANT2 => 0x01, // ANT 2
                Antenna::ANT3 => 0x02, // ANT 3
                Antenna::EXT1 => 0x00, // EXT 1
                Antenna::EXT2 => 0x00, // EXT 2
                Antenna::XVTR => 0x00, // XVTR
                _ => 0x00, // None
            };
            b = if r.split {
                    r.receiver[1].band.to_usize()
                } else {
                    r.receiver[0].band.to_usize()
                };
            let tx_antenna = if r.split {
                                 r.receiver[1].band_info[b].tx_antenna
                             } else {
                                 r.receiver[0].band_info[b].tx_antenna
                             };
            c4 = match tx_antenna {
                Antenna::ANT1 => 0x00, // ANT 1
                Antenna::ANT2 => 0x01, // ANT 2
                Antenna::ANT3 => 0x02, // ANT 3
                _ => 0x00,
            };
            c4 |= (self.receivers - 1) << 3;

        } else {
            match self.ozy_command {
                1 => {
                    c0 = 0x02; // C0
                    // TX frequency
                    let mut f = if r.split {
                                    frequency_b as i32
                                } else {
                                    frequency as i32
                                };
                    f = f - r.receiver[0].band_info[b].lo as i32;
                    f = f + r.receiver[0].band_info[b].lo_error as i32;
                    c1 = (f >> 24) as u8; // C1
                    c2 = (f>>16) as u8; // C2
                    c3 = (f>>8) as u8; // C3
                    c4 = f as u8; // C4
                     },
                2 => {
                    c0 = 0x04 + (self.current_receiver * 2); // C0
                    // RX frequency
                    let mut f = r.receiver[self.current_receiver as usize].frequency as i32;
                    f = f - r.receiver[0].band_info[b].lo as i32;
                    f = f + r.receiver[0].band_info[b].lo_error as i32;
                    c1 = (f >> 24) as u8; // C1
                    c2 = (f>>16) as u8; // C2
                    c3 = (f>>8) as u8; // C3
                    c4 = f as u8; // C4
                    self.current_receiver += 1;
                    if self.current_receiver >= self.receivers {
                        self.current_receiver = 0;
                    }
                     },
                3 => {
                    c0 = 0x12; // C0
                    c1 = 0x00; // C1 // drive level 0..255
                    if r.is_transmitting() {
                        let mut b = r.receiver[0].band.to_usize();
                        if r.split {
                            b = r.receiver[1].band.to_usize();
                        }

                        let power = r.transmitter.drive;

                        let mut target_dbm = 10.0 * ((power * 1000.0).log10());
                        let gbb = r.transmitter.pa_calibration[b];
                        target_dbm -= gbb;
                        let target_volts = (10.0_f32.powf(target_dbm * 0.1) * 0.05).sqrt();
                        let volts=(target_volts / 0.8).min(1.0);
                        let mut actual_volts=volts*(1.0/0.98);

                        if actual_volts<0.0 {
                          actual_volts=0.0;
                        } else if actual_volts>1.0 {
                          actual_volts=1.0;
                        }

                        let level=(actual_volts*255.0) as u8;

                        //c1 = ((r.transmitter.drive/100.0) * 255.0) as u8;
                        c1 = level;
                    }
                    c2 = 0x00; // C2
                    if r.mic_boost {
                        c2 |= 0x01;
                    }
                    if self.device.device == 6 { // HERMES_LITE
                        if self.device.version > 42 { // HERMES_LITE_2
                            c2 |= 0x2C; // PA
                            if r.tune {
                                c2 |= 0x10;
                            }
                        }
                    }
                    c3 = 0x00; // C3
                    c4 = 0x00; // C4
                     },
                4 => {
                    c0 = 0x14; // C0
                    c1 = 0x00; // C1 // preamp adc 0
                    if r.mic_ptt {
                        c1 |= 0x40;
                    }
                    if r.mic_bias_enable {
                        c1 |= 0x20;
                    }
                    if r.mic_bias_ring {
                        c1 |= 0x10;
                    }
                    c2 = 0x00; // C2
                    c3 = 0x00; // C3
                    c4 = 0x00; // C4
                    if self.device.device == 6 { // HERMES_LITE
                        if self.device.version > 42 { // HERMES_LITE_2
                            c4 |= 0x40;
                            c4 |= ((attenuation  + 12) & 0x3F) as u8;
                        }  else { // HERMES_LITE_1
                            c4 |= 0x20;
                            c4 |= ((attenuation  + 12) & 0x1F) as u8;
                        }
                    } else {
                        //if attenuation != 0 {
                            c4 = 0x20;
                            c4 |= (attenuation & 0x1F) as u8;
                        //}
                    }
                     },
                5 => {
                    c0 = 0x16; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    if r.cw_keys_reversed {
                        c2 |= 0x40;
                    }
                    c3 = r.cw_keyer_speed as u8 | ((r.cw_keyer_mode.to_u32() as u8) << 6);
                    c4 = r.cw_keyer_weight as u8 | ((r.cw_keyer_spacing as u8) << 7);
                     },
                6 => {
                    c0 = 0x1C; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    c3 = 0x00; // C3
                    c4 = 0x00; // C4
                     },
                7 => {
                    c0 = 0x1E; // C0
                    c1 = 0x00; // C1
                    if (r.transmitter.mode == Modes::CWL.to_usize() || r.transmitter.mode == Modes::CWU.to_usize()) && r.is_transmitting() {
                        c1 |= 0x01;
                    }
                    c2 = r.cw_keyer_sidetone_volume as u8; // C2
                    c3 = r.cw_keyer_ptt_delay as u8; // C3
                    c4 = 0x00; // C4
                     },
                8 => {
                    c0 = 0x20; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    c3 = 0x28; // C3
                    c4 = 0x0A; // C4
                     },
                9 => {
                    c0 = 0x22; // C0
                    c1 = 0x19; // C1
                    c2 = 0x00; // C2
                    c3 = 0xC8; // C3
                    c4 = 0x00; // C4
                     },
                10 => {
                    c0 = 0x24; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    c3 = 0x00; // C3
                    c4 = 0x00; // C4
                     },
                11 => {
                    c0 = 0x2E; // C0
                    c1 = 0x00; // C1
                    c2 = 0x00; // C2
                    c3 = 0x04; // C3
                    c4 = 0x15; // C4
                     },
                _ => eprintln!("Invalid command {}", self.ozy_command),
            }
            if self.current_receiver == 0 {
                self.ozy_command += 1;
                if self.ozy_command > 11 {
                    self.ozy_command = 1;
                }
            }
        }

        if r.is_transmitting() {
            c0 |= 0x01;
        }

        self.ozy_buffer[0] = SYNC;
        self.ozy_buffer[1] = SYNC;
        self.ozy_buffer[2] = SYNC;
        self.ozy_buffer[3] = c0;
        self.ozy_buffer[4] = c1;
        self.ozy_buffer[5] = c2;
        self.ozy_buffer[6] = c3;
        self.ozy_buffer[7] = c4;

        self.metis_write();

    }

    fn metis_write(&mut self) {
        // copy the buffer
        for i in 0..512 {
            self.metis_buffer[i+self.metis_buffer_offset] = self.ozy_buffer[i];
        }

        if self.metis_buffer_offset == 8 {
            self.metis_buffer_offset = 520;
        } else {
            self.metis_buffer[0] = 0xEF;
            self.metis_buffer[1] = 0xFE;
            self.metis_buffer[2] = 0x01;
            self.metis_buffer[3] = 0x02;
            self.metis_buffer[4] = (self.send_sequence >> 24) as u8;
            self.metis_buffer[5] = (self.send_sequence >> 16) as u8;
            self.metis_buffer[6] = (self.send_sequence >> 8) as u8;
            self.metis_buffer[7] = self.send_sequence as u8;
            self.socket.send_to(&self.metis_buffer, self.device.address).expect("couldn't send data");
            self.send_sequence += 1;
            self.metis_buffer_offset = 8;
        }
    }

    fn metis_start(&self) {
        let mut buf = [0u8; 64];
        buf[0] = 0xEF;
        buf[1] = 0xFE;
        buf[2] = 0x04;
        buf[3] = 0x03;
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
    }

    fn metis_stop(&self) {
        let mut buf = [0u8; 64];
        buf[0] = 0xEF;
        buf[1] = 0xFE;
        buf[2] = 0x04;
        buf[3] = 0x00;
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
    }

    fn f64_to_f32(input: Vec<f64>) -> Vec<f32> {
        input.into_iter().map(|x| x as f32).collect()
    }

}

