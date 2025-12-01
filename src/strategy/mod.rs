//! Strategy module for Sudoku solving strategies.
//!
//! This module provides a JSON-based strategy system that allows defining
//! solving strategies in JSON files and applying them dynamically.
//!
//! # Components
//!
//! - `types`: Core type definitions for strategies, patterns, and actions
//! - `bank`: Strategy loading and management from JSON files
//! - `matcher`: Pattern matching implementations for various strategies
//! - `selector`: Strategy selection logic for choosing which strategy to apply

pub mod types;
pub mod bank;
pub mod matcher;
pub mod selector;

// Re-export commonly used types
pub use types::{
    Strategy, StrategyMetadata, StrategyPattern, StrategyAction,
    StrategyMatch, MatchContext, UnitType,
    PatternCondition, TargetCells, CandidateSource,
};

pub use bank::{StrategyBank, StrategyError};
pub use matcher::{PatternMatcher, create_matcher};
pub use selector::{StrategySelector, SelectionPolicy, StrategyStatistics};
