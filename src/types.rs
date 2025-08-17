//! Core data structures for the 3SF protocol.
//! Blocks, checkpoints, votes, and other fundamental types.

use std::collections::HashMap;
use std::cmp::Ordering;

// Type shortcuts
pub type Hash = String;
pub type ValidatorId = u64;

// Main data structures

/// Transaction placeholder for this simulation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transaction {
    pub id: u64,
}

/// A blockchain block identified by its hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub hash: Hash,
    pub parent_hash: Hash,
    pub slot: u64,
    pub proposer_id: ValidatorId,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Create the genesis block (root of the chain).
    pub fn genesis() -> Self {
        Block {
            hash: "genesis_hash".to_string(),
            parent_hash: "null".to_string(),
            slot: 0,
            proposer_id: 0,
            transactions: vec![],
        }
    }

    /// Check if this block is an ancestor of another block.
    /// Walks the chain backwards through the view.
    pub fn is_ancestor_of(&self, other: &Block, view: &View) -> bool {
        let mut current_hash = other.parent_hash.clone();
        while current_hash != "null" {
            if current_hash == self.hash {
                return true;
            }
            let parent_block = view.blocks.get(&current_hash)
                .expect("Parent block must be in view for ancestry check");
            current_hash = parent_block.parent_hash.clone();
        }
        false
    }
}

/// A checkpoint: (block_hash, slot) pair.
/// See Section 3 for details.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Checkpoint {
    pub block_hash: Hash,
    pub slot: u64,
}

impl Ord for Checkpoint {
    /// Checkpoint ordering by slot number (Section 4).
    fn cmp(&self, other: &Self) -> Ordering {
        self.slot.cmp(&other.slot)
    }
}

impl PartialOrd for Checkpoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A validator's vote message for a slot.
/// Covers both head votes and FFG votes (Section 3 & 6).
#[derive(Debug, Clone)]
pub struct Vote {
    pub chain_head_hash: Hash,
    pub source: Checkpoint, // FFG vote source
    pub target: Checkpoint, // FFG vote target
    pub slot: u64,
    pub validator_id: ValidatorId,
}

/// Block proposal from a slot's designated proposer.
/// From Section 6, Algorithm 7, line 16.
#[derive(Debug, Clone)]
pub struct Proposal {
    pub chain_head_hash: Hash,
    pub view: View, // Proposer's current view
    pub slot: u64,
    pub proposer_id: ValidatorId,
}

/// A validator's view of the network state.
/// See Section 2.1.
#[derive(Debug, Clone, Default)]
pub struct View {
    pub blocks: HashMap<Hash, Block>,
    pub votes: Vec<Vote>,
}

/// Validator status options.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Adversary,
}

/// Validator identity and status.
#[derive(Debug, Clone)]
pub struct Validator {
    pub id: ValidatorId,
    pub status: ValidatorStatus,
}
