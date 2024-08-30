//  The code is a simple implementation of a sparse Merkle tree client that supports two types of sparse nodes: a counter sparse node and a hash chain sparse node. The code is organized as follows: 
//  The  MerkleTreeDigest  struct represents the digest of a Merkle tree. 
//  The  Leaf  struct represents a Merkle tree leaf. 
//  The  StreamID  struct represents a stream identifier. 
//  The  MyDigest  struct represents a hash digest
//  The  Point  struct represents either an effects digest or an event digest.
//  The  StreamUpdate  type represents a stream update.
//  The  StreamUpdater  trait defines the  update  method that updates the sparse node with the given stream updates and returns the digest of the updated Merkle tree.

use std::collections::HashMap;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct MerkleTreeDigest(Vec<u8>); // Placeholder for Merkle tree digest

#[derive(Debug, Clone)]
pub struct Leaf(Vec<u8>); // A Merkle tree leaf


#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct StreamID(u32);

/// The length of the digests used in the merkle tree.
pub const DIGEST_LEN: usize = 32;

#[derive(Debug, PartialEq)]
pub struct MyDigest([u8; DIGEST_LEN]); // A hash digest
const EMPTY_NODE: [u8; DIGEST_LEN] = [0; DIGEST_LEN];

// TODO: Enhance it to support transactions for sparse nodes that re-executes the updates
pub struct Point([u8; DIGEST_LEN]); // Either an effects digest or a event digest
const EMPTY_POINT: Point = Point(EMPTY_NODE);

pub type StreamUpdate = (StreamID, Vec<Point>); // (stream_id, [point_1, point_2, ..., point_n])

pub trait StreamUpdater {
    fn update(&mut self, updates: Vec<StreamUpdate>) -> MerkleTreeDigest;
}

pub fn compute_merkle_tree(digests: Vec<Leaf>) -> MerkleTreeDigest {
    let mut hasher = Sha256::new();
    for digest in digests {
        hasher.update(&digest.0);
    }
    MerkleTreeDigest(hasher.finalize().to_vec())
}

pub struct CounterSparseNode {
    // TODO: Change HashMap to a DB
    pub counts: HashMap<StreamID, u32>,
}

impl CounterSparseNode {
    pub fn new() -> Self {
        Self { counts: HashMap::new() }
    }
}

impl StreamUpdater for CounterSparseNode {
    // Computes H([id, local_count, global_count])
    fn update(&mut self, updates: Vec<StreamUpdate>) -> MerkleTreeDigest {
        let mut leafs = Vec::new();
        for (stream_id, points) in updates {
            let sum: u32 = points.len().try_into().unwrap();
            let counter = self.counts.entry(stream_id).or_insert(0);
            *counter += sum;

            // Only hash the updated entries
            let mut hasher = Sha256::new();
            hasher.update(&stream_id.0.to_be_bytes());
            hasher.update(&sum.to_be_bytes());
            hasher.update(&counter.to_be_bytes());
            let digest = hasher.finalize();
            leafs.push(Leaf(digest.to_vec()));
        }

        // Finalize the digest with only the updated entries
        compute_merkle_tree(leafs)
    }
}

pub struct HashChainSparseNode {
    pub heads: HashMap<StreamID, MyDigest >
}

impl HashChainSparseNode {
    pub fn new() -> Self {
        Self { heads: HashMap::new() }
    }
}

impl StreamUpdater for HashChainSparseNode {
    fn update(&mut self, updates: Vec<StreamUpdate>) -> MerkleTreeDigest {
        let mut leafs: Vec<Leaf> = Vec::new();
        for (stream_id, points) in updates {
            let head = self.heads.entry(stream_id).or_insert(MyDigest([0; 32]));
            for point in points {
                let mut hasher = Sha256::new();
                hasher.update(&head.0);
                hasher.update(&point.0);
                let digest = hasher.finalize();
                *head = MyDigest(digest.into());
            }
            leafs.push(Leaf(head.0.to_vec()));
        }
        return compute_merkle_tree(leafs)
    }
}

fn main() {
    let updates = vec![
        (StreamID(0), vec![EMPTY_POINT, EMPTY_POINT]),
        (StreamID(1), vec![EMPTY_POINT]),
    ];

    let mut counters = CounterSparseNode::new();
    let digest1 = counters.update(updates);
    println!("Counters Digest: {:?}", digest1);

    // Print the final counters for debugging
    println!("Final Counters: {:?}", counters.counts);

    let updates2 = vec![
        (StreamID(0), vec![EMPTY_POINT]),
        (StreamID(2), vec![EMPTY_POINT, EMPTY_POINT]),
    ];

    let digest2 = counters.update(updates2);
    println!("Counters Digest: {:?}", digest2);

    // Print the final counters for debugging after the second update
    println!("Final Counters after second update: {:?}", counters.counts);
}

// Add tests here
#[cfg(test)]
mod tests {
    use std::hash;

    use super::*;

    #[test]
    fn test_counter_sparse_node() {
        let updates = vec![
            (StreamID(0), vec![EMPTY_POINT, EMPTY_POINT]),
            (StreamID(1), vec![EMPTY_POINT]),
        ];

        let mut counters = CounterSparseNode::new();
        let digest1 = counters.update(updates);
        assert_eq!(digest1.0.len(), DIGEST_LEN);

        let updates2 = vec![
            (StreamID(0), vec![EMPTY_POINT]),
            (StreamID(2), vec![EMPTY_POINT, EMPTY_POINT]),
        ];

        let digest2 = counters.update(updates2);
        assert_eq!(digest2.0.len(), DIGEST_LEN);

        // Print the final counters for debugging after the second update
        println!("Final Counters after second update: {:?}", counters.counts);
        assert_eq!(counters.counts.len(), 3);
        assert_eq!(counters.counts.get(&StreamID(0)).unwrap(), &3);
        assert_eq!(counters.counts.get(&StreamID(1)).unwrap(), &1);
        assert_eq!(counters.counts.get(&StreamID(2)).unwrap(), &2);
    }

    #[test]
    fn test_hash_chain_sparse_node() {
        let updates = vec![
            (StreamID(0), vec![EMPTY_POINT, EMPTY_POINT]),
            (StreamID(1), vec![EMPTY_POINT]),
        ];

        let mut hash_chain = HashChainSparseNode::new();
        let digest1 = hash_chain.update(updates);
        assert_eq!(digest1.0.len(), DIGEST_LEN);
        assert_eq!(hash_chain.heads.len(), 2);

        let updates2 = vec![
            (StreamID(0), vec![EMPTY_POINT]),
            (StreamID(2), vec![EMPTY_POINT, EMPTY_POINT, EMPTY_POINT]),
        ];

        let digest2 = hash_chain.update(updates2);

        // Print the final hash chain for debugging after the second update
        println!("Final Hash Chain after second update: {:?}", hash_chain.heads);
        assert_eq!(digest2.0.len(), DIGEST_LEN);
        assert_eq!(hash_chain.heads.len(), 3);
        
        // Check the final hash chain
        let head0 = hash_chain.heads.get(&StreamID(0)).unwrap();
        let head1 = hash_chain.heads.get(&StreamID(1)).unwrap();
        let head2 = hash_chain.heads.get(&StreamID(2)).unwrap();

        // Compute the hash chain as H(EMPTY_POINT) -> H(H(EMPTY_POINT)) -> H(H(H(EMPTY_POINT)))
        let mut hasher = Sha256::new();
        hasher.update(&EMPTY_POINT.0); // We always start with the empty point
        hasher.update(&EMPTY_POINT.0); // First update
        let digest = hasher.finalize();
        assert_eq!(head1.0, digest.as_slice());

        let mut hasher = Sha256::new();
        hasher.update(digest.as_slice());
        hasher.update(&EMPTY_POINT.0); // Second update
        let digest = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(digest.as_slice());
        hasher.update(&EMPTY_POINT.0); // Third update
        let digest = hasher.finalize();

        assert_eq!(head2.0, digest.as_slice());
        assert_eq!(head0, head2);
    }
}
 
