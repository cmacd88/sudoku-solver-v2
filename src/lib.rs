//! Sudoku Solver v2 - MVP
//!
//! A high-performance Sudoku solver using constraint propagation with view abstractions.

pub mod board;
pub mod solver;
pub mod strategy;
pub mod io;
pub mod logging;

// Re-export commonly used types
pub use board::Board;
pub use solver::{Solver, SolverError, SolverResult};
pub use solver::speculative::{SpeculationConfig, SpeculationMode, SpeculationStatistics};
