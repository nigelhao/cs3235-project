// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

pub mod netchannel;
pub mod p2pnetwork;



#[cfg(test)]
mod tests {
    use std::io::BufRead;
    use std::thread;
    use std::time::Duration;
    use lib_chain::block::{BlockNode, BlockNodeHeader, Transaction, Transactions, MerkleTree};
    use std::net::{TcpListener, TcpStream};
    use std::io::{Read, Write, BufReader};

    use crate::netchannel::{NetAddress, NetMessage, NetChannelTCP};
    use crate::p2pnetwork::{P2PNetwork};


    /// Test the NetChannelTCP by creating a fake node that echo messages and connecting to it.
    #[test]
    fn test_net_channel() {
        let fake_node1 = TcpListener::bind("127.0.0.1:9014").unwrap();
        let _fake_node1_handle = thread::spawn(move || {
            for stream in fake_node1.incoming() {
                println!("--- fake_node1 INCOMING STREAM ---");
                let mut stream = stream.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                // read and print lines
                loop {
                    let mut buf = String::new();
                    let len = reader.read_line(&mut buf).unwrap(); 
                    if len > 0 {
                        println!("[fake_node1] [Received] {}", &buf);
                        stream.write_all(buf.as_bytes()).unwrap();
                    } else {
                        break;
                    }
                }
            }
        });
        let mut net_channel = NetChannelTCP::from_addr(&NetAddress { ip: "127.0.0.1".to_owned(), port: 9014 }).unwrap();
        net_channel.write_msg(NetMessage::Unknown("hello".to_owned()));
        let msg = net_channel.read_msg().unwrap();
        println!("[Main thread] [Received] {:?}", &msg);
        assert!(msg == NetMessage::Unknown("hello".to_owned()));
    }


    /// A function for creating a simplified fake neighbor node for testing the P2PNetwork.
    fn fake_neighbor(mut stream: TcpStream) {
        println!("[fake_neighbor] [BEGIN]");
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let _handle_w = thread::spawn(move || {
            println!("[fake_neighbor] [Write thread]");
            let transaction = Transaction {
                sender: "AAA".to_string(),
                receiver: "DDD".to_string(),
                message: "good".to_string(),
                sig: "blabla".to_string()
            };
            let node_header = BlockNodeHeader {
                parent: "ZZZZ".to_string(),
                merkle_root: "12345".to_string(),
                timestamp: 345,
                block_id: String::new(),
                nonce: "98765".to_string(),
                reward_receiver: "AAA".to_string(),
            };
            let block = BlockNode {
                header: node_header,
                transactions_block:  Transactions { merkle_tree: MerkleTree {hashes: vec![]}, transactions: vec![]},
            };

            let mut jsonstr: String = serde_json::to_string(&NetMessage::BroadcastTx(transaction.clone())).unwrap();
            jsonstr.push('\n');
            stream.write_all(jsonstr.as_bytes()).unwrap();

            let mut jsonstr: String = serde_json::to_string(&NetMessage::BroadcastBlock(block.clone())).unwrap();
            jsonstr.push('\n');
            stream.write_all(jsonstr.as_bytes()).unwrap();
        });

        let _handle_r = thread::spawn(move || {
            println!("[fake_neighbor] [Read thread]");
            loop {
                let mut buf = String::new();
                let len = reader.read_line(&mut buf).unwrap(); 
                if len > 0 {
                    println!("{}", "[handle_client] [Read] ".to_owned() + &buf);
                } else {
                    break;
                }
            }
            println!("[p2p_fake_node] [END]");
        });

        _handle_w.join().unwrap();
        _handle_r.join().unwrap();
        println!("[handle_client] [END]");
    }

    /// Test the P2PNetwork by creating fake nodes and send messages.
    /// Notice that you need to compare the log with the expected output manually.
    /// The expected output is in the comments at the end of the test function.
    #[test]
    fn test_p2pnetwork() {
        // create fake nodes
        let fake_node1 = TcpListener::bind("127.0.0.1:9012").unwrap();
        let fake_node2 = TcpListener::bind("127.0.0.1:9013").unwrap();
        let _fake_node1_handle = thread::spawn(move || {
            for stream in fake_node1.incoming() {
                println!("--- fake_neighbor_1 INCOMING STREAM ---");
                fake_neighbor(stream.unwrap());
            }
        });
        let _fake_node2_handle = thread::spawn(move || {
            for stream in fake_node2.incoming() {
                println!("--- fake_neighbor_2 INCOMING STREAM ---");
                fake_neighbor(stream.unwrap());
            }
        });
        
        let (
            network,
            upd_block_in_rx, 
            upd_trans_in_rx,
            block_out_tx,
            trans_out_tx,
            req_block_id_out_tx,
        ) = P2PNetwork::create(
            NetAddress { ip: "127.0.0.1".to_owned(), port: 9011 },
            vec![
                NetAddress { ip: "127.0.0.1".to_owned(), port: 9012 },
                NetAddress { ip: "127.0.0.1".to_owned(), port: 9013 }
            ]
        );



        let _upd_block_in_listener_handle = thread::spawn(move || {
            for block in upd_block_in_rx {
                println!("[Main] [Get Block Update] {:?}", block);
            }
        });

        let _upd_trans_in_listener_handle = thread::spawn(move || {
            for trans in upd_trans_in_rx {
                println!("[Main][Get Trans update] {:?}", trans);
            }
        });


        let transaction = Transaction {
            sender: "hello".to_string(),
            receiver: "hi".to_string(),
            message: "msg".to_string(),
            sig: "sig".to_string()
        };
        let node_header = BlockNodeHeader {
            parent: "hahaha".to_string(),
            merkle_root: "0987".to_string(),
            timestamp: 123,
            block_id: String::new(),
            nonce: "1111".to_string(),
            reward_receiver: "AAA".to_string(),
        };
        let node = BlockNode {
            header: node_header,
            transactions_block:  Transactions { merkle_tree: MerkleTree {hashes: vec![]}, transactions: vec![]},
        };
        block_out_tx.send(node).unwrap();
        trans_out_tx.send(transaction).unwrap();

        thread::sleep(Duration::from_millis(200));

        // Expected Log
        // [NetChannel] Trying to connect to 127.0.0.1:9012
        // --- fake_neighbor_1 INCOMING STREAM ---
        // [NetChannel] Trying to connect to [fake_neighbor] [BEGIN]
        // 127.0.0.1:9013
        // [P2PNetwork] All neighbors connected.
        // [P2PNetwork] Starting processing received messages thread.
        // --- fake_neighbor_2 INCOMING STREAM ---
        // [fake_neighbor] [BEGIN]
        // [fake_neighbor] [Read thread]
        // [P2PNetwork] Starting broadcasting blocks thread.
        // [fake_neighbor] [Write thread]
        // [P2PNetwork] Starting broadcasting transactions thread.
        // [fake_neighbor] [Write thread]
        // [fake_neighbor] [Read thread]
        // [handle_client] [Read] {"BroadcastTx":{"sender":"hello","receiver":"hi","message":"msg","sig":"sig"}}
        // [handle_client] [Read] {"BroadcastBlock":{"header":{"parent":"hahaha","merkle_root":"0987","timestamp":123,"block_id":"","nonce":"1111"},"transactions_block":{"merkle_tree":{"hashes":[]},"transactions":[]}}}
        // [handle_client] [Read] {"BroadcastBlock":{"header":{"parent":"hahaha","merkle_root":"0987","timestamp":123,"block_id":"","nonce":"1111"},"transactions_block":{"merkle_tree":{"hashes":[]},"transactions":[]}}}
        // [handle_client] [Read] {"BroadcastTx":{"sender":"hello","receiver":"hi","message":"msg","sig":"sig"}}

    }


    /// Your own additional test that tests your implementation more throughly (e.g. stress-testing your implementation)
    #[test]
    fn test_p2pnetwork_additional() {
        // Please fill in the blank
        
    }
}







