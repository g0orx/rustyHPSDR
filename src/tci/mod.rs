use std::io::Write;
use tokio::select;
use tokio::sync::mpsc::{self};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio::sync::Semaphore;
use tokio::sync::Mutex;
use futures::{StreamExt, SinkExt};

use crate::radio::RadioMutex;
use crate::discovery::device_name;
use crate::modes::Modes;

#[derive(Clone, Debug)]
pub struct TCI {
    addr: String,
}

pub enum TCIMessage {
    ClientConnected(),
    ClientDisconnected(),
    UpdateMox(bool),
    UpdateDDS(f64),
    UpdateFrequencyA(f64),
    UpdateAFGain(f32),
    IQStart(usize),
}

pub enum TCIDataMessage {
    IQData(Vec<f64>),
    AudioData(Vec<f64>),
}

impl Default for TCIMessage {
    fn default() -> Self {
        TCIMessage::UpdateMox(false)
    }
}

enum TCIStreamType {
    IQ_STREAM = 0,
    RX_AUDIO_STREAM,
    TX_AUDIO_STREAM,
    TX_CHRONO,
    LINEOUT_STREAM,
}

impl TCI {

    pub fn new(addr: String) -> Self {
        Self {
            addr: addr,
        }
    }

    pub async fn run(&self, radio_mutex: RadioMutex, tx: &mpsc::Sender<TCIMessage>, rx: Arc<Mutex<mpsc::Receiver<TCIDataMessage>>>, stop_flag: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>> {
    
        eprintln!("TCI WebSocket Server run {}", &self.addr);
        let listener = TcpListener::bind(&self.addr).await?;
        let semaphore = Arc::new(Semaphore::new(1));
        eprintln!("TCI WebSocket Server listening on: ws://{}", self.addr);

        loop {
            // Check if we should even start this iteration
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }

            tokio::select! {
                // 1. Wait for a new connection
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _addr)) => {
                            stream.set_nodelay(true)?;
                            let radio_handle = radio_mutex.clone();
                            let permit_pool = semaphore.clone();
                            let tx_clone = tx.clone();
                            let rx_clone = rx.clone();
                            tokio::spawn(async move {
                                // Try to acquire the permit for a single-client limit
                                match permit_pool.try_acquire() {
                                    Ok(_permit) => {
                                        if let Err(e) = handle_connection(stream, radio_handle, tx_clone, rx_clone).await {
                                            eprintln!("TCI Connection closed: {}", e);
                                        }
                                    }
                                    Err(_) => {
                                        eprintln!("TCI Connection rejected: A client is already connected.");
                                        // Stream is dropped here, closing the socket
                                    }
                                }
                            });

                        }
                        Err(e) => eprintln!("Accept error: {}", e),
                    }
                }
    
                // 2. Monitor the stop flag via a timer or a dedicated shutdown channel
                // This ensures that even if no one connects, we check the flag periodically
                _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
                    if stop_flag.load(Ordering::SeqCst) {
                        break;
                    }
                }
            }
        }
    
        println!("TCI Server shutting down...");
        Ok(())
    }
}

async fn handle_connection(stream: TcpStream, radio: RadioMutex, tx: mpsc::Sender<TCIMessage>, rx: Arc<Mutex<mpsc::Receiver<TCIDataMessage>>>) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    if tx.send(TCIMessage::ClientConnected()).await.is_err() {
        eprintln!("TCI tx (client connected): Main thread receiver was dropped.");
    }
    eprintln!("TCI Client connected.");

    // TCI Initialization commands
    ws_sender.send(Message::Text(get_protocol(radio.clone().into()))).await?;
    ws_sender.send(Message::Text(get_device(radio.clone().into()))).await?;
    ws_sender.send(Message::Text("receive_only:false".into())).await?;
    ws_sender.send(Message::Text(get_trx_count(radio.clone().into()))).await?;
    ws_sender.send(Message::Text(get_channel_count(radio.clone().into()))).await?;
    ws_sender.send(Message::Text(get_vfo_limits(radio.clone().into()))).await?;
    ws_sender.send(Message::Text(get_if_limits(radio.clone().into()))).await?;
    ws_sender.send(Message::Text(get_modulations_list(radio.clone().into()))).await?;

    ws_sender.send(Message::Text(get_dds(radio.clone().into()))).await?;
    ws_sender.send(Message::Text(get_if(radio.clone().into()))).await?;
    ws_sender.send(Message::Text("ready;".into())).await?;
    ws_sender.send(Message::Text(get_vfo(radio.clone().into()))).await?;
    ws_sender.send(Message::Text(get_mode(radio.clone().into()))).await?;
    ws_sender.send(Message::Text(get_filter_band(radio.clone().into()))).await?;
    ws_sender.send(Message::Text(get_ctun(radio.clone().into()))).await?;
    ws_sender.send(Message::Text("start;".into())).await?;
    ws_sender.send(Message::Text("ready;".into())).await?;
    
    

    let rx_owned = Arc::clone(&rx);
    loop {
        tokio::select! {

            // Listen for messages from the WebSocket
            // Note: Use .next().await via select!
            ws_result = ws_receiver.next() => {
                match ws_result {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            let incoming_text = msg.to_text()?;

                            // TCI commands are semicolon-delimited
                            for command in incoming_text.split(';').filter(|s| !s.is_empty()) {
eprintln!("TCI Server <<< {}",command);
                                let tx_clone = tx.clone();
                                if let Some(response) = process_tci_command(command, &radio, tx_clone).await {
eprintln!("TCI Server >>> {}",response);
                                    ws_sender.send(Message::Text(response.into())).await?;
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error: {}", e);
                        break; // Exit loop on error/disconnect
                    }
                    None => {
                        break;
                    }
                }
            }

            _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {
                // Periodically check the mutex-protected receiver
                let mut rx = rx_owned.lock().await;
                while let Ok(msg) = rx.try_recv() {
                    match msg {
                        TCIDataMessage::IQData(data) => {
                            // data should be 1024 I/Q samples = 2048 f64 samples
eprintln!("IQData {}", data.len());
                            // send 256 I/Q samples
                            let chunk_size = 256*2;
                            for chunk in data.chunks(chunk_size) {
                                // 64 byte header plus 256*2*4 samples (256 f64 I and Q samples converted to f32)
                                let mut packet = Vec::with_capacity(64+(chunk_size*4));

                                let receiver_id: u32 = 0;
                                let sample_rate: u32 = 192000;
                                let format: u32 = 3; //float 32
                                let codec: u32 = 0;
                                let crc: u32 = 0;
                                let length: u32 = 256;
                                let stream_type: u32 = TCIStreamType::IQ_STREAM as u32;
                                let channels: u32 = 2;
                                let reserve:u32 = 0;
              
                                packet.extend_from_slice(&receiver_id.to_le_bytes());
                                packet.extend_from_slice(&sample_rate.to_le_bytes());
                                packet.extend_from_slice(&format.to_le_bytes());
                                packet.extend_from_slice(&codec.to_le_bytes());
                                packet.extend_from_slice(&crc.to_le_bytes());
                                packet.extend_from_slice(&length.to_le_bytes());
                                packet.extend_from_slice(&stream_type.to_le_bytes());
                                packet.extend_from_slice(&channels.to_le_bytes());
                                packet.extend_from_slice(&reserve.to_le_bytes());
                                packet.extend_from_slice(&reserve.to_le_bytes());
                                packet.extend_from_slice(&reserve.to_le_bytes());
                                packet.extend_from_slice(&reserve.to_le_bytes());
                                packet.extend_from_slice(&reserve.to_le_bytes());
                                packet.extend_from_slice(&reserve.to_le_bytes());
                                packet.extend_from_slice(&reserve.to_le_bytes());
                                packet.extend_from_slice(&reserve.to_le_bytes());
                                for iq_pair in chunk.chunks_exact(2) {
                                    let i_sample = iq_pair[0] as f32;
                                    let q_sample = iq_pair[1] as f32;
                                    // SWAP: Push Q then I
                                    packet.extend_from_slice(&q_sample.to_le_bytes());
                                    packet.extend_from_slice(&i_sample.to_le_bytes());
                                }
                                ws_sender.send(Message::Binary(packet.clone())).await?;
                                ws_sender.flush().await?;
                            }
                        }
                        TCIDataMessage::AudioData(data) => {
                            let mut packet = Vec::with_capacity(64+(data.len()*4));
eprintln!("AudioData {}", data.len());
                            let receiver_id: u32 = 0;
                            let sample_rate: u32 = 48000;
                            let format: u32 = 3; //float 32
                            let codec: u32 = 0;
                            let crc: u32 = 0;
                            let length: u32 = (data.len()/2) as u32;
                            let stream_type: u32 = TCIStreamType::RX_AUDIO_STREAM as u32;
                            let channels: u32 = 2;
                            let reserve:u32 = 0;

                            packet.extend_from_slice(&receiver_id.to_le_bytes());
                            packet.extend_from_slice(&sample_rate.to_le_bytes());
                            packet.extend_from_slice(&format.to_le_bytes());
                            packet.extend_from_slice(&codec.to_le_bytes());
                            packet.extend_from_slice(&crc.to_le_bytes());
                            packet.extend_from_slice(&length.to_le_bytes());
                            packet.extend_from_slice(&stream_type.to_le_bytes());
                            packet.extend_from_slice(&channels.to_le_bytes());
                            packet.extend_from_slice(&reserve.to_le_bytes());
                            packet.extend_from_slice(&reserve.to_le_bytes());
                            packet.extend_from_slice(&reserve.to_le_bytes());
                            packet.extend_from_slice(&reserve.to_le_bytes());
                            packet.extend_from_slice(&reserve.to_le_bytes());
                            packet.extend_from_slice(&reserve.to_le_bytes());
                            packet.extend_from_slice(&reserve.to_le_bytes());
                            packet.extend_from_slice(&reserve.to_le_bytes());
                            for audio in data {
                                let audio_sample = audio as f32;
                                packet.extend_from_slice(&audio_sample.to_le_bytes());
                            }
                            ws_sender.send(Message::Binary(packet.clone())).await?;
                            ws_sender.flush().await?;

                        }
                        _ => {}
                    }
                }
            }
        }
    }
    if tx.send(TCIMessage::ClientDisconnected()).await.is_err() {
        eprintln!("TCI tx (client connected): Main thread receiver was dropped.");
    }
    eprintln!("TCI Client disconnected.");
    Ok(())
}

async fn process_tci_command(full_cmd: &str, radio_mutex: &RadioMutex, tx: mpsc::Sender<TCIMessage>) -> Option<String> {
    let parts: Vec<&str> = full_cmd.split(':').collect();
    let cmd_name = parts[0].to_lowercase();
    let args = if parts.len() > 1 { parts[1] } else { "" };
    let params: Vec<&str> = args.split(',').collect();

    match cmd_name.as_str() {
        "iq_samplerate" => {
            if let Ok(sample_rate) = params[0].parse::<i32>() {
                let mut r = radio_mutex.radio.lock().unwrap();
                if r.protocol == 1 {
                    r.sample_rate_changed(sample_rate);
                } else {
                    r.receiver[0].sample_rate_changed(sample_rate);
                }
                Some(format!("iq_samplerate:{};", sample_rate).to_string())
            } else {
                None
            }
        }
        "iq_start" => {
            // param 0 is the receiver - assuming 0 for now
            if tx.send(TCIMessage::IQStart(0)).await.is_err() {
                eprintln!("TCI tx (trx): Main thread receiver was dropped.");
            }
            None
        }
        "rx_sensors_enable" => {
            None
        }
        "tx_sensors_enable" => {
            None
        }
        "split_enable" => {
            None
        }
        "sql_enable" => {
            None
        }
        "sql_level" => {
            None
        }
        "rx_anf_enable" => {
            None
        }
        "drive" => {
            None
        }
        "rx_smeter" => {
            let mut r = radio_mutex.radio.lock().unwrap();
            Some(format!("rx_smeter:{},{};", 0, r.s_meter_dbm).to_string())
        }
        "trx" => {
            if tx.send(TCIMessage::UpdateMox(params[1]=="true")).await.is_err() {
                eprintln!("TCI tx (trx): Main thread receiver was dropped.");
            }
            None
        }
        "dds" => {
            if let Ok(freq) = params[1].parse::<f64>() {
                if tx.send(TCIMessage::UpdateDDS(freq)).await.is_err() {
                    eprintln!("TCI tx (vfo): Main thread receiver was dropped.");
                }
                eprintln!("TCI: VFO set to {}", freq);
            }
            None
        }
        "vfo" => {
            if let Ok(freq) = params[2].parse::<f64>() {
                if tx.send(TCIMessage::UpdateFrequencyA(freq)).await.is_err() {
                    eprintln!("TCI tx (vfo): Main thread receiver was dropped.");
                }
                eprintln!("TCI: VFO set to {}", freq);
            }
            None
        }
        "rx_volume" => {
            if params.len() == 2 {
                let mut r = radio_mutex.radio.lock().unwrap();
                let gain = r.receiver[0].afgain;
                let norm_gain = gain / 100.0;
                let linear_gain = norm_gain.sqrt();
                let actual_gain = -60.0 + (linear_gain * (0.0 - (-60.0)));
                Some(format!("rx_volume:{},{};", 0, actual_gain))
            } else {
                if let Ok(volume) = params[2].parse::<f32>() {
                    let gain = volume;
                    let norm_gain = (gain - (-60.0)) / (0.0 - (-60.0));
                    let log_gain = norm_gain.powf(2.0);
                    let actual_gain = log_gain * 100.0;
                    if tx.send(TCIMessage::UpdateAFGain(actual_gain)).await.is_err() {
                        eprintln!("TCI tx (rx_volume): Main thread receiver was dropped.");
                    }
                    eprintln!("TCI: AFGain set to {}", volume);
                }
                None
            }
        }
        "modulation" => {
            if params.len() == 2 {
                // set the modulation
            } else {
            }
            None
        }
        _ => {
            // Log unknown commands to help debugging protocol sync
            eprintln!("TCI: Unhandled command '{}'", cmd_name);
            for i in 0..params.len() {
                eprintln!("    {}", params[i]);
            }
            None
        }
    }
}

fn get_device(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = format!("device:{:?};",r.model);
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_dds(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = format!("dds:0,{};",r.receiver[0].frequency);
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_vfo(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = format!("vfo:0,0,{};", if r.receiver[0].ctun {
                                r.receiver[0].ctun_frequency
                            } else {
                                r.receiver[0].frequency
                            });
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_if(radio: RadioMutex) -> String {
    let message = format!("if:0,0,0");
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_modulations_list(radio: RadioMutex) -> String {
    let message = format!("modulations_list:LSB,USB,DSB,CWL,CWU,FMN,AM,DIGU,SPEC,DIGL,SAM,DRM;");
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_mode(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = if r.receiver[0].mode == Modes::CWL.to_usize() || r.receiver[0].mode == Modes::CWU.to_usize() {
        format!("modulation:0,{};", "cw")
    } else {
        format!("modulation:0,{:?};", Modes::from_usize(r.receiver[0].mode).unwrap())
    };
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_filter_band(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = format!("rx_filter_band:0,{},{};", r.receiver[0].filter_low as i32, r.receiver[0].filter_high as i32);
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_ctun(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = format!("rx_ctun_ex:0,{};", if r.receiver[0].ctun {"true"} else {"false"});
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_vfo_limits(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = format!("vfo_limits:{},{};", r.frequency_min, r.frequency_max);
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_if_limits(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = format!("if_limits:{},{};", -r.receiver[0].sample_rate/2, r.receiver[0].sample_rate/2);
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_trx_count(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = format!("trx_count:{};", r.receivers);
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_channel_count(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = format!("channel_count:{};", 1);
eprintln!("TCI Server >>> {}", message);
    message
}

fn get_protocol(radio: RadioMutex) -> String {
    let r = radio.radio.lock().unwrap();
    let message = format!("protocol:{},{};", "Thetis", "2.0");
eprintln!("TCI Server >>> {}", message);
    message
}

