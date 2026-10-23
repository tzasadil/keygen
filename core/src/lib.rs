//! Layout model, penalty scoring, and annealing math.
//! Kept free of threads, files, and stdout so it also builds for WebAssembly.

pub mod annealing;
pub mod layout;
pub mod penalty;
