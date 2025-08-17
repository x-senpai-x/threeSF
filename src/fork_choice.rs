//! RLMD-GHOST fork choice implementation.
//! Based on Section 6.1 and Algorithm 5.

use std::collections::{HashMap, HashSet};
use crate::types::*;
use crate::constants::ETA;

/// Filters votes using RLMD rules: keeps latest, removes expired and equivocating votes.
/// This is `FIL_rlmd(V, t)` from Algorithm 5.
fn filter_rlmd_votes(view: &View, current_slot: u64) -> HashMap<ValidatorId, Vote> {
    let mut latest_votes: HashMap<ValidatorId, &Vote> = HashMap::new();
    let mut equivocators = HashSet::new();

    // Find latest votes per validator and catch equivocators
    for vote in &view.votes {
        // Skip votes that are too old
        if vote.slot < current_slot.saturating_sub(ETA) {
            continue;
        }

        if let Some(latest) = latest_votes.get(&vote.validator_id) {
            if vote.slot > latest.slot {
                latest_votes.insert(vote.validator_id, vote);
            } else if vote.slot == latest.slot && vote.chain_head_hash != latest.chain_head_hash {
                // Voting for different heads in same slot = equivocation
                equivocators.insert(vote.validator_id);
            }
        } else {
            latest_votes.insert(vote.validator_id, vote);
        }
    }

    // Build final vote set, excluding equivocators
    latest_votes.into_iter()
        .filter(|(id, _)| !equivocators.contains(id))
        .map(|(id, vote)| (id, vote.clone()))
        .collect()
}

/// GHOST rule: follow the heaviest subtree at each fork.
/// This is `GHOST(V, B_start)` from Algorithm 5.
fn ghost(view: &View, filtered_votes: &HashMap<ValidatorId, Vote>, start_hash: Hash) -> Hash {
    let mut current_hash = start_hash;

    loop {
        // Get all child blocks
        let children: Vec<_> = view.blocks.values()
            .filter(|b| b.parent_hash == current_hash)
            .collect();

        if children.is_empty() {
            break; // No more children, we found the head
        }

        // Pick the child with most votes in its subtree
        let best_child = children.iter()
            .max_by_key(|child_block| {
                filtered_votes.values().filter(|vote| {
                    // Make sure the voted block exists in our view
                    if let Some(vote_block) = view.blocks.get(&vote.chain_head_hash) {
                        // Vote counts if it's for this child or any descendant
                        child_block.hash == vote_block.hash || child_block.is_ancestor_of(vote_block, view)
                    } else {
                        false // Ignore votes for unknown blocks
                    }
                }).count()
            })
            .unwrap(); // There's always at least one child here

        current_hash = best_child.hash.clone();
    }
    current_hash
}

/// Complete RLMD-GHOST fork choice algorithm.
/// This is `RLMD-GHOST(V, B_start, t)` from Algorithm 5.
pub fn rlmd_ghost_fork_choice(view: &View, start_hash: Hash, current_slot: u64) -> Hash {
    let filtered_votes = filter_rlmd_votes(view, current_slot);
    ghost(view, &filtered_votes, start_hash)
}
