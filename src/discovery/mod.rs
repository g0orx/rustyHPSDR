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

use gtk::prelude::*;
use gtk::{ApplicationWindow, Builder, Button, Label, ListBox, ListBoxRow, Orientation, Window};
use network_interface::{NetworkInterface, NetworkInterfaceConfig, Addr};
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::net::SocketAddr;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use socket2::{Socket, Domain, Type, Protocol};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Boards {
    Metis,
    Hermes,
    Hermes2,
    Angelia,
    Orion,
    Orion2,
    Saturn,
    HermesLite,
    HermesLite2,
    Unknown,
}

#[derive(Copy, Clone, Debug)]
pub struct Device {
    pub address: SocketAddr,
    pub my_address: SocketAddr,
    pub device: u8, // protocol relative
    pub board: Boards, // system relative
    pub protocol: u8,
    pub version: u8,
    pub status: u8,
    pub mac: [u8;6],
    pub supported_receivers: u8,
    pub supported_transmitters: u8,
    pub adcs: u8,
    pub frequency_min: u64,
    pub frequency_max: u64,
}


fn add_device(devices: Rc<RefCell<Vec<Device>>>, address: SocketAddr, my_address: SocketAddr, device: u8, board: Boards,protocol: u8,version:u8,status: u8,mac: [u8;6],supported_receivers: u8,supported_transmitters: u8,adcs: u8,frequency_min: u64,frequency_max: u64) {
    devices.borrow_mut().push(Device{address,my_address,device,board,protocol,version,status,mac,supported_receivers,supported_transmitters,adcs,frequency_min,frequency_max});
}

pub fn protocol1_discovery(devices: Rc<RefCell<Vec<Device>>>, socket_addr: SocketAddr) {
    eprintln!("protocol1_discovery: address={:?}", socket_addr);
    let setup_socket = Socket::new(Domain::for_address(socket_addr), Type::DGRAM, Some(Protocol::UDP)).expect("Socket::new failed");
    setup_socket.set_broadcast(true).expect("set_braodcast failed");
    setup_socket.set_read_timeout(Some(Duration::from_millis(250))).expect("set_read_timeout call failed");
    setup_socket.set_write_timeout(Some(Duration::from_millis(250))).expect("set_write_timeout call failed");
    setup_socket.set_reuse_address(true).expect("set_reuse_address failed");
    #[cfg(unix)]
    {
        setup_socket.set_reuse_port(true).expect("set_reuse_port failed");
    }
    setup_socket.bind(&socket_addr.into()).expect("bind failed");
    let socket: UdpSocket = setup_socket.into();

    
    let mut buf = [0u8; 63];
    buf[0] = 0xEF;
    buf[1] = 0xFE;
    buf[2] = 0x02;
    socket.send_to(&buf, "255.255.255.255:1024").expect("couldn't send data");

    loop {

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((amt,src)) => {
                let local_addr = socket.local_addr().expect("failed to get local address");
                if amt == 60  && src.port()==1024 {
                    let mac: [u8;6] = [buf[3],buf[4],buf[5],buf[6],buf[7],buf[8]];
                    let mut board = Boards::Unknown; 
                    let mut adcs=1;
                    let mut supported_receivers=1;
                    let mut supported_transmitters=1;
                    let mut frequency_min=0;
                    let mut frequency_max=61440000;

                    match  buf[10] {
                        0=>{ // METIS
                           board = Boards::Metis;
                           adcs=1;
                           supported_receivers=5;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        1=>{ // HERMES
                           board = Boards::Hermes;
                           adcs=1;
                           supported_receivers=5;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        4=>{ // ANGELIA
                           board = Boards::Angelia;
                           adcs=2;
                           supported_receivers=7;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        5=>{ // ORION
                           board = Boards::Orion;
                           adcs=2;
                           supported_receivers=7;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        6=>{ // HERMES_LITE
                           if buf[9] < 42 {
                               // HERMES_LITE V1
                               board = Boards::HermesLite;
                               supported_receivers=2;
                           } else {
                               // HERMES_LITE V2
                               board = Boards::HermesLite2;
                               supported_receivers=buf[19];
                           }
                           adcs=1;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=30720000;
                           },
                        10=>{ // ORION2
                           board = Boards::Orion2;
                           adcs=2;
                           supported_receivers=7;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        _=>{ // UNKNOWN - use defaults
                           },
                    }
                    add_device(Rc::clone(&devices),src,local_addr,buf[10],board,1,buf[9],buf[2],mac,supported_receivers,supported_transmitters,adcs,frequency_min,frequency_max);
                } 
            },
            Err(_e) => {
                break;
            }
        }
    }
    drop(socket);
}

pub fn protocol2_discovery(devices: Rc<RefCell<Vec<Device>>>, socket_addr: SocketAddr) {
    eprintln!("protocol2_discovery: address={:?}", socket_addr);
    let setup_socket = Socket::new(Domain::for_address(socket_addr), Type::DGRAM, Some(Protocol::UDP)).expect("Socket::new failed");
    setup_socket.set_broadcast(true).expect("set_braodcast failed");
    setup_socket.set_read_timeout(Some(Duration::from_millis(250))).expect("set_read_timeout call failed");
    setup_socket.set_write_timeout(Some(Duration::from_millis(250))).expect("set_write_timeout call failed");
    setup_socket.set_reuse_address(true).expect("set_reuse_address failed");
    #[cfg(unix)]
    {
        setup_socket.set_reuse_port(true).expect("set_reuse_port failed");
    }
    setup_socket.bind(&socket_addr.into()).expect("bind failed");
    let socket: UdpSocket = setup_socket.into();

    let mut buf = [0u8; 60];
    buf[4] = 0x02;
    socket.send_to(&buf, "255.255.255.255:1024").expect("couldn't send data");

    loop {

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((amt,src)) => {
                let local_addr = socket.local_addr().expect("failed to get local address");
                if amt == 60  && src.port() == 1024 {
                    let mac: [u8;6] = [buf[5],buf[6],buf[7],buf[8],buf[9],buf[10]];
                    let mut board = Boards::Unknown;
                    let mut adcs=1;
                    let mut supported_receivers=1;
                    let mut supported_transmitters=1;
                    let mut frequency_min=0;
                    let mut frequency_max=61440000;

                    match  buf[11] {
                        0=>{ // ATLAS
                            board = Boards::Metis;
                            adcs=1;
                            supported_receivers=5;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        1=>{ // HERMES
                            board = Boards::Hermes;
                            adcs=1;
                            supported_receivers=5;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        2=>{ // HERMES2
                            board = Boards::Hermes2;
                            adcs=1;
                            supported_receivers=5;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        3=>{ // ANGELIA
                            board = Boards::Angelia;
                            adcs=2;
                            supported_receivers=7;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        4=>{ // ORION
                            board = Boards::Orion;
                            adcs=2;
                            supported_receivers=7;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        5=>{ // ORION2
                            board = Boards::Orion2;
                            adcs=2;
                            supported_receivers=7;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        6=>{ // HERMES_LITE
                            board = Boards::HermesLite2;
                            adcs=1;
                            supported_receivers=5;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=30720000;
                           },
                        10=>{ // SATURN
                            board = Boards::Saturn;
                            adcs=2;
                            supported_receivers=7;
                            supported_transmitters=1;
                            frequency_min=0;
                            frequency_max=61440000;
                           },
                        _=>{ // UNKNOWN - use defaults
                           },
                    }
                    let local_addr = socket.local_addr().expect("failed to get local address");
                    add_device(Rc::clone(&devices),src,local_addr,buf[11],board,2,buf[13],buf[4],mac,supported_receivers,supported_transmitters,adcs,frequency_min,frequency_max);
                } else {
                    //println!("Expected 60 bytes but Received: {:?} From {:?}",amt,src);
                }
            },
            Err(_e) => {
                break;
            }
        }
    }
    drop(socket);
}

pub fn discover(devices: Rc<RefCell<Vec<Device>>>) {
    devices.borrow_mut().clear();
    let network_interfaces = NetworkInterface::show().unwrap();
    for itf in network_interfaces {
        for addr in &itf.addr {
            match addr {
                Addr::V4(v4_info) => {
                    let ip = v4_info.ip;
                    eprintln!("IPv4 Address {}",ip);
                    if let Ok(socket) = std::net::UdpSocket::bind((ip,5000)) {
                        eprintln!("Interface {} is active and bindable on {}", itf.name, ip);
                        let socket_address = SocketAddr::new(std::net::IpAddr::V4(ip), 50000);
                        protocol1_discovery(Rc::clone(&devices), socket_address);
                        protocol2_discovery(Rc::clone(&devices), socket_address);
                    } else {
                        eprintln!("Interface {} is NOT active", itf.name);
                    }
                    },
                _ => {}, // ignore all others
            }
        }
    }
}

pub fn manual_discovery(devices: Rc<RefCell<Vec<Device>>>, target_ip: std::net::IpAddr) -> bool {
    let socket_addr: SocketAddr = "0.0.0.0:0".parse().expect("Invalid Address");
    let setup_socket = Socket::new(Domain::for_address(socket_addr), Type::DGRAM, Some(Protocol::UDP)).expect("Socket::new failed");
    setup_socket.set_broadcast(false).expect("set_braodcast failed");
    setup_socket.set_read_timeout(Some(Duration::from_millis(250))).expect("set_read_timeout call failed");
    setup_socket.set_write_timeout(Some(Duration::from_millis(250))).expect("set_write_timeout call failed");
    setup_socket.set_reuse_address(true).expect("set_reuse_address failed");
    #[cfg(unix)]
    {
        setup_socket.set_reuse_port(true).expect("set_reuse_port failed");
    }
    setup_socket.bind(&socket_addr.into()).expect("bind failed");
    let socket: UdpSocket = setup_socket.into();

    let target_addr = SocketAddr::new(target_ip, 1024);

    // Try protocol 1 first
    let mut buf = [0u8; 63];
    buf[0] = 0xEF;
    buf[1] = 0xFE;
    buf[2] = 0x02;
    socket.send_to(&buf, target_addr).expect("couldn't send data");

    let mut found = false;
    loop {
        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((amt,src)) => {
                if amt == 60 && src.ip() == target_ip {
                    let mac: [u8;6] = [buf[3],buf[4],buf[5],buf[6],buf[7],buf[8]];
                    let mut board = Boards::Unknown;
                    let mut adcs=1;
                    let mut supported_receivers=1;
                    let mut supported_transmitters=1;
                    let mut frequency_min=0;
                    let mut frequency_max=61440000;

                    match  buf[10] {
                        0=>{ // METIS
                           board = Boards::Metis;
                           adcs=1;
                           supported_receivers=5;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        1=>{ // HERMES
                           board = Boards::Hermes;
                           adcs=1;
                           supported_receivers=5;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        4=>{ // ANGELIA
                           board = Boards::Angelia;
                           adcs=2;
                           supported_receivers=7;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        5=>{ // ORION
                           board = Boards::Orion;
                           adcs=2;
                           supported_receivers=7;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        6=>{ // HERMES_LITE
                           if buf[9] < 42 {
                               // HERMES_LITE V1
                               board = Boards::HermesLite;
                               supported_receivers=2;
                           } else {
                               // HERMES_LITE V2
                               board = Boards::HermesLite2;
                               supported_receivers=buf[19];
                           }
                           adcs=1;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=30720000;
                           },
                        10=>{ // ORION2
                           board = Boards::Orion2;
                           adcs=2;
                           supported_receivers=7;
                           supported_transmitters=1;
                           frequency_min=0;
                           frequency_max=61440000;
                           },
                        _=>{ // UNKNOWN - use defaults
                           },
                    }
                    let local_addr = socket.local_addr().expect("failed to get local address");
                    add_device(Rc::clone(&devices),src,local_addr,buf[10],board,1,buf[9],buf[2],mac,supported_receivers,supported_transmitters,adcs,frequency_min,frequency_max);
                    found = true;
                    break;
                }
            },
            Err(_e) => {
                break;
            }
        }
    }

    // If not found with protocol 1, try protocol 2
    if !found {
        let mut buf = [0u8; 60];
        buf[4] = 0x02;
        socket.send_to(&buf, target_addr).expect("couldn't send data");

        loop {
            let mut buf = [0; 1024];
            match socket.recv_from(&mut buf) {
                Ok((amt,src)) => {
                    if amt == 60 && src.ip() == target_ip {
                        let mac: [u8;6] = [buf[5],buf[6],buf[7],buf[8],buf[9],buf[10]];
                        let mut board = Boards::Unknown;
                        let mut adcs=1;
                        let mut supported_receivers=1;
                        let mut supported_transmitters=1;
                        let mut frequency_min=0;
                        let mut frequency_max=61440000;

                        match  buf[11] {
                            0=>{ // ATLAS
                                board = Boards::Metis;
                                adcs=1;
                                supported_receivers=5;
                                supported_transmitters=1;
                                frequency_min=0;
                                frequency_max=61440000;
                               },
                            1=>{ // HERMES
                                board = Boards::Hermes;
                                adcs=1;
                                supported_receivers=5;
                                supported_transmitters=1;
                                frequency_min=0;
                                frequency_max=61440000;
                               },
                            2=>{ // HERMES2
                                board = Boards::Hermes2;
                                adcs=1;
                                supported_receivers=5;
                                supported_transmitters=1;
                                frequency_min=0;
                                frequency_max=61440000;
                               },
                            3=>{ // ANGELIA
                                board = Boards::Angelia;
                                adcs=2;
                                supported_receivers=7;
                                supported_transmitters=1;
                                frequency_min=0;
                                frequency_max=61440000;
                               },
                            4=>{ // ORION
                                board = Boards::Orion;
                                adcs=2;
                                supported_receivers=7;
                                supported_transmitters=1;
                                frequency_min=0;
                                frequency_max=61440000;
                               },
                            5=>{ // ORION2
                                board = Boards::Orion2;
                                adcs=2;
                                supported_receivers=7;
                                supported_transmitters=1;
                                frequency_min=0;
                                frequency_max=61440000;
                               },
                            6=>{ // HERMES_LITE
                                board = Boards::HermesLite2;
                                adcs=1;
                                supported_receivers=5;
                                supported_transmitters=1;
                                frequency_min=0;
                                frequency_max=30720000;
                               },
                            10=>{ // SATURN
                                board = Boards::Saturn;
                                adcs=2;
                                supported_receivers=7;
                                supported_transmitters=1;
                                frequency_min=0;
                                frequency_max=61440000;
                               },
                            _=>{ // UNKNOWN - use defaults
                               },
                        }
                        let local_addr = socket.local_addr().expect("failed to get local address");
                        add_device(Rc::clone(&devices),src,local_addr,buf[11],board,2,buf[13],buf[4],mac,supported_receivers,supported_transmitters,adcs,frequency_min,frequency_max);
                        found = true;
                        break;
                    }
                },
                Err(_e) => {
                    break;
                }
            }
        }
    }

    drop(socket);
    found
}

pub fn create_discovery_dialog(parent: &ApplicationWindow, discovery_data: Rc<RefCell<Vec<Device>>>, selected_index: Rc<RefCell<Option<i32>>>) -> Window {

    let ui_xml = include_str!("../ui/discovery.xml");
    let builder = Builder::from_string(ui_xml);

    let window: Window = builder
            .object("discovery_window")
            .expect("Could not get object `discovery_window` from builder.");

    window.set_transient_for(Some(parent));
    window.set_modal(true);


    let list: ListBox = builder
            .object("list")
            .expect("Could not get object `list` from builder.");

    list.set_vexpand(true);

    let discovery_data_clone = Rc::clone(&discovery_data);
    discover(discovery_data_clone);
    populate_list_box(&list.clone(), Rc::clone(&discovery_data));
    if !discovery_data.borrow().is_empty() {
        if let Some(first_radio_row) = list.row_at_index(1) {
            first_radio_row.activate();
        }
    }

    let rediscover_button: Button = builder
            .object("rediscover_button")
            .expect("Could not get object `rediscover_button` from builder.");

    let cancel_button: Button = builder
            .object("cancel_button")
            .expect("Could not get object `cancel_button` from builder.");

    let manual_ip_entry: gtk::Entry = builder
            .object("manual_ip_entry")
            .expect("Could not get object `manual_ip_entry` from builder.");

    let add_manual_button: Button = builder
            .object("add_manual_button")
            .expect("Could not get object `add_manual_button` from builder.");

    let start_button: Button = builder
            .object("start_button")
            .expect("Could not get object `start_button` from builder.");
    if discovery_data.borrow().len() <=0 {
        start_button.set_sensitive(false);
    }

    let window_clone = window.clone();
    let selected_index_for_start = selected_index.clone();
    let list_clone = list.clone();
    start_button.connect_clicked(move |_| {
        if let Some(selected_row) = list_clone.selected_row() {
            let index = selected_row.index();
            *selected_index_for_start.borrow_mut() = Some(index);
        } else {
            *selected_index_for_start.borrow_mut() = None;
        }
        window_clone.close();
    });

    let window_clone = window.clone();
    cancel_button.connect_clicked(move |_| {
        window_clone.close();
    });

    let start_button_clone = start_button.clone();
    let discovery_data_for_rediscover = Rc::clone(&discovery_data);
    let list_for_rediscover = list.clone();
    rediscover_button.connect_clicked(move |_| {
        let discovery_data_clone_clone = Rc::clone(&discovery_data_for_rediscover);
        discover(discovery_data_clone_clone);
        populate_list_box(&list_for_rediscover.clone(), Rc::clone(&discovery_data_for_rediscover));
        if !discovery_data_for_rediscover.borrow().is_empty() {
            if let Some(first_radio_row) = list_for_rediscover.row_at_index(1) {
                first_radio_row.activate();
            }
            start_button_clone.set_sensitive(true);
        } else {
            start_button_clone.set_sensitive(false);
        }
    });

    let start_button_clone = start_button.clone();
    let manual_ip_entry_clone = manual_ip_entry.clone();
    let discovery_data_for_manual = Rc::clone(&discovery_data);
    let list_for_manual = list.clone();
    add_manual_button.connect_clicked(move |_| {
        let ip_text = manual_ip_entry_clone.text().to_string();
        if let Ok(ip) = ip_text.parse::<std::net::IpAddr>() {
            let discovery_data_clone_clone = Rc::clone(&discovery_data_for_manual);
            if manual_discovery(discovery_data_clone_clone, ip) {
                populate_list_box(&list_for_manual.clone(), Rc::clone(&discovery_data_for_manual));
                if !discovery_data_for_manual.borrow().is_empty() {
                    if let Some(first_radio_row) = list_for_manual.row_at_index(1) {
                        first_radio_row.activate();
                    }
                    start_button_clone.set_sensitive(true);
                }
            } else {
                // Could show an error dialog, but for now just do nothing
            }
        } else {
            // Invalid IP, could show error
        }
    });

    // following code to fix problem with keep_on_top for modal windows as dialog boxes
    let window_for_focus = window.clone();
    parent.connect_notify_local(Some("is-active"), move |_parent, _| {
        if window_for_focus.is_visible() {
            window_for_focus.present();
        }
    });

    window
}

pub fn device_name(device: Device) -> String {
    let board = format!("{:?}", device.board);
    board
}

fn populate_list_box(list: &ListBox, discovery_data: Rc<RefCell<Vec<Device>>>) {

    // Remove any existing rows
    while let Some(row) = list.row_at_index(0) {
        list.remove(&row);
    }

    // add the Devices
    let header = create_discovery_row(&["Device", "IFace", "IP", "MAC", "Protocol", "Version", "Status"], true);
    header.set_sensitive(false); // Disable selection
    list.append(&header);

    let discovery_iter = discovery_data.borrow().clone().into_iter();
    for val in discovery_iter {
        let radio = device_name(val);
        let iface=format!("{}",val.my_address.ip());
        let ip=format!("{}",val.address.ip());
        let mac=format!("{:02X?}",val.mac);
        let protocol=format!("{}",val.protocol);
        let version=format!("{}.{}",val.version/10,val.version%10);
        let mut status = "Unknown";
        if val.status == 2 {
            status = "Available";
        } else if val.status == 3 {
            status = "In Use";
        }

        let row = create_discovery_row(&[&radio, &iface, &ip, &mac, &protocol, &version, status], false);
        if val.status != 2 {
            row.set_sensitive(false); // Disable selection if in use
        }
        list.append(&row);
    }
}

fn create_discovery_row(columns: &[&str], is_header: bool) -> ListBoxRow {
    let row = ListBoxRow::new();
    let hbox = gtk::Box::new(Orientation::Horizontal, 10);

    let mut col = 0;
    for text in columns {
        let label = Label::new(Some(text));
        label.set_xalign(0.0); // Align text to the left
        if is_header {
            label.add_css_class("heading");
        }
        if col==0 {
            label.set_size_request(90,-1);
        } else if col==1 || col==2 {
            label.set_size_request(110,-1);
        } else if col==3 {
            label.set_size_request(160,-1);
        } else {
            label.set_size_request(70,-1);
        }
        hbox.append(&label);
        col += 1;
    }

    row.set_child(Some(&hbox));
    row
}
