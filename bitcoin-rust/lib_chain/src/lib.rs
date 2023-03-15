// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

pub mod block;

#[cfg(test)]
mod tests {
    use std::fs;
    use serde::{Serialize, de::DeserializeOwned};
    use crate::block::{BlockTree, BlockNode, Transaction, MerkleTree};

    fn serialize_clone<T: Serialize + DeserializeOwned>(obj: &T) -> T {
        let jsonstr: String = serde_json::to_string(&obj).unwrap();
        serde_json::from_str(&jsonstr).unwrap()
    }

    fn read_string_from_file(filepath: &str) -> String {
        let contents = fs::read_to_string(filepath)
            .expect(&("Cannot read ".to_owned() + filepath));
        contents
    }

    fn write_string_to_file(filepath: &str, content: String) {
        fs::write(filepath, content).expect(&("Cannot write ".to_owned() + filepath));
    }

    /// Test the signature verification on `Transaction`.
    #[test]
    fn test_transaction_signature() {
        let mut tx = Transaction {
            sender: "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==".to_string(),
            receiver: "MDgCMQDOpK8YWmcg8ffNF/O7xlBDq/DBdoUnc4yyWrV0y/X3LF+dddjaGksXzGl3tHskpgkCAwEAAQ==".to_string(),
            message: "SEND $300   // By Alice   // 1678250102871".to_string(),
            sig: "l8gsKxmAUzhgqbVqGlXaO69+Qhr87QthvZjUbYZXvnb+tanxCi8wm3c5UjHZ+HKm".to_string()
        };
        assert!(tx.verify_sig() == true);
        tx.sig = "A8gsKxmAUzhgqbVqGlXaO69+Qhr87QthvZjUbYZXvnb+tanxCi8wm3c5UjHZ+HKm".to_string();
        assert!(tx.verify_sig() == false);
    }

    /// Test the generation of merkle tree.
    #[test]
    fn test_merkle_tree() {
        let tx1 = Transaction {
            sender: "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==".to_string(),
            receiver: "MDgCMQDZDExOs97sRTnQLYtgFjDKpDzmO7Uo5HPP62u6MDimXBpZtGxtwa8dhJe5NBIsJjUCAwEAAQ==".to_string(),
            message: "SEND $100   // By Alice   // 1678198053097".to_string(),
            sig: "LJxQJi3pzVlM/7U/y5BV6kbJ9A3kXAyw2yLmBO3tG0gaEenwjRUbU9FGL7folRYA".to_string()
        };
        let tx2 = Transaction { 
            sender: "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==".to_string(),
            receiver: "MDgCMQDeoEeA8OtGME/SRwp+ASKVOnjlEUHYvQfo0FLp3+fwVi/SztDdJskjzCRasGk06UUCAwEAAQ==".to_string(),
            message: "SEND $200   // By Alice   // 1678198045087".to_string(),
            sig: "SOuwjm0I1vwt3LE2dVWuaFJAIYrswewl1/B1eiyuvgyFU4pWeqP4pIcuHgC3JAPh".to_string()
        };
        let tx3 = Transaction { 
            sender: "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==".to_string(),
            receiver: "MDgCMQDeoEeA8OtGME/SRwp+ASKVOnjlEUHYvQfo0FLp3+fwVi/SztDdJskjzCRasGk06UUCAwEAAQ==".to_string(),
            message: "SEND $200   // By Alice   // 1678198048091".to_string(),
            sig: "Lbmm5uxAvg8HBlC/wAgpna8iNwaHk+Yw74eKR0F7vonOPiS63YUxR/n07SjNpTUH".to_string()
        };
        let tx4 = Transaction { 
            sender: "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==".to_string(),
            receiver: "MDgCMQDZDExOs97sRTnQLYtgFjDKpDzmO7Uo5HPP62u6MDimXBpZtGxtwa8dhJe5NBIsJjUCAwEAAQ==".to_string(),
            message: "SEND $100   // By Alice   // 1678198050093".to_string(),
            sig: "EQWylQX/AIcQDStCGB6ujRmqDalO5z52VguJO9f5S0C1VPhGFh58r8Mi0Wo+ue8a".to_string()
        };


        let tx_vec = vec![tx1, tx2, tx3, tx4];
        let (merkle_root, merkle) = MerkleTree::create_merkle_tree(tx_vec);

        for level in merkle.hashes {
            println!("{}\n", level.join(", "))
        }
        assert!(merkle_root == "3c1ad8fb380a0808e5c0c0a864040d74338de2aa3a9f5aa2657371b4f5a68ae6");
        // Expected output:
        // 0cb819e19d34ee88b6c20c78cea0aec818f1d6875479c9c9a1505c27495d7a91, 9977c7f05151d3b9bae366fd55facd78ef3054508c1950be7821322e4a7f11fe, b2f3877e2827e9761b902b0eb092a787fe662e58f65eb8978f14e59490a368ff, 2764749767b38c8f947005179a3b5aba56de75261c9b42bdf45799baca0738b4
        // d9961bbcbbcedfa994a951662da7285ccc82940a6e65028da233e21b1c543a13, aeec55bd3c6bcc5749cd248fd88b65de45fd14b42abdad4b3c874c2e00865a1b
        // 3c1ad8fb380a0808e5c0c0a864040d74338de2aa3a9f5aa2657371b4f5a68ae6
    }

    /// Test basic block tree creation
    #[test]
    fn block_test_basic() {
        let block_json = read_string_from_file("./testdata/add_blocks_basic__2.json");
        let mut block_node = serde_json::from_str::<BlockNode>(&block_json).unwrap();
        
        // validation test
        assert!(block_node.validate_block(5) == (true, block_node.header.block_id.clone()));
        assert!(block_node.validate_block(8) == (false, block_node.header.block_id.clone()));
        let correct_id = block_node.header.block_id.clone();
        block_node.header.block_id = "000006d08aa94e7acbe657fc385a2260823a528702b4de57452dfda0587dc8e0".to_string();
        assert!(block_node.validate_block(5) == (false, correct_id.clone()));
        block_node.header.block_id = correct_id;

        // block add orphan test
        let mut default_block_tree = BlockTree::new();
        default_block_tree.add_block(block_node.clone(), 5);
        assert!(default_block_tree.working_block_id == "0".to_string());
        assert!(default_block_tree.root_id == "0".to_string());
        assert!(default_block_tree.orphans.len() == 1);
    }

    /// Test adding blocks to the blocktree (orphan not considered)
    #[test]
    fn blocktree_add_blocks_basic() {
        let mut default_btree = BlockTree::new();
        // print current pwd
        println!("current dir: {:?}", std::env::current_dir());
        for i in vec![1,2,3,4,5,6,7,8] {
            // read block from "./testdata/add_block_basic__{i}.json"
            let block_json = read_string_from_file(&format!("./testdata/add_blocks_basic__{}.json", i));
            let block_node = serde_json::from_str::<BlockNode>(&block_json).unwrap();
            default_btree.add_block(block_node, 5);
        }
        assert!(default_btree.working_block_id == "0000052b06a4d5c725f3713aed93d4b4e1da93a7b4f7cb870ef1f7e6b6b0fcb8".to_string());
        assert!(default_btree.finalized_balance_map[&"MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==".to_owned()] == 299791558);
        assert!(default_btree.finalized_balance_map[&"MDgCMQDZDExOs97sRTnQLYtgFjDKpDzmO7Uo5HPP62u6MDimXBpZtGxtwa8dhJe5NBIsJjUCAwEAAQ==".to_owned()] == 300);
        assert!(default_btree.finalized_balance_map[&"MDgCMQDeoEeA8OtGME/SRwp+ASKVOnjlEUHYvQfo0FLp3+fwVi/SztDdJskjzCRasGk06UUCAwEAAQ==".to_owned()] == 20);
        assert!(default_btree.block_depth[&"00000e3737f396b050fd38ed30e8813818229ffa43ce5f77b3781ace835a8db6".to_owned()] == 7);
        assert!(default_btree.finalized_block_id == "00000f93bcb625d8181e02c5e952672b3b178ab6cb56c86546b605e8915a1b11");
        //println!("default_btree: {:?}", default_btree);
    }

    /// Test adding blocks to the blocktree (orphan considered)
    #[test]
    fn blocktree_add_blocks_orphan() {
        let mut default_btree = BlockTree::new();
        // print current pwd
        println!("current dir: {:?}", std::env::current_dir());
        for i in vec![3,6,1,4,2,8,5,7] {
            // read block from "./testdata/add_block_basic__{i}.json"
            let block_json = read_string_from_file(&format!("./testdata/add_blocks_basic__{}.json", i));
            let block_node = serde_json::from_str::<BlockNode>(&block_json).unwrap();
            default_btree.add_block(block_node, 5);
        }
        assert!(default_btree.working_block_id == "0000052b06a4d5c725f3713aed93d4b4e1da93a7b4f7cb870ef1f7e6b6b0fcb8".to_string());
        assert!(default_btree.finalized_balance_map[&"MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==".to_owned()] == 299791558);
        assert!(default_btree.finalized_balance_map[&"MDgCMQDZDExOs97sRTnQLYtgFjDKpDzmO7Uo5HPP62u6MDimXBpZtGxtwa8dhJe5NBIsJjUCAwEAAQ==".to_owned()] == 300);
        assert!(default_btree.finalized_balance_map[&"MDgCMQDeoEeA8OtGME/SRwp+ASKVOnjlEUHYvQfo0FLp3+fwVi/SztDdJskjzCRasGk06UUCAwEAAQ==".to_owned()] == 20);
        assert!(default_btree.block_depth[&"00000e3737f396b050fd38ed30e8813818229ffa43ce5f77b3781ace835a8db6".to_owned()] == 7);
        assert!(default_btree.finalized_block_id == "00000f93bcb625d8181e02c5e952672b3b178ab6cb56c86546b605e8915a1b11");
        //println!("default_btree: {:?}", default_btree);
    }


    /// Your own test that tests your blocktree implementation more throughly (e.g., orphan, invalid block, etc.)
    #[test]
    fn blocktree_additional_test() {
        // Please fill in the blank
        // You can add your own json files and read them for testing
        
    }

}

