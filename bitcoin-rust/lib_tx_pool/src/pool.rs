// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file contains the definition of the transaction pool.
// The transaction pool `TxPool` is a data structure that stores all the valid transactions that are not yet finalized.
// It helps with filtering the transactions that can be included in a new block.
use std::{convert, collections::{HashMap, BTreeMap, HashSet}, hash::Hash};
use serde::{Serialize, Deserialize};
use lib_chain::block::{Signature, Transaction, TxId, BlockId, BlockNode};

/// The maximum number of transactions that can be stored in the pool. Extra transactions will be dropped.
const MAX_TX_POOL: usize = 10000;


/// A transaction pool that stores received transactions that are not yet finalized.
#[derive(Serialize, Deserialize, Debug, Clone)] 
pub struct TxPool {
    /// A list of transaction ids in the pool
    pub pool_tx_ids: Vec<TxId>,
    /// A map from transaction id (TxId) to transaction
    pub pool_tx_map: HashMap<TxId, Transaction>,
    /// A set of transaction ids that have been removed from the pool, so that duplicate transactions can be filtered out.
    pub removed_tx_ids: HashSet<TxId>,
    /// The id of the last finalized block. Transactions that are finalized will be removed from the pool and added to the removed_tx_ids set.
    pub last_finalized_block_id: BlockId
}


impl TxPool {
    /// Create a new transaction pool
    pub fn new () -> TxPool {
        TxPool { 
            pool_tx_ids: vec![], 
            pool_tx_map: HashMap::new(), 
            last_finalized_block_id: "0".to_string(),
            removed_tx_ids: HashSet::new()
        }
    }

    /// Add a transaction `tx` to the pool if it satisfies the following conditions:
    /// - The transaction is not already in the pool
    /// - The transaction is not already in the removed_tx_ids set
    /// - The pool size is less than MAX_TX_POOL
    /// - The transaction has valid signature
    /// It returns true if the transaction satisfies the conditions above and is successfully added to the pool, and false otherwise.
    pub fn add_tx(&mut self, tx: Transaction) -> bool {
        // Please fill in the blank
        todo!();
        
    }

    /// Deleting a tx from the pool. This function is used by remove_txs_from_finalized_blocks and some unit tests.
    /// It should update pool_tx_ids, pool_tx_map, and removed_tx_ids.
    /// If the transaction does not exist in the pool, make sure it is added to removed_tx_ids.
    pub fn del_tx(&mut self, tx_id: TxId) -> () {
        // Please fill in the blank
        todo!();
        
    }


    /// Filter `max_count` number of tx from the pool. It is used for creating puzzle.
    /// - `max_count`: the maximum number of transactions to be returned
    /// - `excluding_txs`: a list of transactions that should not be included in the returned list. 
    ///                    It is used to filter out those transactions on the longest chain but hasn't been finalized yet.
    pub fn filter_tx(&self, max_count: u16, excluding_txs: & Vec<Transaction>) -> Vec<Transaction> {
        // Please fill in the blank
        todo!();
        
    }

    /// Remove transactions from the pool given a list of finalized blocks. Update last_finalized_block_id as the last block in the list.
    pub fn remove_txs_from_finalized_blocks(&mut self, finalized_blocks: &Vec<BlockNode>) {
        // Please fill in the blank
        todo!();
        
    }

    /// Get status information of the tx_pool for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the tx_pool. 
        // It should be displayed in the Client UI eventually.
        todo!();
        
    }
}


