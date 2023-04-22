// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This file contains the definition of the BlockTree
/// The BlockTree is a data structure that stores all the blocks that have been mined by this node or received from other nodes.
/// The longest path in the BlockTree is the main chain. It is the chain from the root to the working_block_id.
use core::panic;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};

use base64ct::{Base64, Encoding};
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs1v15::VerifyingKey;
use rsa::signature::{Signature as RsaSignature, Verifier};

pub type UserId = String;
pub type BlockId = String;
pub type Signature = String;
pub type TxId = String;

/// Merkle tree is used to verify the integrity of transactions in a block.
/// It is generated from a list of transactions. It will be stored inside `Transactions` struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MerkleTree {
    /// A list of lists of hashes, where the first list is the list of hashes of the transactions,
    /// the second list is the list of hashes of the first list, and so on.
    /// See the `create_merkle_tree` function for more details.
    pub hashes: Vec<Vec<String>>,
}

impl MerkleTree {
    /// Create a merkle tree from a list of transactions.
    /// The merkle tree is a list of lists of hashes,
    /// where the first list is the list of hashes of the transactions.
    /// The last list is the list with only one hash, called the Merkle root.
    /// - `txs`: a list of transactions
    /// - The return value is the root hash of the merkle tree
    pub fn create_merkle_tree(txs: Vec<Transaction>) -> (String, MerkleTree) {
        if txs.len() == 0 {
            // panic!("create_merkel_tree get empty Transaction Vector.");
            return ("".to_owned(), MerkleTree { hashes: Vec::new() });
        }

        // Create the first level of hashes from the transaction vector
        let mut hashes: Vec<Vec<String>> = vec![txs.iter().map(|tx| tx.gen_hash()).collect()];

        // Keep hashing pairs of hashes together until there is only one hash remaining
        while hashes.last().unwrap().len() > 1 {
            let mut new_level: Vec<String> = vec![];

            // Get the last level of hashes
            let last_level = hashes.last().unwrap();

            // Hash pairs of hashes together to create a new level of hashes
            for chunk in last_level.chunks_exact(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk.get(0).unwrap());
                hasher.update(chunk.get(1).unwrap_or(&chunk[0]));

                new_level.push(format!("{:x}", hasher.finalize()));
            }

            // If there is an odd number of hashes, duplicate the last hash and hash it with itself to
            // create a new hash
            if last_level.len() % 2 == 1 {
                let last_hash = last_level.last().unwrap().to_owned();
                let mut hasher = Sha256::new();
                hasher.update(last_hash.as_bytes());
                hasher.update(last_hash.as_bytes());

                new_level.push(format!("{:x}", hasher.finalize()));
            }

            // Add the new level of hashes to the list of hashes
            hashes.push(new_level);
        }

        // The last hash in the last level of hashes is the root hash of the Merkle tree
        let root_hash = hashes.last().unwrap()[0].to_owned();

        // Return the root hash and the MerkleTree
        (root_hash, MerkleTree { hashes })
    }
}

/// The struct containing a list of transactions and the merkle tree of the transactions.
/// Each block will contain one `Transactions` struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transactions {
    /// The merkle tree of the transactions
    pub merkle_tree: MerkleTree,
    /// A list of transactions
    pub transactions: Vec<Transaction>,
}

/// The struct is used to store the information of one transaction.
/// The transaction id is not stored explicitly, but can be generated from the transaction using the `gen_hash` function.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    /// The user_id of the sender
    pub sender: UserId,
    /// The user_id of the receiver
    pub receiver: UserId,
    /// The message of the transaction.
    /// The expected format is `SEND $300   // By Alice   // 1678173972743`,
    /// where `300` is the amount of money to be sent,
    /// and the part after the first `//` is the comment: `Alice` is the friendly name of the sender, and `1678173972743` is the timestamp of the transaction.
    /// The comment part does not affect the validity of the transaction nor the computation of the balance.
    pub message: String,
    /// The signature of the transaction in base64 format
    pub sig: Signature,
}

impl Transaction {
    /// Create a new transaction struct given the sender, receiver, message, and signature.
    pub fn new(sender: UserId, receiver: UserId, message: String, sig: Signature) -> Transaction {
        Transaction {
            sender,
            receiver,
            message,
            sig,
        }
    }

    /// Compute the transaction id from the transaction. The transaction id is the sha256 hash of the serialized transaction struct in hex format.
    pub fn gen_hash(&self) -> TxId {
        let mut hasher = Sha256::new();
        let hasher_str = serde_json::to_string(&self).unwrap();
        hasher.update(hasher_str);
        let result = hasher.finalize();
        let tx_hash: TxId = format!("{:x}", result);
        tx_hash
    }

    /// Verify the signature of the transaction. Return true if the signature is valid, and false otherwise.
    pub fn verify_sig(&self) -> bool {
        // Please fill in the blank
        // verify the signature using the sender_id as the public key (you might need to change the format into PEM)
        // You can look at the `verify` function in `bin_wallet` for reference. They should have the same functionality.

        // Format the public key to be in PEM format
        let begin_rsa_pub_key = String::from("-----BEGIN RSA PUBLIC KEY-----\n");
        let end_rsa_pub_key = String::from("\n-----END RSA PUBLIC KEY-----\n");

        let first_half = self.sender.get(0..64).unwrap();
        let second_half = self.sender.get(64..80).unwrap();

        let sender_id_pem =
            begin_rsa_pub_key + &first_half + "\n" + &second_half + &end_rsa_pub_key;

        let public_key = rsa::RsaPublicKey::from_pkcs1_pem(&sender_id_pem).unwrap();
        let verifying_key = VerifyingKey::<Sha256>::new(public_key);

        // Craft msg
        let msg: String = String::from("[\"")
            + &self.sender
            + "\",\""
            + &self.receiver
            + "\",\""
            + &self.message
            + "\"]";

        // Obtain signature
        let signature = Base64::decode_vec(&self.sig).unwrap();
        let verify_signature = RsaSignature::from_bytes(&signature).unwrap();

        // Verify the message using public key, signature, and msg
        let verify_result = verifying_key.verify(&msg.as_bytes(), &verify_signature);

        return match verify_result {
            Ok(()) => true,
            Err(e) => {
                eprintln!("[Chain] Signature verification failed: {}", e);
                false
            }
        };
    }
}

/// The struct representing a whole block tree.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockTree {
    /// A map from block id to the block node
    pub all_blocks: HashMap<BlockId, BlockNode>, //X
    /// A map from block id to the list of its children (as block ids)
    pub children_map: HashMap<BlockId, Vec<BlockId>>, //X
    /// A map from block id to the depth of the block. The genesis block has depth 0.
    pub block_depth: HashMap<BlockId, u64>, //X
    /// The id of the root block (the genesis block)
    pub root_id: BlockId, //-X
    /// The id of the working block (the block at the end of the longest chain)
    pub working_block_id: BlockId,
    /// A map to bookkeep the orphan blocks.
    /// Orphan blocks are blocks whose parent are not in the block tree yet.
    /// They should be added to the block tree once they can be connected to the block tree.
    pub orphans: HashMap<BlockId, BlockNode>, //TODOL REQUIRED TO RECURSIVELY CHECK FOR ORPHAN NODE ON NEW BLOCK ADD
    /// The id of the latest finalized block
    pub finalized_block_id: BlockId, //X
    /// A map from the user id to its balance
    pub finalized_balance_map: HashMap<UserId, i64>, //X
    /// A set of transaction ids that have been finalized. It includes all the transaction ids in the finalized blocks.
    pub finalized_tx_ids: HashSet<TxId>, //X
}

impl BlockTree {
    /// Create a new block tree with the genesis block as the root.
    pub fn new() -> BlockTree {
        let mut bt = BlockTree {
            all_blocks: HashMap::new(),
            children_map: HashMap::new(),
            block_depth: HashMap::new(),
            root_id: String::new(),
            working_block_id: String::new(),
            orphans: HashMap::new(),
            finalized_block_id: String::new(),
            finalized_balance_map: HashMap::new(),
            finalized_tx_ids: HashSet::new(),
        };
        let genesis_block = BlockNode::genesis_block();
        bt.all_blocks.insert("0".to_string(), genesis_block.clone());
        bt.block_depth.insert("0".to_string(), 0);
        bt.root_id = "0".to_string();
        bt.working_block_id = "0".to_string();
        for tx in genesis_block.transactions_block.transactions {
            let amount = tx.message.split(" ").collect::<Vec<&str>>()[1]
                .trim_start_matches('$')
                .parse::<i64>()
                .unwrap();
            bt.finalized_balance_map.insert(tx.receiver, amount);
        }
        bt.finalized_block_id = "0".to_string();
        bt
    }

    pub fn stdout_notify(msg: String) {
        let msg = HashMap::from([("Notify".to_string(), msg.clone())]);
        println!("{}", serde_json::to_string(&msg).unwrap());
    }

    /// Add a block to the block tree. If the block is not valid to be added to the tree
    /// (i.e. it does not satsify the conditions below), ignore the block. Otherwise, add the block to the BlockTree.
    ///
    /// 1. The block must have a valid nonce and the hash in the puzzle solution satisfies the difficulty requirement. X
    /// 2. The block_id of the block must be equal to the computed hash in the puzzle solution. X
    /// 3. The block does not exist in the block tree or the orphan map. X
    /// 4. The transactions in the block must be valid. See the `verify_sig` function in the `Transaction` struct for details. X
    /// 5. The parent of the block must exist in the block tree. X
    ///     Otherwise, it will be bookkeeped in the orphans map. X
    ///     TODO: When the parent block is added to the block tree, the block will be removed from the orphan map and checked against the conditions again.
    /// 6. The transactions in the block must not be duplicated with any transactions in its ancestor blocks. X
    /// 7. Each sender in the txs in the block must have enough balance to pay for the transaction.
    ///    Conceptually, the balance of one address is the sum of the money sent to the address minus the money sent from the address
    ///    when walking from the genesis block to this block, according to the order of the txs in the blocks.
    ///    Mining reward is a constant of $10 (added to the reward_receiver address **AFTER** considering transactions in the block).
    ///
    /// When a block is successfully added to the block tree, update the related fields in the BlockTree struct
    /// (e.g., working_block_id X, finalized_block_id, finalized_balance_map, finalized_tx_ids, block_depth X, children_map X, all_blocks X, etc)
    pub fn add_block(&mut self, block: BlockNode, leading_zero_len: u16) -> () {
        // Please fill in the blank

        //Verify if block already exist in block_tree or orphan_map
        let block_exist = self.all_blocks.contains_key(&block.header.block_id)
            || self.orphans.contains_key(&block.header.block_id);

        // Verify if block meet the condition stated.
        if block_exist {
            eprintln!("[Chain] Block already exist");
            return;
        }

        if !self.verify_block_integrity(block.clone(), leading_zero_len) {
            eprintln!("[Chain] Invalid block");
            return;
        }

        // Verify if parent block exist
        let parent_block = match self.get_block(block.header.parent.clone()) {
            Some(block) => block,
            None => {
                BlockTree::stdout_notify(
                    "[Chain] Parent node not found. Storing block as orphan...".to_string(),
                );
                self.orphans.insert(block.header.block_id.clone(), block);
                return;
            }
        };

        // Verify that there is no duplicated transaction(s) in any ancestor blocks
        if !self.verify_block_ancestry(block.header.merkle_root.clone(), parent_block.clone()) {
            eprintln!("[Chain] Found duplicate transactions in ancestor block");
            return;
        }

        //Verify if transactions in the block are valid
        if !self.verifiy_transactions_balance(block.clone(), parent_block.clone()) {
            eprintln!("[Chain] Transaction does not have sufficient balance");
            return;
        }

        self.update_blocktree(block.clone());

        for (_, o_block) in self.orphans.clone() {
            if o_block.header.parent == block.header.block_id {
                BlockTree::stdout_notify(
                    "[Chain] Orphan block found. Adding orphan to chain...".to_string(),
                );
                self.add_orphan_block(o_block, leading_zero_len);
                self.orphans.remove(&block.header.block_id.clone());
                return;
            }
        }
    }

    /// Get the block node by the block id if exists. Otherwise, return None.
    pub fn get_block(&self, block_id: BlockId) -> Option<BlockNode> {
        // Please fill in the blank
        return self.all_blocks.get(&block_id).cloned();
    }

    /// Get the finalized blocks on the longest path after the given block id, from the oldest to the most recent.
    /// The given block id should be any of the ancestors of the current finalized block id or the current finalized block id itself.
    /// If it is not the case, the function will panic (i.e. we do not consider inconsistent block tree caused by attacks in this project)
    pub fn get_finalized_blocks_since(&self, since_block_id: BlockId) -> Vec<BlockNode> {
        // Please fill in the blank
        let mut f_blocks: Vec<BlockNode> = Vec::new();
        let mut block = self
            .get_block(self.finalized_block_id.clone())
            .unwrap()
            .clone();
        if self.block_depth.get(&since_block_id) > self.block_depth.get(&self.finalized_block_id) {
            panic!("Given block id is not finalized");
        }

        if block.header.block_id == since_block_id {
            return f_blocks;
        }

        loop {
            f_blocks.insert(0, block.clone());

            if block.header.parent == since_block_id || block.header.parent == self.root_id {
                break;
            }

            block = self.get_block(block.header.parent.clone()).unwrap().clone();
        }

        return f_blocks;
    }

    // pub fn get_finalized_blocks_since(&self, since_block_id: BlockId) -> Vec<BlockNode> {
    //     // Please fill in the blank
    //     let mut f_blocks: Vec<BlockNode> = Vec::new();
    //     let mut block = self
    //         .get_block(self.finalized_block_id.clone())
    //         .unwrap()
    //         .clone();

    //     while block.header.block_id != since_block_id && block.header.parent != self.root_id {
    //         f_blocks.insert(0, block.clone());
    //         block = self.get_block(block.header.parent.clone()).unwrap().clone();
    //     }
    //     block = self.get_block(self.root_id.clone()).unwrap().clone();
    //     f_blocks.insert(0, block.clone());

    //     return f_blocks;
    // }

    /// Get the pending transactions on the longest chain that are confirmed but not finalized.
    pub fn get_pending_finalization_txs(&self) -> Vec<Transaction> {
        let mut block = self.get_block(self.working_block_id.clone()).unwrap();
        let mut pending_txs: Vec<Transaction> = Vec::new();

        while block.header.block_id != self.finalized_block_id {
            pending_txs.extend(block.transactions_block.transactions);
            block = self.get_block(block.header.parent.clone()).unwrap();
        }

        return pending_txs;
    }

    /// Get status information of the BlockTree for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the BlockTree.
        // It should be displayed in the Client UI eventually.
        let mut map = BTreeMap::new();
        map.insert("#blocks".to_string(), self.all_blocks.len().to_string());
        map.insert("#orphans".to_string(), self.orphans.len().to_string());
        map.insert(
            "finalized_id".to_string(),
            self.finalized_block_id.to_string(),
        );
        map.insert("root_id".to_string(), self.root_id.to_string());

        let working_block_id = self.working_block_id.clone();
        let block_depth = *self.block_depth.get(&working_block_id).unwrap();

        map.insert("working_depth".to_string(), block_depth.to_string());
        map.insert("working_id".to_string(), working_block_id.to_string());
        return map;
    }

    pub fn verify_block_integrity(&self, block: BlockNode, leading_zero_len: u16) -> bool {
        let header = block.header.clone();
        let transactions = block.transactions_block.transactions.clone();

        let puzzle = Puzzle {
            parent: header.parent,
            merkle_root: header.merkle_root,
            reward_receiver: header.reward_receiver,
        };

        let puzzle_serialized = serde_json::to_string(&puzzle).unwrap();
        let challenge = header.nonce.to_string() + &puzzle_serialized;

        let computed_block_id = format!("{:x}", Sha256::digest(challenge.as_bytes()));

        let valid_block_id = header.block_id == computed_block_id;
        let valid_leading_zero_len = header
            .block_id
            .starts_with(&"0".repeat(leading_zero_len as usize));

        let valid_transactions = transactions.iter().all(|t| t.verify_sig());

        let is_valid = valid_block_id && valid_leading_zero_len && valid_transactions;

        return is_valid;
    }

    pub fn verify_block_ancestry(
        &self,
        block_merkle_root: String,
        mut parent_block: BlockNode,
    ) -> bool {
        while parent_block.header.parent != self.root_id {
            if block_merkle_root == parent_block.header.merkle_root {
                return false;
            }

            parent_block = self
                .get_block(parent_block.header.parent.clone())
                .unwrap()
                .clone();
        }

        return true; //No duplicate transaction found
    }

    pub fn verifiy_transactions_balance(
        &self,
        block: BlockNode,
        mut parent_block: BlockNode,
    ) -> bool {
        let mut balance_map = self.finalized_balance_map.clone();

        for tx in self.get_pending_finalization_txs().iter() {
            let receiver_id = tx.receiver.clone();
            let sender_id = tx.sender.clone();

            let amount = tx.message.split(" ").collect::<Vec<&str>>()[1]
                .trim_start_matches('$')
                .parse::<i64>()
                .unwrap();

            *balance_map.entry(sender_id.clone()).or_insert(0) -= amount;
            *balance_map.entry(receiver_id.clone()).or_insert(0) += amount;
        }

        while parent_block.header.parent != self.finalized_block_id {
            *balance_map
                .entry(parent_block.header.reward_receiver)
                .or_insert(0) += 10;

            parent_block = self
                .get_block(parent_block.header.parent.clone())
                .unwrap()
                .clone();
        }

        //Verify if current block transaction is valid or not
        for tx in block.transactions_block.transactions.iter() {
            let receiver_id = tx.receiver.clone();
            let sender_id = tx.sender.clone();

            let amount = tx.message.split(" ").collect::<Vec<&str>>()[1]
                .trim_start_matches('$')
                .parse::<i64>()
                .unwrap();

            let sender_balance = balance_map.get(&sender_id).unwrap_or(&0);
            if sender_balance < &amount {
                return false;
            }

            *balance_map.entry(sender_id.clone()).or_insert(0) -= amount;
            *balance_map.entry(receiver_id.clone()).or_insert(0) += amount;
        }

        *balance_map
            .entry(parent_block.header.reward_receiver)
            .or_insert(0) += 10;

        return true;
    }

    pub fn finalize_block(&mut self, mut block: BlockNode) -> () {
        let f_block_children = self.children_map.get(&self.finalized_block_id).unwrap();

        //k-deep confirmation rule = 6
        for _ in 0..6 {
            if block.header.parent == self.finalized_block_id {
                BlockTree::stdout_notify("[Chain] Insufficient block to finalize".to_string());
                return;
            }

            if let Some(parent_block) = self.get_block(block.header.parent) {
                block = parent_block.clone();
            } else {
                return;
            }
        }

        let new_f_block = block;
        let mut new_f_block_balance_map = self.finalized_balance_map.clone();
        let mut new_f_block_tx_ids: HashSet<String> = HashSet::new();

        if f_block_children.contains(&new_f_block.header.block_id) {
            for tx in new_f_block.transactions_block.transactions.iter() {
                let receiver_id = tx.receiver.clone();
                let sender_id = tx.sender.clone();

                let amount = tx.message.split(" ").collect::<Vec<&str>>()[1]
                    .trim_start_matches('$')
                    .parse::<i64>()
                    .unwrap();

                *new_f_block_balance_map
                    .entry(sender_id.clone())
                    .or_insert(0) -= amount;
                *new_f_block_balance_map
                    .entry(receiver_id.clone())
                    .or_insert(0) += amount;

                new_f_block_tx_ids.insert(tx.gen_hash());
            }

            *new_f_block_balance_map
                .entry(new_f_block.header.reward_receiver)
                .or_insert(0) += 10;

            self.finalized_tx_ids.extend(new_f_block_tx_ids.clone());
            self.finalized_balance_map = new_f_block_balance_map.clone();
            self.finalized_block_id = new_f_block.header.block_id.clone();
        }
    }

    pub fn update_blocktree(&mut self, block: BlockNode) -> () {
        self.all_blocks
            .insert(block.header.block_id.clone(), block.clone());

        let parent_block_depth = *self.block_depth.get(&block.header.parent).unwrap();
        let block_depth = parent_block_depth + 1;
        self.block_depth
            .insert(block.header.block_id.clone(), block_depth.clone());

        self.children_map
            .entry(block.header.parent.clone())
            .or_insert_with(Vec::new)
            .push(block.header.block_id.clone());

        let all_children = self.children_map.get(&block.header.parent).unwrap();

        let largest_child = all_children
            .iter()
            .max_by(|&a, &b| a.cmp(&b))
            .unwrap_or(&block.header.block_id);

        self.working_block_id = largest_child.clone();

        self.finalize_block(block.clone());
    }

    //Similar to add_block() just do not check for blocks exist in orphan_map
    pub fn add_orphan_block(&mut self, block: BlockNode, leading_zero_len: u16) -> () {
        // Please fill in the blank

        //Check if block exist
        let block_exist = self.all_blocks.contains_key(&block.header.block_id);

        // //Check if block meet the condition stated.
        if !self.verify_block_integrity(block.clone(), leading_zero_len) || block_exist {
            eprintln!("[Chain] Invalid block or block already exist in chain");

            return;
        }

        // //Check if parent block exist
        // Ensure that there is no duplicated transactions in ancestor blocks
        let parent_block = match self.get_block(block.header.parent.clone()) {
            Some(block) => block,
            None => {
                BlockTree::stdout_notify(
                    "[Chain] Parent node not found. Storing block as orphan...".to_string(),
                );
                self.orphans.insert(block.header.block_id.clone(), block);
                return;
            }
        };

        if !self.verify_block_ancestry(block.header.merkle_root.clone(), parent_block.clone()) {
            eprintln!("[Chain] Found duplicate transactions in ancestor block");
            return;
        }

        if !self.verifiy_transactions_balance(block.clone(), parent_block.clone()) {
            eprintln!("[Chain] Transaction does not have sufficient balance");
            return;
        }

        self.update_blocktree(block.clone());

        //Recursively calls to add all possible orphan blocks to the chain
        for (_, o_block) in self.orphans.clone() {
            if o_block.header.parent == block.header.block_id {
                BlockTree::stdout_notify(
                    "[Chain] Orphan block found. Adding orphan to chain...".to_string(),
                );

                self.add_orphan_block(o_block, leading_zero_len);
                self.orphans.remove(&block.header.block_id.clone());

                return;
            }
        }
    }
}
/// The struct representing a puzzle for the miner to solve. The puzzle is to find a nonce such that when concatenated
/// with the serialized json string of this `Puzzle` struct, the sha256 hash of the result has the required leading zero length.
#[derive(Serialize)]
pub struct Puzzle {
    pub parent: BlockId,
    pub merkle_root: String,
    pub reward_receiver: UserId,
}

/// The struct representing a block header. Each `BlockNode` has one `BlockNodeHeader`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockNodeHeader {
    /// The block id of the parent block.
    pub parent: BlockId,
    /// The merkle root of the transactions in the block.
    pub merkle_root: String,
    /// The timestamp of the block. For genesis block, it is 0. For other blocks, greater or equal to 1 is considered valid.
    pub timestamp: u64,
    /// The block id of the block (the block id is the sha256 hash of the concatination of the nonce and a `Puzzle` derived from the block)
    pub block_id: BlockId,
    /// The nonce is the solution found by the miner for the `Puzzle` derived from this block.
    pub nonce: String,
    /// The reward receiver of the block.
    pub reward_receiver: UserId,
}

/// The struct representing a block node.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockNode {
    /// The header of the block.
    pub header: BlockNodeHeader,
    /// The transactions in the block.
    pub transactions_block: Transactions,
}

impl BlockNode {
    /// Create the genesis block that contains the initial transactions
    /// (give $299792458 to the address of Alice `MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==`)
    pub fn genesis_block() -> BlockNode {
        let header = BlockNodeHeader {
            parent: "0".to_string(),
            merkle_root: "0".to_string(),
            timestamp: 0,
            block_id: "0".to_string(),
            nonce: "0".to_string(),
            reward_receiver: "GENESIS".to_string(),
        };

        let transactions_block = Transactions {
            transactions: vec![Transaction::new(
                "GENESIS".to_owned(),
                "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ=="
                    .to_string(),
                "SEND $299792458".to_owned(),
                "GENESIS".to_owned(),
            )],
            merkle_tree: MerkleTree { hashes: vec![] }, // Skip merkle tree generation for genesis block
        };

        BlockNode {
            header,
            transactions_block,
        }
    }

    /// Check for block validity based solely on this block (not considering its validity inside a block tree).
    /// Return a tuple of (bool, String) where the bool is true if the block is valid and false otherwise.
    /// The string is the re-computed block id.
    /// The following need to be checked:
    /// 1. The block_id in the block header is indeed the sha256 hash of the concatenation of the nonce and the serialized json string of the `Puzzle` struct derived from the block.
    /// 2. All the transactions in the block are valid.
    /// 3. The merkle root in the block header is indeed the merkle root of the transactions in the block.
    pub fn validate_block(&self, leading_zero_len: u16) -> (bool, BlockId) {
        let header = self.header.clone();
        let transactions = self.transactions_block.transactions.clone();

        let puzzle = Puzzle {
            parent: header.parent,
            merkle_root: header.merkle_root,
            reward_receiver: header.reward_receiver,
        };

        let puzzle_serialized = serde_json::to_string(&puzzle).unwrap();
        let challenge = header.nonce.to_string() + &puzzle_serialized;

        let computed_block_id = format!("{:x}", Sha256::digest(challenge.as_bytes()));

        let valid_block_id = header.block_id == computed_block_id;
        let valid_leading_zero_len = header
            .block_id
            .starts_with(&"0".repeat(leading_zero_len as usize));

        let valid_transactions = transactions.iter().all(|t| t.verify_sig());
        let valid_merkle_root =
            MerkleTree::create_merkle_tree(transactions).0 == self.header.merkle_root;

        let is_valid =
            valid_block_id && valid_leading_zero_len && valid_transactions && valid_merkle_root;

        (is_valid, computed_block_id)
    }
}
