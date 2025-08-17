//! FFG (Friendly Finality Gadget) implementation.
//! Handles checkpoint justification based on Section 4.

use std::collections::{HashMap, HashSet};
use crate::types::*;

/// Determines if a checkpoint is justified given the current view.
/// Uses recursion with caching for efficiency. Based on Algorithm 1's `J(C, V)`.
pub fn is_justified(
    checkpoint: &Checkpoint,
    view: &View,
    justification_cache: &mut HashMap<Checkpoint, bool>,
) -> bool {
    // Use cache to skip redundant calculations
    if let Some(&is_justified) = justification_cache.get(checkpoint) {
        return is_justified;
    }

    // Genesis is always justified
    if checkpoint.block_hash == "genesis_hash" && checkpoint.slot == 0 {
        justification_cache.insert(checkpoint.clone(), true);
        return true;
    }

    let mut supermajority_voters = HashSet::new();
    for vote in &view.votes {
        // Vote target slot must match checkpoint slot
        if vote.target.slot == checkpoint.slot {
            // Source checkpoint must also be justified (recursive check)
            if is_justified(&vote.source, view, justification_cache) {
                let source_block = view.blocks.get(&vote.source.block_hash).unwrap();
                let target_block = view.blocks.get(&vote.target.block_hash).unwrap();
                let checkpoint_block = view.blocks.get(&checkpoint.block_hash).unwrap();

                // Check ancestry: source <= checkpoint <= target
                if source_block.is_ancestor_of(checkpoint_block, view) &&
                   checkpoint_block.is_ancestor_of(target_block, view) {
                    supermajority_voters.insert(vote.validator_id);
                }
            }
        }
    }
    
    let n = 100; // Validator count for this simulation
    let result = supermajority_voters.len() as u64 > (2 * n / 3);
    justification_cache.insert(checkpoint.clone(), result);
    result
}

/// Returns the highest justified checkpoint by slot number.
/// See Section 4 for ordering rules.
pub fn greatest_justified_checkpoint(
    view: &View,
    justification_cache: &mut HashMap<Checkpoint, bool>,
) -> Checkpoint {
    view.votes.iter()
        .map(|v| v.source.clone())
        .filter(|cp| is_justified(cp, view, justification_cache))
        .max()
        .unwrap_or(Checkpoint { block_hash: "genesis_hash".to_string(), slot: 0 })
}
