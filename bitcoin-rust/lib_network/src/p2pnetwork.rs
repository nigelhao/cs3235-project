// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

use crate::netchannel::*;
/// P2PNetwork is a struct that implements a peer-to-peer network.
/// It is used to send and receive messages to/from neighbors.
/// It also automatically broadcasts messages.
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the P2PNetwork.
use lib_chain::block::{BlockId, BlockNode, Transaction, TxId};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// The struct to represent statistics of a peer-to-peer network.
pub struct P2PNetwork {
    /// The number of messages sent by this node.
    pub send_msg_count: u64,
    /// The number of messages received by this node.
    pub recv_msg_count: u64,
    /// The address of this node.
    pub address: NetAddress,
    /// The addresses of the neighbors.
    pub neighbors: Vec<NetAddress>,
}

impl P2PNetwork {
    /// Creates a new P2PNetwork instance and associated FIFO communication channels.
    /// There are 5 FIFO channels.
    /// Those channels are used for communication within the process.
    /// They abstract away the network and neighbor nodes.
    /// More specifically, they are for communicating between `bin_nakamoto` threads
    /// and threads that are responsible for TCP network communication.
    /// The usage of those five channels can be guessed from the type:
    /// 1. Receiver<BlockNode>: read from this FIFO channel to receive blocks from the network.
    /// 2. Receiver<Transaction>: read from this FIFO channel to receive transactions from the network.
    /// 3. Sender<BlockNode>: write to this FIFO channel to broadcast a block to the network.
    /// 4. Sender<Transaction>: write to this FIFO channel to broadcast a transaction to the network.
    /// 5. Sender<BlockId>: write to this FIFO channel to request a block from the network.
    pub fn create(
        address: NetAddress,
        neighbors: Vec<NetAddress>,
    ) -> (
        Arc<Mutex<P2PNetwork>>,
        Receiver<BlockNode>,
        Receiver<Transaction>,
        Sender<BlockNode>,
        Sender<Transaction>,
        Sender<BlockId>,
    ) {
        // Please fill in the blank
        // You might need to perform the following steps:
        // 1. create a P2PNetwork instance
        // 2. create mpsc channels for sending and receiving messages
        // 3. create a thread for accepting incoming TCP connections from neighbors
        // 4. create TCP connections to all neighbors
        // 5. create threads for each TCP connection to send messages
        // 6. create threads to listen to messages from neighbors
        // 7. create threads to distribute received messages (send to channels or broadcast to neighbors)
        // 8. return the created P2PNetwork instance and the mpsc channels
            

        // 1. create a P2PNetwork instance
        let network = Arc::new(Mutex::new(P2PNetwork {
            send_msg_count: 0,
            recv_msg_count: 0,
            address: address.clone(),
            neighbors: neighbors.clone(),
        }));


        // 2. create mpsc channels for sending and receiving messages
        let (block_node_sender, block_node_receiver): (Sender<BlockNode>, Receiver<BlockNode>) = mpsc::channel();
        let (tx_sender, tx_receiver): (Sender<Transaction>, Receiver<Transaction>) = mpsc::channel();
        let (block_id_sender, _): (Sender<BlockId>, Receiver<BlockId>) = mpsc::channel();


        // 3. create a thread for accepting incoming TCP connections from neighbors
        let connect_to_neighbours = thread::spawn(move || { 

            // 4. create TCP connections to all neighbors
            for neighbor in neighbors {
                
                match TcpStream::connect(format!("{}:{}", neighbor.ip, neighbor.port)) {
                   
                    Ok(stream) => {

                        println!("Connected to {}:{}", neighbor.ip, neighbor.port);
               
                        let channel = NetChannelTCP::from_stream(stream);
                        
                        // 5. create threads for each TCP connection to send messages
                        thread::spawn(move || {
                            
                            // Continuously check for messages to send
                            // If there is a message to be sent, send the message, then increment send_msg_count
                            loop {
                                
                            }
                            
                        });

                        // 6. create threads to listen to messages from neighbors
                        thread::spawn(move || {
                            
                            // Continuously check for messages to receive
                            // If there is a message to be received, receive the message, then increment recv_msg_count
                            loop {

                            }
                            
                        });

                        // 7. create threads to distribute received messages (send to channels or broadcast to neighbors)
                        // How is this different from 5?
                        thread::spawn(move || {
                            
                            // Continuously check for messages to broadcast after receiving a message
                            // If there is a message to be broadcast, broadcast the message, then increment send_msg_count
                            loop {

                            }
                            
                        });

                    }
               
                    Err(e) => {
                        eprintln!("Failed to connect to {}:{}: {}", neighbor.ip, neighbor.port, e);
                    }
                }
            }

            println!("[P2PNetwork] All neighbors connected.");
            println!("[P2PNetwork] Starting processing received messages thread.");

        });


        // 8. return the created P2PNetwork instance and the mpsc channels
        return (
            network,
            block_node_receiver,
            tx_receiver,
            block_node_sender,
            tx_sender,
            block_id_sender,
        )
        

    }

    /// Get status information of the P2PNetwork for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the network.
        // It should be displayed in the Client UI eventually.
        todo!();
    }
}
