// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file implements the Nakamoto struct, related data structs and methods.
// The Nakamoto leverages lib_chain, lib_miner, lib_tx_pool and lib_network to implement the Nakamoto consensus algorithm.
// You can see detailed instructions in the comments below.

use lib_chain::block::{
    BlockNode, BlockNodeHeader, BlockTree, MerkleTree, Puzzle, Transaction, Transactions,
};
use lib_miner::miner::{Miner, PuzzleSolution};
use lib_network::netchannel::NetAddress;
use lib_network::p2pnetwork::P2PNetwork;
use lib_tx_pool::pool::TxPool;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::{thread, time::Duration};

type UserId = String;

/// The struct to represent configuration of the Nakamoto instance.`
/// The configuration does not contain any user information. The Nakamoto algorithm is user-independent.
/// The configuration sets information about neighboring nodes, miner, block creation, etc.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// the list of addresses of neighboring nodes
    pub neighbors: Vec<NetAddress>,
    /// the address of this node
    pub addr: NetAddress,
    /// the number of threads used to mine a new block (for miner)
    pub miner_thread_count: u16,
    /// the length of the nonce string (for miner)
    pub nonce_len: u16,
    // difficulty to mine a new block (for miner)
    pub difficulty_leading_zero_len: u16,
    // difficulty to accept a new block (for verifying the block)
    pub difficulty_leading_zero_len_acc: u16,
    // the seed for the miner thread 0 (for miner)
    pub miner_thread_0_seed: u64,
    // the reward receiver (for mined blocks)
    pub mining_reward_receiver: UserId,
    // the max number of transactions in one block (for creating a new block)
    pub max_tx_in_one_block: u16,
}

/// Create a puzzle for the miner given a chain and a tx pool (as smart pointers).
/// It returns the puzzle (serialization of the Puzzle struct) and the corresponding incomplete block (nonce and block_id not filled)
fn create_puzzle(
    chain_p: Arc<Mutex<BlockTree>>,
    tx_pool_p: Arc<Mutex<TxPool>>,
    tx_count: u16,
    reward_receiver: UserId,
) -> (String, BlockNode) {
    // Please fill in the blank
    // Filter transactions from tx_pool and get the last node of the longest chain.

    let working_block_id = chain_p.lock().unwrap().working_block_id.clone();

    let excluding_txs = chain_p.lock().unwrap().get_pending_finalization_txs();

    let filtered_tx = tx_pool_p
        .lock()
        .unwrap()
        .filter_tx(tx_count, &excluding_txs);

    let (merkle_root, merkle_tree) = MerkleTree::create_merkle_tree(filtered_tx.clone());

    // build the puzzle
    let puzzle = Puzzle {
        // Please fill in the blank
        // Create a puzzle with the block_id of the parent node and the merkle root of the transactions.
        reward_receiver: reward_receiver.clone(),
        merkle_root: merkle_root.clone(),
        parent: working_block_id.clone(),
    };
    let puzzle_str = serde_json::to_string(&puzzle).unwrap().to_owned();

    // Please fill in the blank
    // Create a block node with the transactions and the merkle root.
    // Leave the nonce and the block_id empty (to be filled after solving the puzzle).
    // The timestamp can be set to any positive interger.

    let pre_block = BlockNode {
        header: BlockNodeHeader {
            parent: working_block_id,
            merkle_root: merkle_root,
            timestamp: 1,
            block_id: "".to_string(),
            nonce: "".to_string(),
            reward_receiver: reward_receiver,
        },
        transactions_block: Transactions {
            merkle_tree: merkle_tree,
            transactions: filtered_tx,
        },
    };

    // In the end, it returns  (puzzle_str, pre_block);
    return (puzzle_str, pre_block);
}

/// The struct to represent the Nakamoto instance.
/// The Nakamoto instance contains the chain, the miner, the network and the tx pool as smart pointers.
/// It also contains a FIFO channel for sending transactions to the Blockchain
pub struct Nakamoto {
    /// the chain (BlockTree)
    pub chain_p: Arc<Mutex<BlockTree>>,
    /// the miner
    pub miner_p: Arc<Mutex<Miner>>,
    /// the p2pnetwork
    pub network_p: Arc<Mutex<P2PNetwork>>,
    /// the transaction pool
    pub tx_pool_p: Arc<Mutex<TxPool>>,
    /// the FIFO channel for sending transactions to the Blockchain
    trans_tx: Sender<Transaction>,
}

impl Nakamoto {
    /// A function to send notification messages to stdout (For debugging purpose only)
    pub fn stdout_notify(msg: String) {
        let msg = HashMap::from([("Notify".to_string(), msg.clone())]);
        println!("{}", serde_json::to_string(&msg).unwrap());
    }

    /// Create a Nakamoto instance given the serialized chain, tx pool and config as three json strings.
    pub fn create_nakamoto(chain_str: String, tx_pool_str: String, config_str: String) -> Nakamoto {
        // Please fill in the blank
        // Deserialize the config from the given json string.
        let config: Config = serde_json::from_str(&config_str).unwrap();

        // Deserialize the chain and the tx pool from the given json strings.
        let chain: BlockTree = serde_json::from_str(&chain_str).unwrap();
        let tx_pool: TxPool = serde_json::from_str(&tx_pool_str).unwrap();

        // Create the miner and the network according to the config.
        let miner = Miner::new();
        let (
            network_p,
            upd_block_in_rx,
            upd_trans_in_rx,
            block_out_tx,
            trans_out_tx,
            req_block_id_out_tx,
        ) = P2PNetwork::create(config.clone().addr, config.clone().neighbors);

        let (trans_tx_sender, trans_tx_receiver): (Sender<Transaction>, Receiver<Transaction>) =
            mpsc::channel();

        let (miner_sender, miner_receiver): (Sender<bool>, Receiver<bool>) = mpsc::channel();

        let chain_p = Arc::new(Mutex::new(chain));
        let miner_p = Arc::new(Mutex::new(miner));
        let tx_pool_p = Arc::new(Mutex::new(tx_pool));

        let cancellation_token = Arc::new(RwLock::new(false));

        // Start necessary threads that read from and write to FIFO channels provided by the network.
        let tx_pool_p_thread = Arc::clone(&tx_pool_p);
        let trans_tx_sender_thread = trans_tx_sender.clone();
        let miner_sender_thread = miner_sender.clone();

        thread::spawn(move || loop {
            let transaction = upd_trans_in_rx.recv().unwrap();
            Nakamoto::stdout_notify("[TxRecv] Get Transaction".to_string());
            tx_pool_p_thread.lock().unwrap().add_tx(transaction.clone());
            trans_tx_sender_thread.send(transaction).unwrap();
            miner_sender_thread.send(true).unwrap();
        });

        let config_thread = config.clone();
        let chain_p_thread = Arc::clone(&chain_p);

        let cancellation_token_thread = Arc::clone(&cancellation_token);
        let miner_sender_thread = miner_sender.clone();

        thread::spawn(move || loop {
            let block = upd_block_in_rx.recv().unwrap();

            Nakamoto::stdout_notify("[BlockRecv] Get Block".to_string());
            Nakamoto::stdout_notify(block.header.block_id.clone());

            //Wack the block to the tree
            {
                let mut writable = cancellation_token_thread.write().unwrap();
                *writable = true;
            }

            chain_p_thread
                .lock()
                .unwrap()
                .add_block(block.clone(), config_thread.difficulty_leading_zero_len_acc);

            {
                let mut writable = cancellation_token_thread.write().unwrap();
                *writable = false;
            }

            miner_sender_thread.send(true).unwrap();
        });

        // Start necessary thread(s) to control the miner.

        let config_thread = config.clone();

        let chain_p_thread = Arc::clone(&chain_p);
        let tx_pool_p_thread = Arc::clone(&tx_pool_p);
        let miner_p_thread = Arc::clone(&miner_p);

        let cancellation_token_thread = Arc::clone(&cancellation_token);
        let miner_sender_thread = miner_sender.clone();

        thread::spawn(move || loop {
            let transaction = trans_tx_receiver.recv().unwrap();
            trans_out_tx.send(transaction.clone()).unwrap();
            miner_sender_thread.send(true).unwrap();
        });

        thread::spawn(move || loop {
            //Wait for a new transaction if transaction pool is empty
            miner_receiver.recv().unwrap();

            //loop will attempt to clear out the entire transaction pool till its empty.
            loop {
                let miner_p_thread = Arc::clone(&miner_p_thread);
                let chain_p_thread = Arc::clone(&chain_p_thread);
                let tx_pool_p_thread = Arc::clone(&tx_pool_p_thread);

                let cancellation_token_thread = Arc::clone(&cancellation_token_thread);

                {
                    let excluding_txs = chain_p_thread
                        .lock()
                        .unwrap()
                        .get_pending_finalization_txs();

                    let filtered_tx = tx_pool_p_thread
                        .lock()
                        .unwrap()
                        .filter_tx(config_thread.max_tx_in_one_block, &excluding_txs);

                    if filtered_tx.is_empty() {
                        //Capture existing transaction
                        continue;
                    }
                }

                let since_block_id = tx_pool_p_thread
                    .lock()
                    .unwrap()
                    .last_finalized_block_id
                    .clone();
                let finalized_blocks = chain_p_thread
                    .lock()
                    .unwrap()
                    .get_finalized_blocks_since(since_block_id);

                tx_pool_p_thread
                    .lock()
                    .unwrap()
                    .remove_txs_from_finalized_blocks(&finalized_blocks);

                let (puzzle_str, pre_block) = create_puzzle(
                    chain_p_thread.clone(),
                    tx_pool_p_thread.clone(),
                    config.max_tx_in_one_block.clone(),
                    config_thread.mining_reward_receiver.clone(),
                );

                //Control miner
                if pre_block.transactions_block.transactions.is_empty() {
                    //Another safety net to capture some retarded transaction
                    continue;
                }

                Nakamoto::stdout_notify("[Miner] Start solving puzzle".to_string());
                Nakamoto::stdout_notify(puzzle_str.clone());

                let solution = Miner::solve_puzzle(
                    miner_p_thread,
                    puzzle_str,
                    config_thread.nonce_len,
                    config_thread.difficulty_leading_zero_len,
                    config_thread.miner_thread_count,
                    config_thread.miner_thread_0_seed,
                    cancellation_token_thread,
                );

                match solution {
                    Some(PuzzleSolution {
                        puzzle,
                        nonce,
                        hash,
                    }) => {
                        let block_node = BlockNode {
                            header: BlockNodeHeader {
                                block_id: hash.clone(),
                                nonce: nonce.clone(),
                                ..pre_block.header
                            },
                            ..pre_block
                        };

                        Nakamoto::stdout_notify("[Miner] Puzzle solved".to_string());

                        chain_p_thread.lock().unwrap().add_block(
                            block_node.clone(),
                            config_thread.difficulty_leading_zero_len_acc,
                        );

                        block_out_tx.send(block_node).unwrap();
                    }
                    None => {
                        Nakamoto::stdout_notify("[Miner] Interrupted".to_string());
                    }
                };
            }
        });

        // Return the Nakamoto instance that holds pointers to the chain, the miner, the network and the tx pool.
        return Nakamoto {
            chain_p: chain_p,
            miner_p: miner_p,
            network_p: network_p,
            tx_pool_p: tx_pool_p,
            trans_tx: trans_tx_sender,
        };
    }

    /// Get the status of the network as a dictionary of strings. For debugging purpose.
    pub fn get_network_status(&self) -> BTreeMap<String, String> {
        self.network_p.lock().unwrap().get_status()
    }

    /// Get the status of the chain as a dictionary of strings. For debugging purpose.
    pub fn get_chain_status(&self) -> BTreeMap<String, String> {
        let chain_p_lock = self.chain_p.lock().unwrap();

        // let mut tmp_block = chain_p_lock
        //     .get_block(chain_p_lock.working_block_id.clone())
        //     .unwrap();

        // println!("\n\n");
        // loop {
        //     print!("{:?} <- ", tmp_block.header.block_id);
        //     tmp_block = chain_p_lock
        //         .get_block(tmp_block.header.parent.clone())
        //         .unwrap();

        //     if tmp_block.header.block_id == chain_p_lock.root_id {
        //         break;
        //     }
        // }
        // println!("{:?} \n\n", tmp_block.header.block_id);

        return chain_p_lock.get_status();
        // self.chain_p.lock().unwrap().get_status()
    }

    /// Get the status of the transaction pool as a dictionary of strings. For debugging purpose.
    pub fn get_txpool_status(&self) -> BTreeMap<String, String> {
        self.tx_pool_p.lock().unwrap().get_status()
    }

    /// Get the status of the miner as a dictionary of strings. For debugging purpose.
    pub fn get_miner_status(&self) -> BTreeMap<String, String> {
        self.miner_p.lock().unwrap().get_status()
    }

    /// Publish a transaction to the Blockchain
    pub fn publish_tx(&mut self, transaction: Transaction) -> () {
        // Please fill in the blank
        // Add the transaction to the transaction pool and send it to the broadcast channel
        self.tx_pool_p.lock().unwrap().add_tx(transaction.clone());
        self.trans_tx.send(transaction).unwrap();
    }

    /// Get the serialized chain as a json string.
    pub fn get_serialized_chain(&self) -> String {
        let chain = self.chain_p.lock().unwrap().clone();
        serde_json::to_string_pretty(&chain).unwrap()
    }

    /// Get the serialized transaction pool as a json string.
    pub fn get_serialized_txpool(&self) -> String {
        let tx_pool = self.tx_pool_p.lock().unwrap().clone();
        serde_json::to_string_pretty(&tx_pool).unwrap()
    }
}
