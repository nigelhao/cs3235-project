# Chain

## Functionality

1. Define the transaction, block, blocktree structure.
   
2. Check the validity of one block, i.e., the leading bits of its hash are equal to 0.
Use SHA256 hash function like Bitcoin. The node structure is like:
```
{
  "header": {
    "parent": "", // the hash of previous block header 
    "merkle_root": "", // the root of the merkle tree of trasactions
    "timestamp": u64,
    "nonce": ""
  },
  "transactions": [
    "merkle_tree": MerkleTree,
    "transactions: Vec<Transaction>
  ]
}
```
Here we use single SHA-256 for computing the merkle tree instead of double SHA-256 in Bitcoin.

3. Add one block to the blocktree if valid and update the blocktree (e.g., working block, block depth, chain) accordingly.
