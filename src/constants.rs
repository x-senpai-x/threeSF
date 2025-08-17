//! Protocol constants from the paper and simulation parameters.

/// Network delay bound (simplified for simulation).
pub const DELTA: u64 = 1;

/// Security parameter for k-deep confirmation.
pub const KAPPA: u64 = 4;

/// Vote expiration period in slots.
pub const ETA: u64 = 5;
