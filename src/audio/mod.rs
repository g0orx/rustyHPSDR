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

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{InputCallbackInfo, OutputCallbackInfo, SampleRate, Stream, StreamConfig};
use ringbuf::storage::Heap;
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::wrap::caching::Caching;
use ringbuf::{HeapRb, SharedRb};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Needed for keeping streams within this structure. they are marked as !Send only because Android arch.
// Since this program is not run on Android, it's safe to mark the whole struct as safe to Send between threads.
unsafe impl Send for Audio {}

#[derive(Default, Deserialize, Serialize)]
pub struct Audio {
    #[serde(skip_serializing, skip_deserializing)]
    input_stream: Option<Stream>,
    #[serde(skip_serializing, skip_deserializing)]
    output_stream: Option<Stream>,
    #[serde(skip_serializing, skip_deserializing)]
    pub output_underruns: i32,
    #[serde(skip_serializing, skip_deserializing)]
    input_buffer: Option<Caching<Arc<SharedRb<Heap<i16>>>, false, true>>,
    #[serde(skip_serializing, skip_deserializing)]
    output_buffer: Option<Caching<Arc<SharedRb<Heap<i16>>>, true, false>>,
}

impl Audio {

    pub fn new() -> Audio {
        let input_stream = None;
        let output_stream = None;
        let output_underruns = 0;
        let input_buffer = None;
        let output_buffer = None;
        Audio {
            input_stream,
            output_stream,
            output_underruns,
            input_buffer,
            output_buffer,
        }
    }

    pub fn init(&mut self) {
        self.input_stream = None;
        self.output_stream = None;
        self.output_underruns = 0;
    }

    pub fn open_input(&mut self, device_name: &String) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();

        // Find the input device
        let device = if device_name == "default" {
            host.default_input_device()
        } else {
            host.input_devices()?
                .find(|d| d.name().map(|n| n == *device_name).unwrap_or(false))
        }
        .ok_or("No input device found")?;

        let config = device.default_input_config()?;

        let period_size = match config.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max: _ } => {
                *min as usize
            }
            cpal::SupportedBufferSize::Unknown => {
                1024
            }
        };

        let (mut prod, cons) = HeapRb::new(period_size * 2).split();
        self.input_buffer = Some(cons);

        let stream = device.build_input_stream(
            &config.config(),
            move |data: &[i16], _: &InputCallbackInfo| {
                prod.push_slice(data);
            },
            |err| eprintln!("audio input error: {}", err),
            None,
        )?;

        stream.play()?;
        self.input_stream = Some(stream);

        Ok(())
    }

    pub fn read_input(&mut self) -> Vec<i16> {
        self.input_buffer.as_mut().unwrap().pop_iter().collect()
    }

    pub fn close_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(stream) = self.input_stream.take() {
            drop(stream);
        }
        Ok(())
    }

    pub fn open_output(&mut self, device_name: &String) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();

        // Find the output device
        let device = if device_name == "default" {
            host.default_output_device()
        } else {
            host.output_devices()?
                .find(|d| d.name().map(|n| n == *device_name).unwrap_or(false))
        }
        .ok_or("No output device found")?;

        let config = device.default_output_config()?;

        // Create a custom config for stereo 48kHz output
        let mut stream_config: StreamConfig = config.clone().into();
        stream_config.channels = 2;
        stream_config.sample_rate = SampleRate(48000);
        //let (prod, mut cons) = HeapRb::new(4800).split();
        let (prod, mut cons) = HeapRb::new(9600).split();
        self.output_buffer = Some(prod);

        let stream = device.build_output_stream(
            &stream_config,
            move |data: &mut [i16], _: &OutputCallbackInfo| {
                if cons.occupied_len() < data.len() {
                    for sample in data.iter_mut() {
                        *sample = 0;
                    }
                    return;
                }
                cons.pop_slice(data);
            },
            |err| eprintln!("audio output error: {}", err),
            None,
        )?;

        stream.play()?;
        self.output_stream = Some(stream);

        Ok(())
    }

    pub fn close_output(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(stream) = self.output_stream.take() {
            drop(stream);
        }
        Ok(())
    }

    pub fn write_output(&mut self, buffer: &Vec<i16>) -> Result<(), Box<dyn std::error::Error>> {
        self.output_buffer.as_mut().unwrap().push_slice(buffer);
        Ok(())
    }

    pub fn list_pcm_devices(input: bool) -> Vec<String> {
        let mut devices = Vec::<String>::new();
        let host = cpal::default_host();

        let device_iter = if input {
            host.input_devices()
        } else {
            host.output_devices()
        };

        match device_iter {
            Ok(devices_iter) => {
                for device in devices_iter {
                    if let Ok(name) = device.name() {
                        devices.push(name);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get device list: {}", e);
            }
        }

        devices
    }
}
