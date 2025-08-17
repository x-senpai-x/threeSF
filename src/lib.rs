//! 3-Slot Finality Protocol Implementation in Rust
//!
//! This crate implements the 3-Slot Finality (3SF) protocol for Ethereum,
//! focusing on the RLMD-GHOST version from Section 6 of the paper.
//! It is structured as a library with a simulation binary.

pub mod constants;
pub mod types;
pub mod ffg;
pub mod fork_choice;
pub mod node;
