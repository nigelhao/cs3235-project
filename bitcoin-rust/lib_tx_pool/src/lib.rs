// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

pub mod pool;



#[cfg(test)]
mod tests {
    use std::fs;
    use std::fmt::Debug;
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize, de::DeserializeOwned};
    use lib_chain::block::{BlockTree, BlockNode, Transaction, BlockNodeHeader, Transactions, MerkleTree};

    use crate::pool::TxPool;

    fn read_string_from_file(filepath: &str) -> String {
        let contents = fs::read_to_string(filepath)
            .expect(&("Cannot read ".to_owned() + filepath));
        contents
    }
    
    /// Test the basic operations of TxPool
    #[test]
    fn test_transaction_ops () {
        let txs_str = read_string_from_file("./testdata/txs_0.json");
        let txs = serde_json::from_str::<Vec<Transaction>>(&txs_str).unwrap();

        let mut tx_pool = TxPool::new();
        for v in &txs {
            tx_pool.add_tx(v.clone());
        }

        assert!(tx_pool.pool_tx_ids.len() == 14);
        
        tx_pool.del_tx(txs[0].gen_hash());
        tx_pool.del_tx(txs[1].gen_hash());
        tx_pool.del_tx(txs[2].gen_hash());
        tx_pool.del_tx(txs[3].gen_hash());
        tx_pool.del_tx(txs[4].gen_hash());

        // clone txs and take the slice from 5 to 10
        let txs_5_10 = txs[5..10].to_vec();
        let filtered_tx = tx_pool.filter_tx(5, &txs_5_10);
        println!("filtered_tx: {:?}", filtered_tx);
        assert!(filtered_tx.len() == 4);
        assert!(filtered_tx.iter().any(|tx| tx.message == "SEND $300   // By Alice   // 1678173978750"));
        assert!(filtered_tx.iter().any(|tx| tx.message == "SEND $100   // By Alice   // 1678173979751"));
        assert!(filtered_tx.iter().any(|tx| tx.message == "SEND $200   // By Alice   // 1678173980752"));
        assert!(filtered_tx.iter().any(|tx| tx.message == "SEND $100   // By Alice   // 1678173982754"));

    }

    /// Your own additional test that tests your implementation more throughly 
    /// (e.g. invalid signature, and test methods that are not covered in the tests above)
    #[test]
    fn test_tx_pool_additional () {
        // Please fill in the blank
        
    }
}

