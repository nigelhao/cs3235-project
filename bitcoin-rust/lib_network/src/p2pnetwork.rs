// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

use crate::netchannel::*;
use crate::netchannel::*;
/// P2PNetwork is a struct that implements a peer-to-peer network.
/// It is used to send and receive messages to/from neighbors.
/// It also automatically broadcasts messages.
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the P2PNetwork.
use lib_chain::block::{self, BlockId, BlockNode, Transaction, TxId};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert;
use std::net::TcpListener;
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
        // 1. create a P2PNetwork instance X
        // 2. create mpsc channels for sending and receiving messages X
        // 3. create a thread for accepting incoming TCP connections from neighbors
        // 4. create TCP connections to all neighbors
        // 5. create threads for each TCP connection to send messages
        // 6. create threads to listen to messages from neighbors
        // 7. create threads to distribute received messages (send to channels or broadcast to neighbors)
        // 8. return the created P2PNetwork instance and the mpsc channels

        // 1. create a P2PNetwork instance
        let p2p_network = Arc::new(Mutex::new(P2PNetwork {
            send_msg_count: 0,
            recv_msg_count: 0,
            address: address.clone(),
            neighbors: neighbors.clone(),
        }));

        // 2. create mpsc channels for sending and receiving messages
        //Read block_node from p2p network
        let (block_node_r_sender, block_node_r_receiver): (Sender<BlockNode>, Receiver<BlockNode>) =
            mpsc::channel();
        //Send block_node to p2p nework
        let (block_node_w_sender, block_node_w_receiver): (Sender<BlockNode>, Receiver<BlockNode>) =
            mpsc::channel();

        //Read tx from p2p network
        let (tx_r_sender, tx_r_receiver): (Sender<Transaction>, Receiver<Transaction>) =
            mpsc::channel();
        //Send tx to p2p network
        let (tx_w_sender, tx_w_receiver): (Sender<Transaction>, Receiver<Transaction>) =
            mpsc::channel();

        let (block_id_sender, block_id_receiver): (Sender<BlockId>, Receiver<BlockId>) =
            mpsc::channel();

        let ack: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
        let channels: Arc<Mutex<Vec<NetChannelTCP>>> = Arc::new(Mutex::new(Vec::new()));

        let channels_thread = Arc::clone(&channels);
        let neighbors_connected_thread = thread::spawn(move || {
            for neighbor in neighbors {
                let net_channel = NetChannelTCP::from_addr(&neighbor).unwrap();
                channels_thread.lock().unwrap().push(net_channel);
            }
            println!("[P2PNetwork] All neighbors connected.");
        });

        neighbors_connected_thread.join().unwrap();

        let ack_thread = Arc::clone(&ack);
        let channels_thread = Arc::clone(&channels);
        let p2p_network_thread = Arc::clone(&p2p_network);
        let block_node_w_sender_thread = block_node_w_sender.clone();
        let tx_w_sender_thread = tx_w_sender.clone();
        let block_id_sender_thread = block_id_sender.clone();
        thread::spawn(move || {
            println!("[P2PNetwork] Starting processing received messages thread.");

            let listener =
                TcpListener::bind(&format!("{}:{}", address.ip.clone(), address.port.clone()))
                    .unwrap();

            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    let net_channel = NetChannelTCP::from_stream(stream);

                    let ack_thread = Arc::clone(&ack_thread);
                    let channels_thread = Arc::clone(&channels_thread);
                    let p2p_network_thread = Arc::clone(&p2p_network_thread);
                    let block_node_w_sender_thread = block_node_w_sender_thread.clone();
                    let tx_w_sender_thread = tx_w_sender_thread.clone();
                    let block_id_sender_thread = block_id_sender_thread.clone();

                    thread::spawn(move || {
                        let mut net_channel_read = net_channel;
                        while let Some(message) = net_channel_read.read_msg() {
                            //Prevent broadcasting previously broadcasted messages
                            let mut ack = ack_thread.lock().unwrap();
                            p2p_network_thread.lock().unwrap().recv_msg_count += 1;

                            let message_str = format!("{:?}", message);
                            if ack.contains(&message_str) {
                                drop(ack);
                                continue;
                            } else {
                                ack.insert(message_str);
                                drop(ack);
                            }

                            match message {
                                NetMessage::BroadcastBlock(block_node) => {
                                    block_node_w_sender_thread.send(block_node.clone()).unwrap();

                                    for channel in channels_thread.lock().unwrap().iter_mut() {
                                        let mut net_channel_write = channel.clone_channel();
                                        let message =
                                            NetMessage::BroadcastBlock(block_node.clone());
                                        net_channel_write.write_msg(message);
                                        p2p_network_thread.lock().unwrap().send_msg_count += 1;
                                    }
                                }
                                NetMessage::BroadcastTx(transaction) => {
                                    tx_w_sender_thread.send(transaction.clone()).unwrap();

                                    for channel in channels_thread.lock().unwrap().iter_mut() {
                                        let mut net_channel_write = channel.clone_channel();
                                        let message = NetMessage::BroadcastTx(transaction.clone());
                                        net_channel_write.write_msg(message);
                                        p2p_network_thread.lock().unwrap().send_msg_count += 1;
                                    }
                                }
                                NetMessage::RequestBlock(block_id) => {
                                    block_id_sender_thread.send(block_id.clone()).unwrap();

                                    for channel in channels_thread.lock().unwrap().iter_mut() {
                                        let mut net_channel_write = channel.clone_channel();
                                        let message = NetMessage::RequestBlock(block_id.clone());
                                        net_channel_write.write_msg(message);
                                        p2p_network_thread.lock().unwrap().send_msg_count += 1;
                                    }
                                }
                                NetMessage::Unknown(_) => {}
                            }
                        }
                    });
                }
            }
        });

        // TODO: Verify if messages from neighbor channel is needed?
        let ack_thread = Arc::clone(&ack);
        let channels_thread = Arc::clone(&channels);
        let p2p_network_thread = Arc::clone(&p2p_network);
        thread::spawn(move || {
            for channel_read in channels_thread.lock().unwrap().iter_mut() {
                let mut net_channel_read = channel_read.clone_channel();

                let ack_thread = Arc::clone(&ack_thread);
                let channels_thread_tmp = Arc::clone(&channels_thread);
                let p2p_network_thread = Arc::clone(&p2p_network_thread);
                thread::spawn(move || loop {
                    let message = net_channel_read.read_msg().unwrap();
                    p2p_network_thread.lock().unwrap().recv_msg_count += 1;

                    //Prevent broadcasting previously broadcasted messages
                    let mut ack = ack_thread.lock().unwrap();
                    let message_str = format!("{:?}", message);
                    if ack.contains(&message_str) {
                        drop(ack);
                        continue;
                    } else {
                        ack.insert(message_str);
                        drop(ack);
                    }

                    match message {
                        NetMessage::BroadcastBlock(block_node) => {
                            for channel_write in channels_thread_tmp.lock().unwrap().iter_mut() {
                                let mut net_channel_write = channel_write.clone_channel();
                                let message = NetMessage::BroadcastBlock(block_node.clone());
                                net_channel_write.write_msg(message);
                                p2p_network_thread.lock().unwrap().send_msg_count += 1;
                            }
                        }
                        NetMessage::BroadcastTx(transaction) => {
                            for channel_write in channels_thread_tmp.lock().unwrap().iter_mut() {
                                let mut net_channel_write = channel_write.clone_channel();
                                let message = NetMessage::BroadcastTx(transaction.clone());
                                net_channel_write.write_msg(message);
                                p2p_network_thread.lock().unwrap().send_msg_count += 1;
                            }
                        }
                        NetMessage::RequestBlock(block_id) => {
                            for channel_write in channels_thread_tmp.lock().unwrap().iter_mut() {
                                let mut net_channel_write = channel_write.clone_channel();
                                let message = NetMessage::RequestBlock(block_id.clone());
                                net_channel_write.write_msg(message);
                                p2p_network_thread.lock().unwrap().send_msg_count += 1;
                            }
                        }
                        NetMessage::Unknown(_) => {}
                    }
                });
            }
        });

        let channels_thread = Arc::clone(&channels);
        let p2p_network_thread = Arc::clone(&p2p_network);
        thread::spawn(move || {
            println!("[P2PNetwork] Starting broadcasting blocks thread.");

            loop {
                if let Ok(block_node) = block_node_r_receiver.recv() {
                    // Do something with the received message
                    for channel in channels_thread.lock().unwrap().iter_mut() {
                        let mut net_channel = channel.clone_channel();
                        let message = block_node.clone();
                        let p2p_network_thread = Arc::clone(&p2p_network_thread);

                        thread::spawn(move || {
                            let message = NetMessage::BroadcastBlock(message);
                            net_channel.write_msg(message);
                            p2p_network_thread.lock().unwrap().send_msg_count += 1;
                        });
                    }
                }
            }
        });

        let channels_thread = Arc::clone(&channels);
        let p2p_network_thread = Arc::clone(&p2p_network);
        thread::spawn(move || {
            println!("[P2PNetwork] Starting broadcasting transactions thread.");

            loop {
                if let Ok(tx) = tx_r_receiver.recv() {
                    // Do something with the received message
                    for channel in channels_thread.lock().unwrap().iter_mut() {
                        let mut net_channel = channel.clone_channel();
                        let message = tx.clone();
                        let p2p_network_thread = Arc::clone(&p2p_network_thread);

                        thread::spawn(move || {
                            let message = NetMessage::BroadcastTx(message);
                            net_channel.write_msg(message);
                            p2p_network_thread.lock().unwrap().send_msg_count += 1;
                        });
                    }
                }
            }
        });

        let channels_thread = Arc::clone(&channels);
        let p2p_network_thread = Arc::clone(&p2p_network);
        thread::spawn(move || loop {
            if let Ok(block_id) = block_id_receiver.recv() {
                // Do something with the received message
                for channel in channels_thread.lock().unwrap().iter_mut() {
                    let mut net_channel = channel.clone_channel();
                    let message = block_id.clone();
                    let p2p_network_thread = Arc::clone(&p2p_network_thread);

                    thread::spawn(move || {
                        let message = NetMessage::RequestBlock(message);
                        net_channel.write_msg(message);
                        p2p_network_thread.lock().unwrap().send_msg_count += 1;
                    });
                }
            }
        });

        return (
            p2p_network,
            block_node_w_receiver,
            tx_w_receiver,
            block_node_r_sender,
            tx_r_sender,
            block_id_sender,
        );
    }

    /// Get status information of the P2PNetwork for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the network.
        // It should be displayed in the Client UI eventually.
        let mut map = BTreeMap::new();
        map.insert(
            "#address".to_string(),
            format!(
                "ip: {} port: {}",
                self.address.ip.clone(),
                self.address.port.clone()
            ),
        );
        map.insert("#recv_msg".to_string(), self.recv_msg_count.to_string());
        map.insert("#send_msg".to_string(), self.send_msg_count.to_string());

        return map;
    }
}
