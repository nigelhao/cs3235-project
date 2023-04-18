// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file mainly implements the NetChannelTCP struct and related methods.
// The NetChannelTCP struct is used to send and receive messages over the network.
// The message format is defined in the NetMessage enum.
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the NetChannelTCP.

use lib_chain::block::{BlockId, BlockNode, Transaction};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::io::BufRead;
use std::io::BufReader;
use std::io::{Read, Write};
use std::net::TcpStream;

/// The struct to represent a network address.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize, Debug)]
pub struct NetAddress {
    /// the ip address. Example: "127.0.0.1"
    pub ip: String,
    /// the port number. Example: 8000
    pub port: i32,
}

impl NetAddress {
    pub fn new(ip: String, port: i32) -> NetAddress {
        NetAddress { ip, port }
    }
}

/// The enum to represent a network message that is sent or received using `NetChannelTCP`.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum NetMessage {
    /// The message to broadcast a block to the neighbor.
    BroadcastBlock(BlockNode),
    /// The message to broadcast a transaction to the neighbor.
    BroadcastTx(Transaction),
    /// The message to request a block (i.e. missing in the local block tree) from neighbor.
    RequestBlock(BlockId),
    /// The message to represent other temporary messages (e.g. for debugging).
    Unknown(String),
}

/// The struct to represent a network channel that is used to send and receive messages to a neighbor node.
pub struct NetChannelTCP {
    /// The TCP stream
    stream: TcpStream,
    /// The reader to read from the TCP stream
    reader: BufReader<TcpStream>,
}

impl NetChannelTCP {
    /// Create a new NetChannelTCP from a NetAddress and establish the TCP connection.
    /// Return an error string if the connection fails.
    pub fn from_addr(addr: &NetAddress) -> Result<Self, String> {
        let addr_port = format!("{}:{}", addr.ip, addr.port);
        loop {
            match TcpStream::connect(&addr_port) {
                Ok(tcp_stream) => {
                    let buf_reader = BufReader::new(tcp_stream.try_clone().unwrap());
                    let net_channel = NetChannelTCP {
                        stream: tcp_stream,
                        reader: buf_reader,
                    };
                    println!("[NetChannel] Connected to {}:{}", addr.ip, addr.port);
                    return Ok(net_channel);
                }
                Err(e) => {
                    println!(
                        "[NetChannel] Failed to connect to {}: {}. Retrying in 1 second...",
                        addr_port, e
                    );
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        }
    }

    // pub fn from_addr(addr: &NetAddress) -> Result<Self, String> {
    //     // Please fill in the blank
    //     println!(
    //         "[NetChannel] Trying to connect to {}:{}",
    //         addr.ip, addr.port
    //     );

    //     let addr_port = format!("{}:{}", addr.ip, addr.port);
    //     match TcpStream::connect(&addr_port) {
    //         Ok(tcp_stream) => {
    //             let buf_reader = BufReader::new(tcp_stream.try_clone().unwrap());
    //             let net_channel = NetChannelTCP {
    //                 stream: tcp_stream,
    //                 reader: buf_reader,
    //             };
    //             Ok(net_channel)
    //         }
    //         Err(e) => Err(format!("Failed to connect to {}: {}", addr_port, e)),
    //     }
    // }

    /// Create a new NetChannelTCP from a TcpStream.
    /// This is useful for creating a NetChannelTCP instance from the listener side.
    pub fn from_stream(stream: TcpStream) -> Self {
        // Please fill in the blank
        let buf_reader = BufReader::new(stream.try_clone().unwrap());

        NetChannelTCP {
            stream: stream,
            reader: buf_reader,
        }
    }

    /// Clone the NetChannelTCP instance.
    /// This is useful if you have multiple threads dealing with reading and writing to the TCP channel.
    pub fn clone_channel(&mut self) -> Self {
        // Please fill in the blank
        let cloned_stream = self.stream.try_clone().expect("Failed to clone TCP stream");
        Self::from_stream(cloned_stream)
    }

    /// Read one line of message from the TCP stream.
    /// Return None if the stream is closed.
    /// Otherwise, parse the line as a NetMessage and return it.
    pub fn read_msg(&mut self) -> Option<NetMessage> {
        // Please fill in the blank

        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None, // Stream closed
            Ok(_) => {
                let deserialized_msg: Result<NetMessage, serde_json::Error> =
                    serde_json::from_str(&line);
                match deserialized_msg {
                    Ok(msg) => Some(msg),
                    Err(e) => {
                        eprintln!("Error deserializing message: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading message from stream: {}", e);
                None
            }
        }
    }

    /// Write a NetMessage to the TCP stream.
    /// The message is serialized to a one-line JSON string and a newline is appended in the end.
    pub fn write_msg(&mut self, msg: NetMessage) -> () {
        // Please fill in the blank
        let serialized_msg = format!("{}\n", serde_json::to_string(&msg).unwrap());
        if let Err(e) = self.stream.write_all(serialized_msg.as_bytes()) {
            eprintln!("Error writing message to stream: {}", e);
        }
    }
}
