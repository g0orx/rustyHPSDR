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

use std::net::{UdpSocket, SocketAddr};
use socket2::{Socket, Domain, Type, Protocol};

use crate::alex::*;
use crate::antenna::Antenna;
use crate::audio::*;
use crate::discovery::{Device, Boards};
use crate::modes::Modes;
use crate::radio::{Keyer, RadioMutex};
use crate::receiver::{AudioOutput, Receiver};

const HEADER_SIZE: usize  = 16;  // 16 byte header
const SAMPLE_SIZE: usize = 3;    // 3 byte (24 bit) samples
const INTERLEAVE_FACTOR: usize = 2; // 2 samples (I & Q) interleaved
const MIC_HEADER_SIZE: usize = 4;   // just a sequance number
const MIC_SAMPLE_SIZE: usize = 2;    // 2 byte (16 bit) samples
const MIC_SAMPLES: usize = 64;       // 64 samples per buffer
const IQ_BUFFER_SIZE: usize = 240;   // 240 IQ samples

const RX_YELLOW_LED: u32 = 0x00000001;
const HPF_13MHZ: u32 =     0x00000002;
const HPF_20MHZ: u32 =     0x00000004;
const PREAMP_6M: u32 =     0x00000008;
const HPF_9_5MHZ: u32 =    0x00000010;
const HPF_6_5MHZ: u32 =    0x00000020;
const HPF_1_5MHZ: u32 =    0x00000040;
const UNUSED_1: u32 =      0x00000080;
const XVTR_RX_IN: u32 =    0x00000100;
const RX_2_IN: u32 =       0x00000200;
const RX_1_IN: u32 =       0x00000400;
const RX_1_OUT: u32 =      0x00000800;
const HPF_BYPASS: u32 =    0x00001000;
const ATTEN_20_dB: u32 =   0x00002000;
const ATTEN_10_dB: u32 =   0x00004000;
const RX_RED_LED: u32 =    0x00008000;
const UNUSED_2: u32 =      0x00010000;
const UNUSED_3: u32 =      0x00020000;
const TRX_STATUS: u32 =    0x00040000;
const TX_YELLOW_LED: u32 = 0x00080000;
const LPF_30_20: u32 =     0x00100000;
const LPF_60_40: u32 =     0x00200000;
const LPF_80: u32 =        0x00400000;
const LPF_160: u32 =       0x00800000;
const ANT_1: u32 =         0x01000000;
const ANT_2: u32 =         0x02000000;
const ANT_3: u32 =         0x04000000;
const TR_RELAY: u32 =      0x08000000;
const TX_RED_LED: u32 =    0x10000000;
const LPF_BYPASS: u32 =    0x20000000;
const LPF_12_10: u32 =     0x40000000;
const LPF_17_15: u32 =     0x80000000;

//#[derive(Default)]
pub struct Protocol2 {
    device: Device, 
    socket: UdpSocket,     
    receivers: u8,
    general_sequence: u32,
    high_priority_sequence: u32,
    receive_specific_sequence: u32,
    transmit_specific_sequence: u32,
    audio_sequence: u32,
    tx_iq_sequence: u32,
    previous_filter1: u32,
    previous_filter2: u32,
    rx_audio: Vec<Audio>,
    tx_audio: Audio,
}   

impl Protocol2 {

    pub fn new(device: Device) -> Protocol2 {
        let socket_addr: SocketAddr = "0.0.0.0:0".parse().expect("Invalid Address");
        let setup_socket = Socket::new(Domain::for_address(socket_addr), Type::DGRAM, Some(Protocol::UDP)).expect("Socket::new failed");
        setup_socket.set_reuse_address(true).expect("set_reuse_address failed");
        #[cfg(unix)]
        {
            setup_socket.set_reuse_port(true).expect("set_reuse_port failed");
        }
        setup_socket.bind(&socket_addr.into()).expect("bind failed");
        let socket: UdpSocket = setup_socket.into();

        let receivers: u8 = 2;
        let general_sequence: u32 = 0;
        let high_priority_sequence: u32 = 0;
        let receive_specific_sequence: u32 = 0;
        let transmit_specific_sequence: u32 = 0; 
        let audio_sequence: u32 = 0; 
        let tx_iq_sequence: u32 = 0; 
        let previous_filter1: u32 = 0;
        let previous_filter2: u32 = 0;
        let mut rx_audio: Vec<Audio> = Vec::new();
        for _i in 0..receivers {
            rx_audio.push(Audio::new());
        }
        let tx_audio: Audio = Audio::new();

        

        Protocol2{device,
                           socket,
                           receivers,
                           general_sequence,
                           high_priority_sequence,
                           receive_specific_sequence,
                           transmit_specific_sequence,
                           audio_sequence,
                           tx_iq_sequence,
                           previous_filter1,
                           previous_filter2,
                           rx_audio,
                           tx_audio,
        }

    }

    pub fn run(&mut self, radio_mutex: &RadioMutex) {
        let r = radio_mutex.radio.lock().unwrap();
        if r.receiver[0].local_output {
            let _ = self.rx_audio[0].open_output(&r.receiver[0].output_device);
        }
        if r.receiver[1].local_output {
            let _ = self.rx_audio[1].open_output(&r.receiver[1].output_device);
        }
        if r.transmitter.local_input {
            let _ = self.tx_audio.open_input(&r.transmitter.input_device);
        }
        drop(r);

        let mut tx_iq_buffer: Vec<f64> = vec![0.0; IQ_BUFFER_SIZE*2];
        let mut tx_iq_buffer_offset: usize = 0;

        self.send_general();
        self.send_high_priority(radio_mutex);
        self.send_transmit_specific(radio_mutex);
        self.send_receive_specific(radio_mutex);

        let mut buffer = vec![0; 4096];
        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((size, src)) => {
                    match src.port() {
                        1024 => {}, // Command responce
                        1025 => { // High Priority
                                // first 4 bytes are the sequence number - should check it
                                let mut r = radio_mutex.radio.lock().unwrap();
                                    let previous_ptt = r.ptt;
                                    let previous_dot = r.dot;
                                    let previous_dash = r.dash;
                                    r.ptt = (buffer[4] & 0x01) == 0x01;
                                    r.dot = ((buffer[4] >> 1) & 0x01) == 0x01;
                                    r.dash = ((buffer[4] >> 2) & 0x01) == 0x01;

                                    r.pll_locked = ((buffer[5] >> 2) & 0x01) == 0x01;
                                    r.transmitter.alex_forward_power = u16::from_be_bytes([buffer[14], buffer[15]]);
                                    r.transmitter.alex_reverse_power = u16::from_be_bytes([buffer[22], buffer[23]]);
                                    r.supply_volts = u16::from_be_bytes([buffer[49], buffer[50]]) as i32;

                                    if r.ptt != previous_ptt || r.dot != previous_dot || r.dash != previous_dash {
                                        r.set_state();
                                    }

                                    r.received = true;

                                drop(r);
                                
                                self.send_high_priority(radio_mutex);
                                },
                        1026 => { // Mic/Line In Samples
                                let mut r = radio_mutex.radio.lock().unwrap();
                                // use samples from radio microphone if not local microphone or tuning
                                if !r.transmitter.local_input || r.tune {
                                    let data_size = MIC_SAMPLES * MIC_SAMPLE_SIZE;
                                    let mut b = MIC_HEADER_SIZE;
                                    if size >= MIC_HEADER_SIZE + data_size {
                                        for _i in 0..MIC_SAMPLES {
                                            let sample = ((i16::from_be_bytes([buffer[b], buffer[b+1]])) as f32) / 32767.0;
                                            b += 2;
                                            if r.transmitter.add_mic_sample(sample) && r.is_transmitting() {
                                                for j in 0..r.transmitter.output_samples {
                                                    let ix = j * 2;
                                                    let ox = tx_iq_buffer_offset * 2;
                                                    tx_iq_buffer[ox] = r.transmitter.iq_buffer[ix as usize];
                                                    tx_iq_buffer[ox+1] = r.transmitter.iq_buffer[(ix+1) as usize];
                                                    tx_iq_buffer_offset += 1;
                                                    if tx_iq_buffer_offset >= IQ_BUFFER_SIZE {
                                                        self.send_iq_buffer(tx_iq_buffer.clone());
                                                        tx_iq_buffer_offset = 0;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                r.received = true;
                                },
                        1027 => {}, // Wide Band IQ samples
                        1035..=1042 => { // RX IQ samples
                            let ddc = (src.port()-1035) as usize;
                            let mut r = radio_mutex.radio.lock().unwrap();

                            if ddc ==0 || (ddc == 1 && r.rx2_enabled) {

                            let iq_sample_count = u16::from_be_bytes([buffer[14], buffer[15]]) as usize;
                            let data_size = iq_sample_count * SAMPLE_SIZE * INTERLEAVE_FACTOR;
                            let mut b = HEADER_SIZE;
    
                            if size >= HEADER_SIZE + data_size {
                                for _i in 0..iq_sample_count {
                                    let i_sample = if buffer[b] & 0x80 != 0 {
                                                       u32::from_be_bytes([0xFF, buffer[b], buffer[b+1], buffer[b+2]]) as i32
                                                   } else {
                                                       u32::from_be_bytes([0, buffer[b], buffer[b+1], buffer[b+2]]) as i32
                                                   };
                                    b += 3;
                                    let q_sample = if buffer[b] & 0x80 != 0 {
                                                       u32::from_be_bytes([0xFF, buffer[b], buffer[b+1], buffer[b+2]]) as i32
                                                   } else {
                                                       u32::from_be_bytes([0, buffer[b], buffer[b+1], buffer[b+2]]) as i32
                                                   };
                                    b += 3;

                                    let i = r.receiver[ddc].samples*2;
                                    r.receiver[ddc].iq_input_buffer[i]=i_sample as f64/16777215.0;
                                    r.receiver[ddc].iq_input_buffer[i+1]=q_sample as f64/16777215.0;
                                    r.receiver[ddc].samples += 1;
                                    if r.receiver[ddc].samples >= r.receiver[ddc].buffer_size {
                                        r.receiver[ddc].process_iq_samples();
                                        r.receiver[ddc].samples = 0;
                                        for i in 0..r.receiver[ddc].output_samples {
                                            let ix = i * 2;
                                            let left_sample: f32 = (r.receiver[ddc].audio_buffer[ix] * 32767.0) as f32;
                                            let right_sample: f32 = (r.receiver[ddc].audio_buffer[ix+1] * 32767.0) as f32;
                                            let rox = r.receiver[ddc].remote_audio_buffer_offset;

                                            // always stereo to radio
                                            r.receiver[ddc].remote_audio_buffer[rox] = (left_sample as i16 >> 8) as u8;
                                            r.receiver[ddc].remote_audio_buffer[rox+1] = left_sample as u8;
                                            r.receiver[ddc].remote_audio_buffer[rox+2] = (right_sample as i16 >> 8) as u8;
                                            r.receiver[ddc].remote_audio_buffer[rox+3] = right_sample as u8;
                                            /*
                                            match r.receiver[ddc].audio_output {
                                                AudioOutput::Stereo => {
                                                    r.receiver[ddc].remote_audio_buffer[rox] = (left_sample >> 8) as u8;
                                                    r.receiver[ddc].remote_audio_buffer[rox+1] = left_sample as u8;
                                                    r.receiver[ddc].remote_audio_buffer[rox+2] = (right_sample >> 8) as u8;
                                                    r.receiver[ddc].remote_audio_buffer[rox+3] = right_sample as u8;
                                                },
                                                AudioOutput::Left => {
                                                    r.receiver[ddc].remote_audio_buffer[rox] = (left_sample >> 8) as u8;
                                                    r.receiver[ddc].remote_audio_buffer[rox+1] = left_sample as u8;
                                                    r.receiver[ddc].remote_audio_buffer[rox+2] = 0;
                                                    r.receiver[ddc].remote_audio_buffer[rox+3] = 0;
                                                },
                                                AudioOutput::Right => {
                                                    r.receiver[ddc].remote_audio_buffer[rox] = 0;
                                                    r.receiver[ddc].remote_audio_buffer[rox+1] = 0;
                                                    r.receiver[ddc].remote_audio_buffer[rox+2] = (right_sample >> 8) as u8;
                                                    r.receiver[ddc].remote_audio_buffer[rox+3] = right_sample as u8;
                                                },
                                                AudioOutput::Mute => {
                                                    r.receiver[ddc].remote_audio_buffer[rox] = 0;
                                                    r.receiver[ddc].remote_audio_buffer[rox+1] = 0;
                                                    r.receiver[ddc].remote_audio_buffer[rox+2] = 0;
                                                    r.receiver[ddc].remote_audio_buffer[rox+3] = 0;
                                                },
                                            }
                                            */

                                            r.receiver[ddc].remote_audio_buffer_offset += 4;
                                            if r.receiver[ddc].remote_audio_buffer_offset >= r.receiver[ddc].remote_audio_buffer_size {
                                                if r.receiver[ddc].active {
                                                    self.send_audio(r.receiver[ddc].clone());
                                                }
                                                r.receiver[ddc].remote_audio_buffer_offset = 4;
                                            }

                                            if r.receiver[ddc].local_output {
                                                let lox=r.receiver[ddc].local_audio_buffer_offset * 2;
                                                match r.receiver[ddc].audio_output {
                                                    AudioOutput::Stereo => {
                                                        r.receiver[ddc].local_audio_buffer[lox]=left_sample;
                                                        r.receiver[ddc].local_audio_buffer[lox+1]=right_sample;
                                                    },
                                                    AudioOutput::Left => {
                                                        r.receiver[ddc].local_audio_buffer[lox]=left_sample;
                                                        r.receiver[ddc].local_audio_buffer[lox+1]=0.0;
                                                    },
                                                    AudioOutput::Right => {
                                                        r.receiver[ddc].local_audio_buffer[lox]=0.0;
                                                        r.receiver[ddc].local_audio_buffer[lox+1]=right_sample;
                                                    },
                                                    AudioOutput::Mute => {
                                                        r.receiver[ddc].local_audio_buffer[lox]=0.0;
                                                        r.receiver[ddc].local_audio_buffer[lox+1]=0.0;
                                                    },
                                                }
                                                r.receiver[ddc].local_audio_buffer_offset += 1;
                                                if r.receiver[ddc].local_audio_buffer_offset == r.receiver[ddc].local_audio_buffer_size {
                                                    r.receiver[ddc].local_audio_buffer_offset = 0;
                                                    let buffer_clone = r.receiver[ddc].local_audio_buffer.clone();
                                                    let _ = self.rx_audio[ddc].write_output(&buffer_clone);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            r.received = true;
                            }
                        },
                        _ => {
                            println!("invalid UDP port: {}", src.port());
                        },
                    }
                }
                Err(e) =>  {println!("Error receiving UDP packet: {}", e);}
            }

            let mut r = radio_mutex.radio.lock().unwrap();
            if r.transmitter.local_input && !r.transmitter.local_input_changed && !r.tune {
                let (mic_buffer, count) = self.tx_audio.read_input();
                //eprintln!("mic samples {}", count);
                for i in 0..count {
                    if r.transmitter.add_mic_sample(mic_buffer[i]) && r.is_transmitting() {
                        for j in 0..r.transmitter.output_samples {
                            let ix = j * 2;
                            let ox = tx_iq_buffer_offset * 2;
                            tx_iq_buffer[ox] = r.transmitter.iq_buffer[ix as usize];
                            tx_iq_buffer[ox+1] = r.transmitter.iq_buffer[(ix+1) as usize];
                            tx_iq_buffer_offset += 1;
                            if tx_iq_buffer_offset >= IQ_BUFFER_SIZE {
                                self.send_iq_buffer(tx_iq_buffer.clone());
                                tx_iq_buffer_offset = 0;
                            }
                        }
                    }
                }
            }
            drop(r);

            // check for any changes we need to handle here
            let mut r = radio_mutex.radio.lock().unwrap();
            let updated = r.updated;
            let keepalive = r.keepalive;
            r.updated = false;
            r.keepalive = false;
            let local_input = r.transmitter.local_input;
            let local_input_changed = r.transmitter.local_input_changed;
            let input_device = r.transmitter.input_device.clone();
            let input_device_changed = r.transmitter.input_device_changed;
            r.transmitter.local_input_changed = false;
            r.transmitter.input_device_changed = false;
            let rx1_local_output_changed_to = r.receiver[0].local_output_changed_to;
            let rx1_local_output_changed = r.receiver[0].local_output_changed;
            let rx1_local_output_device_changed = r.receiver[0].local_output_device_changed;
            let rx1_local_output = r.receiver[0].local_output;
            let rx1_output_device = r.receiver[0].output_device.clone();
            r.receiver[0].local_output_device_changed = false;
            let rx2_local_output_changed_to = r.receiver[1].local_output_changed_to;
            let rx2_local_output_changed = r.receiver[1].local_output_changed;
            let rx2_local_output_device_changed = r.receiver[1].local_output_device_changed;
            let rx2_local_output = r.receiver[1].local_output;
            let rx2_output_device = r.receiver[1].output_device.clone();
            r.receiver[1].local_output_device_changed = false;
            drop(r);
            if keepalive || updated {
                self.send_general();
                self.send_transmit_specific(radio_mutex);
                self.send_receive_specific(radio_mutex);
                self.send_high_priority(radio_mutex);
            }
            if local_input_changed {
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
                let mut r = radio_mutex.radio.lock().unwrap();
                r.receiver[0].local_output = false;
                drop(r);
                let _ = self.rx_audio[0].close_output();
                let _ = self.rx_audio[0].open_output(&rx1_output_device);
                let mut r = radio_mutex.radio.lock().unwrap();
                r.receiver[0].local_output = true;
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
                let mut r = radio_mutex.radio.lock().unwrap();
                r.receiver[1].local_output = false;
                drop(r);
                let _ = self.rx_audio[1].close_output();
                let _ = self.rx_audio[1].open_output(&rx2_output_device);
                let mut r = radio_mutex.radio.lock().unwrap();
                r.receiver[1].local_output = true;
            }
        }
    }

    pub fn send_general(&mut self) {
        // send to port 1024
        let mut buf = [0u8; 60];
        buf[0] = ((self.general_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.general_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.general_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.general_sequence) & 0xFF) as u8;

        buf[23] = 0x00; // wideband not enabled
        buf[37] = 0x08; // phase word (not frequency)
        buf[38] = 0x01; // enable hardware timer

        buf[58] = 0x01; // enable PA

        if self.device.adcs == 2 {
          buf[59] = 0x03; // enable ALEX 0 and 1
        } else {
          buf[59] = 0x01; // enable ALEX 0
        }

        self.device.address.set_port(1024);
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.general_sequence += 1;
    }


    pub fn send_high_priority(&mut self, radio_mutex: &RadioMutex) {
        // port 1027
        let r = radio_mutex.radio.lock().unwrap();

        let mut buf = [0u8; 1444];
        buf[0] = ((self.high_priority_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.high_priority_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.high_priority_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.high_priority_sequence) & 0xFF) as u8;
    
        buf[4] = 0x01; // running
        if r.is_transmitting() {
            buf[4] |= 0x02;
        }
    
        // receiver frequency
        for i in 0..r.receivers {
            // convert frequency to phase
            let mut f = r.receiver[i as usize].frequency;
            let b = r.receiver[i as usize].band.to_usize();
            f = f - r.receiver[i as usize].band_info[b].lo;
            f = f - r.receiver[i as usize].band_info[b].lo_error;

            let phase = ((4294967296.0*f)/122880000.0) as u32;
            buf[(9+(i*4)) as usize] = ((phase>>24) & 0xFF) as u8;
            buf[(10+(i*4)) as usize] = ((phase>>16) & 0xFF) as u8;
            buf[(11+(i*4)) as usize] = ((phase>>8) & 0xFF) as u8;
            buf[(12+(i*4)) as usize] = (phase & 0xFF) as u8;
        }

        // transmit frequency
        let f = if r.split {
                    let mut f = r.receiver[1].frequency;
                    if r.receiver[1].ctun {
                        f = r.receiver[1].ctun_frequency;
                    }
                    let b = r.receiver[1].band.to_usize();
                    f = f - r.receiver[1].band_info[b].lo;
                    f = f - r.receiver[1].band_info[b].lo_error;
                    f
                } else {
                    let mut f = r.receiver[0].frequency;
                    if r.receiver[0].ctun {
                        f = r.receiver[0].ctun_frequency;
                    }
                    let b = r.receiver[0].band.to_usize();
                    f = f - r.receiver[0].band_info[b].lo;
                    f = f - r.receiver[0].band_info[b].lo_error;
                    f
                };
        let phase = ((4294967296.0*f)/122880000.0) as u32;
        buf[329] = ((phase>>24) & 0xFF) as u8;
        buf[330] = ((phase>>16) & 0xFF) as u8;
        buf[331] = ((phase>>8) & 0xFF) as u8;
        buf[332] = (phase & 0xFF) as u8;

        // transmit power
        let power = if r.is_transmitting() {
                            let mut p = r.transmitter.drive * 255.0 / 100.0;
                            if p > 255.0 {
                                p = 255.0;
                            }
                            p
                        } else {
                            0.0
                        };
        buf[345] = power as u8;

        let mut filter1: u32 = 0x00000000;

        if r.is_transmitting() {
            filter1 |= 0x08000000; // TX_ENABLE
            let b = if r.split {
                        r.receiver[1].band.to_usize()
                    } else {
                        r.receiver[0].band.to_usize()
                    };
            let tx_antenna = if r.split {
                                 r.receiver[1].band_info[b].tx_antenna
                             } else {
                                 r.receiver[0].band_info[b].tx_antenna
                             };
            match tx_antenna {
                Antenna::ANT1 => filter1 |= ALEX_ANTENNA_1,
                Antenna::ANT2 => filter1 |= ALEX_ANTENNA_2,
                Antenna::ANT3 => filter1 |= ALEX_ANTENNA_3,
                _ => filter1 |= ALEX_ANTENNA_1,
            }
        } else {
            // set the rx antenna
            let b = r.receiver[0].band.to_usize();
            match r.receiver[0].band_info[b].antenna {
                Antenna::ANT1 => filter1 |= ALEX_ANTENNA_1,
                Antenna::ANT2 => filter1 |= ALEX_ANTENNA_2,
                Antenna::ANT3 => filter1 |= ALEX_ANTENNA_3,
                Antenna::EXT1 => filter1 |= ALEX_RX_ANTENNA_EXT1,
                Antenna::EXT2 => filter1 |= ALEX_RX_ANTENNA_EXT2,
                Antenna::XVTR => filter1 |= ALEX_RX_ANTENNA_XVTR,
                _ => {},
            }
        }

        // set BPF
        let mut f = r.receiver[0].frequency;
        if f < 1500000.0 {
            filter1 |= HPF_BYPASS;
        } else if f < 2100000.0 {
            filter1 |= HPF_1_5MHZ;
        } else if f < 5500000.0 {
            filter1 |= HPF_6_5MHZ;
        } else if f < 11000000.0 {
            filter1 |= HPF_9_5MHZ;
        } else if f < 22000000.0 {
            filter1 |= HPF_13MHZ;
        } else if f < 35000000.0 {
            filter1 |= HPF_20MHZ;
        } else {
            filter1 |= PREAMP_6M;
        }


        // set LPF
        if f > 32000000.0 {
            filter1 |= LPF_BYPASS; // 6M
        } else if f > 22000000.0 {
            filter1 |= LPF_12_10; // 12M/10M
        } else if f > 15000000.0 {
            filter1 |= LPF_17_15; // 17M/15M
        } else if f > 8000000.0 {
            filter1 |= LPF_30_20; // 30M/20M
        } else if f > 4500000.0 {
            filter1 |= LPF_60_40; // 60M/40M
        } else if f > 2400000.0 {
            filter1 |= LPF_80; // 80M
        } else if f > 1500000.0 {
            filter1 |= LPF_160; // 160M
        } else {
            filter1 |= LPF_BYPASS;
        }

        
        buf[1432]=((filter1 >> 24) & 0xFF) as u8;
        buf[1433]=((filter1 >> 16) & 0xFF) as u8;
        buf[1434]=((filter1 >> 8) & 0xFF) as u8;
        buf[1435]=(filter1 & 0xFF) as u8;
 
        let mut filter2: u32 = 0x00000000;
        f = r.receiver[1].frequency;
        if self.device.board == Boards::Orion2 {
            if f < 1500000.0 {
                filter2 |= HPF_BYPASS; // BYPASS
            } else if f < 2100000.0 {
                filter2 |= HPF_1_5MHZ;
            } else if f < 5500000.0 {
                filter2 |= HPF_6_5MHZ;
            } else if f < 11000000.0 {
                filter2 |= HPF_9_5MHZ;
            } else if f < 22000000.0 {
                filter2 |= HPF_13MHZ;
            } else if f < 35000000.0 {
                filter2 |= HPF_20MHZ;
            } else {
                filter2 |= PREAMP_6M;
            }
        } else if f < 1500000.0 {
            filter2 |= 0x1000;
        } else if f < 2100000.0 {
            filter2 |= 0x40;
        } else if f < 5500000.0 {
            filter2 |= 0x20;
        } else if f < 11000000.0 {
            filter2 |= 0x10;
        } else if f < 22000000.0 {
            filter2 |= 0x02;
        } else if f < 35000000.0 {
            filter2 |= 0x04;
        } else {
            filter2 |= 0x08;
        }

        buf[1430] = ((filter2>>8)&0xFF) as u8;
        buf[1431] = (filter2&0xFF) as u8;

        let mut rx = 0;
        if r.receiver[1].active {
            rx = 1;
        }
        let b = r.receiver[rx as usize].band.to_usize();
        let attenuation = r.receiver[rx as usize].band_info[b].attenuation;

        if r.is_transmitting() {
            buf[1443] = 0;
            buf[1442] = 0;
        } else {
            buf[1443] = attenuation as u8;
            buf[1442] = attenuation as u8;
        }

        self.device.address.set_port(1027);
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.high_priority_sequence += 1;
    }

    pub fn send_audio(&mut self, mut rx: Receiver) {
        // port 1028
        rx.remote_audio_buffer[0] = ((self.audio_sequence >> 24) & 0xFF) as u8;
        rx.remote_audio_buffer[1] = ((self.audio_sequence >> 16) & 0xFF) as u8;
        rx.remote_audio_buffer[2] = ((self.audio_sequence >> 8) & 0xFF) as u8;
        rx.remote_audio_buffer[3] = ((self.audio_sequence) & 0xFF) as u8;
        self.device.address.set_port(1028);
        self.socket.send_to(&rx.remote_audio_buffer, self.device.address).expect("couldn't send data");
        self.audio_sequence += 1;
    }

    pub fn send_receive_specific(&mut self, radio_mutex: &RadioMutex) {
        // port 1025
        let r = radio_mutex.radio.lock().unwrap();

        let mut buf = [0u8; 1444];
        buf[0] = ((self.receive_specific_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.receive_specific_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.receive_specific_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.receive_specific_sequence) & 0xFF) as u8;

        buf[4] = r.adc.len() as u8;
        for i in 0..r.adc.len() {
            buf[5] |= (r.adc[i].dither as u8) << i;
            buf[6] |= (r.adc[i].random as u8) << i;
        }
        buf[7] = 0x03; // 2 receivers -- DDC0 and DDC1

        for i in 0..r.receivers {
          buf[(17+(i*6)) as usize] = r.receiver[i as usize].adc as u8;
          buf[(18+(i*6)) as usize] = (((r.receiver[i as usize].sample_rate/1000)>>8)&0xFF) as u8; // sample_rate
          buf[(19+(i*6)) as usize] = ((r.receiver[i as usize].sample_rate/1000)&0xFF) as u8; // sample_rate to use for DDC0
          buf[(22+(i*6)) as usize] = 24;  // 24 bits per sample
        }

        self.device.address.set_port(1025);
        //println!("send_receive_specific: 1025");
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.receive_specific_sequence += 1;
    }

    pub fn send_transmit_specific(&mut self, radio_mutex: &RadioMutex) {
        // port 1026
        let r = radio_mutex.radio.lock().unwrap();
        let tx = &r.transmitter;

        let mut buf = [0u8; 60];
        buf[0] = ((self.transmit_specific_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.transmit_specific_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.transmit_specific_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.transmit_specific_sequence) & 0xFF) as u8;

        buf[4] = 1; // DACs

        if r.cw_keyer_sidetone_volume != 0 {
            buf[5] = 0x01;
        } else {
            buf[5] = 0x00;
        }

        if tx.mode == Modes::CWL.to_usize()  || tx.mode == Modes::CWU.to_usize() {
            buf[5] |= 0x02;
        }
        if r.cw_keys_reversed {
            buf[5] |= 0x04;
        }     
        if r.cw_keyer_mode == Keyer::ModeA {
            buf[5] |= 0x08;
        }     
        if r.cw_keyer_mode == Keyer::ModeB {
            buf[5] |= 0x28;
        }
        if r.cw_keyer_sidetone_volume != 0 {
            buf[5] |= 0x10;
        } 
        if r.cw_keyer_spacing != 0 {
            buf[5] |= 0x40;
        }
        if r.cw_breakin {
            buf[5] |= 0x80;
        }

        buf[6] = r.cw_keyer_sidetone_volume as u8;
        buf[7] = (r.cw_keyer_sidetone_frequency >> 8) as u8;
        buf[8] = r.cw_keyer_sidetone_frequency as u8;

        buf[9] = r.cw_keyer_speed as u8;
        buf[10] = r.cw_keyer_weight as u8;
        buf[11] = ((r.cw_keyer_hang_time >> 8) & 0xFF) as u8;
        buf[12] = (r.cw_keyer_hang_time &0xFF) as u8;

        buf[50] = 0x00;
        if r.line_in {
            buf[50] |= 0x01;
        }
        if r.mic_boost {
            buf[50] |= 0x02;
        }
        if !r.mic_ptt {
            buf[50] |= 0x04;
        }
        if r.mic_bias_ring { // ptt on tip else bias on tip and ptt on ring
            buf[50] |= 0x08;
        }
        if r.mic_bias_enable {
            buf[50] |= 0x10;
        }
        if r.mic_saturn_xlr {
            buf[50] |= 0x20;
        }

        buf[51] = r.transmitter.lineingain as u8;

        self.device.address.set_port(1026);
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.transmit_specific_sequence += 1;
    }


    fn send_iq_buffer(&mut self, buffer: Vec<f64>) {
        // port 1029
        let mut buf = [0u8; 1444];
        buf[0] = ((self.tx_iq_sequence >> 24) & 0xFF) as u8;
        buf[1] = ((self.tx_iq_sequence >> 16) & 0xFF) as u8;
        buf[2] = ((self.tx_iq_sequence >> 8) & 0xFF) as u8;
        buf[3] = ((self.tx_iq_sequence) & 0xFF) as u8;

        // send 240 24 bit I/Q samples
        let mut b = 4;
        for x in 0..IQ_BUFFER_SIZE {
            let ix = x * 2;
            let mut isample = buffer[ix] * 8388607.0;
            if isample>=0.0 {
                isample = (isample + 0.5).floor();
            } else {
                isample = (isample - 0.5).ceil();
            }
            let mut qsample = buffer[ix+1] * 8388607.0;
            if qsample>=0.0 {
                qsample = (qsample + 0.5).floor();
            } else {
                qsample = (qsample - 0.5).ceil();
            }

            let i = isample as i32;
            let q = qsample as i32;

            buf[b]=(i >> 16) as u8;
            buf[b+1]=(i >> 8) as u8;
            buf[b+2]=i as u8;
            buf[b+3]=(q >> 16) as u8;
            buf[b+4]=(q >> 8) as u8;
            buf[b+5]=q as u8;

            b += 6;
        }

        self.device.address.set_port(1029);
        self.socket.send_to(&buf, self.device.address).expect("couldn't send data");
        self.tx_iq_sequence += 1;
    }

    fn f64_to_f32(input: Vec<f64>) -> Vec<f32> {
        input.into_iter().map(|x| x as f32).collect()
    }

}
