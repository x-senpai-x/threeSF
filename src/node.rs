//! Node implementation for validators in the 3SF protocol.
//! Coordinates FFG and fork choice logic.

use std::collections::HashMap;
use crate::types::*;
use crate::constants::*;
use crate::ffg;
use crate::fork_choice;

/// A validator node's complete state in the 3SF protocol.
/// Matches the `v_i` state from Algorithm 7.
pub struct Node {
    pub validator: Validator,
    pub view: View,
    pub frozen_view: View, // `V_i^frozen` in the paper
    pub ch_ava: Hash,      // Available chain head
    pub ch_fin: Hash,      // Finalized chain head
    // Cache results to speed up repeated calculations
    justification_cache: HashMap<Checkpoint, bool>,
    finalization_cache: HashMap<Checkpoint, bool>,
}

impl Node {
    /// Initialize a new validator node starting from genesis.
    pub fn new(id: ValidatorId) -> Self {
        let genesis_block = Block::genesis();
        let genesis_hash = genesis_block.hash.clone();
        let mut initial_view = View::default();
        initial_view.blocks.insert(genesis_hash.clone(), genesis_block);

        Node {
            validator: Validator { id, status: ValidatorStatus::Active },
            view: initial_view.clone(),
            frozen_view: initial_view,
            ch_ava: genesis_hash.clone(),
            ch_fin: genesis_hash,
            justification_cache: HashMap::new(),
            finalization_cache: HashMap::new(),
        }
    }

    /// Handle incoming blocks and votes from the network.
    pub fn receive_message(&mut self, block: Option<Block>, vote: Option<Vote>) {
        if let Some(b) = block {
            self.view.blocks.entry(b.hash.clone()).or_insert(b);
        }
        if let Some(v) = vote {
            self.view.votes.push(v);
        }
    }

    // Core 3SF protocol phases from Algorithm 7

    /// Propose a new block for this slot.
    /// See Algorithm 7, lines 13-16.
    pub fn propose(&mut self, current_slot: u64) -> Proposal {
        println!("Node {} PROPOSING for slot {}", self.validator.id, current_slot);

        let gjc = ffg::greatest_justified_checkpoint(&self.view, &mut self.justification_cache);
        let head_hash = fork_choice::rlmd_ghost_fork_choice(&self.view, gjc.block_hash, current_slot);

        // Create new block extending the chosen head
        let new_block = Block {
            hash: format!("block_slot_{}_proposer_{}", current_slot, self.validator.id),
            parent_hash: head_hash,
            slot: current_slot,
            proposer_id: self.validator.id,
            transactions: vec![], // Empty for this simulation
        };
        self.view.blocks.insert(new_block.hash.clone(), new_block.clone());

        Proposal {
            chain_head_hash: new_block.hash,
            view: self.view.clone(), // Share our view with other validators
            slot: current_slot,
            proposer_id: self.validator.id,
        }
    }

    /// Process a proposal from another validator.
    /// From Algorithm 7, lines 30-31.
    pub fn on_receive_proposal(&mut self, proposal: &Proposal) {
        println!("Node {} received proposal for slot {}", self.validator.id, proposal.slot);
        // Add proposer's blocks and votes to our frozen view
        for (hash, block) in &proposal.view.blocks {
            self.frozen_view.blocks.entry(hash.clone()).or_insert(block.clone());
        }
        for vote in &proposal.view.votes {
            self.frozen_view.votes.push(vote.clone());
        }
    }

    /// Cast our vote for this slot.
    /// See Algorithm 7, lines 18-22.
    pub fn vote(&mut self, current_slot: u64) -> Vote {
        println!("Node {} VOTING for slot {}", self.validator.id, current_slot);

        let gjc_frozen = ffg::greatest_justified_checkpoint(&self.frozen_view, &mut self.justification_cache);
        let head_hash = fork_choice::rlmd_ghost_fork_choice(&self.frozen_view, gjc_frozen.block_hash.clone(), current_slot);
        
        let head_block = self.frozen_view.blocks.get(&head_hash).unwrap();

        // Update chAva based on k-deep rule
        let k_deep_prefix = self.get_k_deep_prefix(head_block, KAPPA);
        self.ch_ava = vec![self.ch_ava.clone(), k_deep_prefix, gjc_frozen.block_hash.clone()]
            .into_iter()
            .max_by_key(|h| self.frozen_view.blocks.get(h).unwrap().slot)
            .unwrap();

        // Build FFG vote with source and target checkpoints
        let source = gjc_frozen;
        let target = Checkpoint { block_hash: self.ch_ava.clone(), slot: current_slot };

        Vote {
            chain_head_hash: head_hash,
            source,
            target,
            slot: current_slot,
            validator_id: self.validator.id,
        }
    }

    /// Try to fast-confirm blocks with supermajority support.
    /// From Algorithm 7, lines 24-27.
    pub fn fast_confirm(&mut self, current_slot: u64) {
        let mut vote_counts: HashMap<Hash, usize> = HashMap::new();
        for vote in &self.view.votes {
            if vote.slot == current_slot {
                *vote_counts.entry(vote.chain_head_hash.clone()).or_insert(0) += 1;
            }
        }
        
        let n = 100; // Validator count
        if let Some((fast_cand, _count)) = vote_counts.iter().find(|(_, count)| **count as u64 > (2 * n / 3)) {
             println!("Node {} FAST CONFIRMING {} in slot {}", self.validator.id, fast_cand, current_slot);
             self.ch_ava = fast_cand.clone();
             // TODO: implement full finalization logic with GF(V)
        }
    }

    /// Merge our view with frozen view to end the slot.
    /// Algorithm 7, line 29.
    pub fn merge(&mut self) {
        println!("Node {} MERGING view", self.validator.id);
        self.frozen_view = self.view.clone();
        // Reset caches for next slot
        self.justification_cache.clear();
        self.finalization_cache.clear();
    }
    
    /// Find the block that's k slots back from the head.
    fn get_k_deep_prefix(&self, head_block: &Block, k: u64) -> Hash {
        let mut current_block = head_block.clone();
        // Walk back k slots from the head
        while current_block.slot > head_block.slot.saturating_sub(k) {
            if current_block.parent_hash == "null" {
                break;
            }
            current_block = self.frozen_view.blocks.get(&current_block.parent_hash).unwrap().clone();
        }
        current_block.hash
    }
}
