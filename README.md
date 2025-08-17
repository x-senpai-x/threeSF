# 3SF - 3-Slot Finality Protocol

Rust implementation of the 3-Slot Finality protocol for Ethereum consensus.

## What it does

- Implements FFG justification rules
- RLMD-GHOST fork choice algorithm  
- Simulates validator nodes and protocol phases
- Demonstrates finalization within 3 slots

## Run the simulation

```bash
cargo run
```

## Structure

- `src/ffg.rs` - FFG justification logic
- `src/fork_choice.rs` - RLMD-GHOST implementation
- `src/node.rs` - Validator node logic
- `src/types.rs` - Core data structures
- `src/main.rs` - Protocol simulation

## Reference

Based on: https://ethresear.ch/t/3-slot-finality-ssf-is-not-about-single-slot/20927
