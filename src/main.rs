//! 3-Slot Finality protocol simulation showing finalization across multiple slots.
//! Reference: https://ethresear.ch/t/3-slot-finality-ssf-is-not-about-single-slot/20927

use threeSF::node::Node;
use threeSF::types::{Vote, Checkpoint};
use threeSF::ffg;
use std::collections::HashMap;

fn main() {
    println!("=== 3-Slot Finality (3SF) Protocol Simulation ===");
    println!("Demonstrating finalization within 3 slots for honest proposers\n");

    // Set up 10 validator nodes
    let mut nodes: Vec<Node> = (0..10).map(Node::new).collect();
    let num_slots = 8; // Run enough slots to see finalization cycles
    
    println!("🔧 Initialized {} validator nodes", nodes.len());
    println!("📊 Simulating {} slots to demonstrate 3SF finality\n", num_slots);

    for current_slot in 1..=num_slots {
        simulate_slot(&mut nodes, current_slot);
        
        // Display protocol state after each slot
        display_protocol_state(&nodes, current_slot);
        
        // Check for finalization events
        check_finalization_status(&mut nodes, current_slot);
        
        println!("{}", "=".repeat(80));
    }

    println!("\n🎯 3SF Simulation Complete!");
    println!("The simulation demonstrates how blocks proposed by honest proposers");
    println!("achieve finalization within 3 slots under the 3SF protocol.");
}

fn simulate_slot(nodes: &mut Vec<Node>, slot: u64) {
    println!("🕐 SLOT {} - Beginning Protocol Phases", slot);
    
    // Pick proposer using round-robin
    let proposer_id = ((slot - 1) % nodes.len() as u64) as usize;
    println!("👤 Proposer: Node {}", proposer_id);
    
    // PROPOSE PHASE
    println!("📝 PROPOSE Phase:");
    let proposal = nodes[proposer_id].propose(slot);
    println!("   ✓ Node {} proposed block: {}", proposer_id, proposal.chain_head_hash);
    
    // Send proposal to all validators
    println!("📡 Distributing proposal to all validators...");
    for (i, node) in nodes.iter_mut().enumerate() {
        if i != proposer_id {
            node.on_receive_proposal(&proposal);
        }
    }
    
    // VOTE PHASE
    println!("🗳️  VOTE Phase:");
    let votes: Vec<Vote> = nodes.iter_mut().map(|node| {
        let vote = node.vote(slot);
        println!("   ✓ Node {} voted for head: {} (FFG: {} -> {})", 
                 vote.validator_id, 
                 vote.chain_head_hash,
                 format!("({}, {})", vote.source.block_hash, vote.source.slot),
                 format!("({}, {})", vote.target.block_hash, vote.target.slot));
        vote
    }).collect();
    
    // Broadcast votes to network
    println!("📡 Broadcasting {} votes to network...", votes.len());
    for node in nodes.iter_mut() {
        for vote in &votes {
            node.receive_message(None, Some(vote.clone()));
        }
    }
    
    // FAST CONFIRM PHASE
    println!("⚡ FAST CONFIRM Phase:");
    let mut fast_confirmations = 0;
    for node in nodes.iter_mut() {
        let old_ch_ava = node.ch_ava.clone();
        node.fast_confirm(slot);
        if node.ch_ava != old_ch_ava {
            fast_confirmations += 1;
        }
    }
    if fast_confirmations > 0 {
        println!("   ✓ {} nodes fast-confirmed blocks", fast_confirmations);
    } else {
        println!("   - No fast-confirmations in this slot");
    }
    
    // MERGE PHASE
    println!("🔄 MERGE Phase: Updating validator views");
    for node in nodes.iter_mut() {
        node.merge();
    }
}

fn display_protocol_state(nodes: &Vec<Node>, slot: u64) {
    println!("\n📊 Protocol State After Slot {}:", slot);
    
    // Show state from a few different nodes
    let sample_nodes = [0, 3, 7];
    for &node_id in &sample_nodes {
        if node_id < nodes.len() {
            let node = &nodes[node_id];
            println!("   Node {}: ch_ava={}, ch_fin={}", 
                     node_id, 
                     truncate_hash(&node.ch_ava), 
                     truncate_hash(&node.ch_fin));
        }
    }
    
    // Network-wide stats
    let total_blocks: usize = nodes[0].view.blocks.len();
    let total_votes: usize = nodes[0].view.votes.len();
    println!("   Network State: {} blocks, {} votes in view", total_blocks, total_votes);
}

fn check_finalization_status(nodes: &mut Vec<Node>, slot: u64) {
    if slot < 3 {
        return; // Need 3+ slots to check finalization
    }
    
    println!("\n🔍 Checking Finalization Status:");
    
    // Look at recent checkpoints for justification
    let mut justification_cache = HashMap::new();
    let node = &nodes[0]; // Use node 0's view
    
    // Check recent slots
    for check_slot in (slot.saturating_sub(2))..=slot {
        // Get blocks from this slot
        let slot_blocks: Vec<_> = node.view.blocks.values()
            .filter(|b| b.slot == check_slot)
            .collect();
            
        for block in slot_blocks {
            let checkpoint = Checkpoint {
                block_hash: block.hash.clone(),
                slot: check_slot,
            };
            
            let is_justified = ffg::is_justified(&checkpoint, &node.view, &mut justification_cache);
            if is_justified {
                println!("   ✅ JUSTIFIED: Block {} in slot {}", 
                         truncate_hash(&block.hash), check_slot);
                
                // Might be ready for finalization
                if check_slot <= slot.saturating_sub(2) {
                    println!("   🎯 POTENTIAL FINALIZATION: Block {} (proposed in slot {}) may be finalized", 
                             truncate_hash(&block.hash), check_slot);
                }
            }
        }
    }
    
    // Show 3SF property in action
    if slot >= 4 {
        println!("   📈 3SF Property: Blocks from slot {} should be approaching finalization", 
                 slot - 3);
    }
}

fn truncate_hash(hash: &str) -> String {
    if hash.len() > 12 {
        format!("{}...", &hash[..12])
    } else {
        hash.to_string()
    }
}
