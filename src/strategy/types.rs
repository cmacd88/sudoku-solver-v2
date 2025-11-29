//! Type definitions for the strategy system.
//!
//! This module defines the core types used in the JSON strategy system,
//! including strategy definitions, patterns, actions, and matches.

use serde::{Deserialize, Serialize};

/// A complete strategy definition loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    /// Metadata about the strategy
    #[serde(flatten)]
    pub metadata: StrategyMetadata,
    
    /// The pattern to match
    pub pattern: StrategyPattern,
    
    /// The action to take when pattern matches
    pub action: StrategyAction,
    
    /// Priority for strategy selection (higher = more priority)
    #[serde(default = "default_priority")]
    pub priority: u32,
}

fn default_priority() -> u32 {
    50
}

/// Metadata about a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetadata {
    /// Unique name of the strategy
    pub name: String,
    
    /// Difficulty level (1 = easiest, higher = harder)
    pub difficulty: u32,
    
    /// Human-readable description
    pub description: String,
    
    /// Board dimensions this strategy applies to (e.g., ["9x9", "6x6"])
    #[serde(default = "default_dimensions")]
    pub applicable_dimensions: Vec<String>,
}

fn default_dimensions() -> Vec<String> {
    vec!["9x9".to_string()]
}

/// Pattern types that can be matched
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StrategyPattern {
    /// Match cells with specific candidate properties
    CellGroup {
        /// Which unit types to check (row, column, box)
        unit_type: Vec<UnitType>,
        
        /// Conditions that must be met
        conditions: Vec<PatternCondition>,
    },
    
    /// Match a single cell with specific properties
    SingleCell {
        /// Conditions for the cell
        conditions: Vec<PatternCondition>,
    },
    
    /// Match candidates pointing in a direction
    PointingCandidates {
        /// Source unit type (usually box)
        source_unit: UnitType,
        
        /// Target unit type (usually row or column)
        target_unit: UnitType,
        
        /// Conditions for the pattern
        conditions: Vec<PatternCondition>,
    },
    
    /// Match patterns across multiple parallel units (X-Wing, Swordfish)
    CrossUnit {
        /// Which unit types to check (row or column)
        unit_type: Vec<UnitType>,
        
        /// Conditions for the pattern
        conditions: Vec<PatternCondition>,
    },
    
    /// Match chain patterns (XY-Wing, XYZ-Wing, etc.)
    ChainPattern {
        /// Type of chain (xy_wing, xyz_wing, etc.)
        chain_type: String,
        
        /// Conditions for the pattern
        conditions: Vec<PatternCondition>,
    },
}

/// Types of constraint units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitType {
    Row,
    Column,
    Box,
}

/// Conditions for pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PatternCondition {
    /// Exact number of cells must match
    CellCount {
        count: usize,
    },
    
    /// Exact number of candidates in matched cells
    CandidateCount {
        count: usize,
    },
    
    /// All matched cells must have the same candidates
    SameCandidates {
        value: bool,
    },
    
    /// Cell must have only one candidate
    SingleCandidate,
    
    /// Value can only appear in specific number of cells
    ValueOccurrences {
        value: Option<u8>,
        count: usize,
    },
    
    /// Candidates must be restricted to a line within a box
    RestrictedToLine {
        value: bool,
    },
    
    /// Number of parallel units (for X-Wing, Swordfish)
    ParallelUnitCount {
        count: usize,
    },
    
    /// Number of positions per unit (for X-Wing, Swordfish)
    PositionsPerUnit {
        #[serde(default)]
        count: Option<usize>,
        #[serde(default)]
        min: Option<usize>,
        #[serde(default)]
        max: Option<usize>,
    },
    
    /// Positions must be aligned across units
    AlignedPositions {
        value: bool,
    },
    
    /// All matched cells must have the same candidate
    SameCandidate {
        value: bool,
    },
    
    /// Pivot cell must have exactly N candidates (for XY-Wing)
    PivotCandidates {
        count: usize,
    },
    
    /// Wing cells must have exactly N candidates (for XY-Wing)
    WingCandidates {
        count: usize,
    },
    
    /// Cells must form proper XY-Wing structure
    SharedCandidateStructure {
        value: bool,
    },
}

/// Actions to take when a pattern matches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StrategyAction {
    /// Eliminate candidates from cells
    EliminateCandidates {
        /// Which cells to target
        target: TargetCells,
        
        /// Which candidates to eliminate
        candidates: CandidateSource,
    },
    
    /// Set a cell value
    SetCellValue {
        /// Which cell to set
        target: TargetCells,
        
        /// Which value to set
        value: CandidateSource,
    },
}

/// Specifies which cells to target with an action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetCells {
    /// Cells in the matched pattern
    MatchedCells,
    
    /// Other cells in the same unit
    OtherCellsInUnit,
    
    /// Cells in a different unit
    CellsInTargetUnit,
    
    /// All peer cells
    PeerCells,
    
    /// Cells in perpendicular units (for X-Wing, Swordfish)
    PerpendicularUnits,
    
    /// Cells that can see both wing cells (for XY-Wing)
    CellsSeeingBothWings,
}

/// Specifies which candidates to use
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateSource {
    /// Candidates from matched cells
    MatchedCandidates,
    
    /// A specific value
    SpecificValue { value: u8 },
    
    /// The single remaining candidate
    SingleCandidate,
    
    /// The common candidate in wing cells (for XY-Wing)
    CommonWingCandidate,
}

/// A match found by a pattern matcher
#[derive(Debug, Clone)]
pub struct StrategyMatch {
    /// The strategy that matched
    pub strategy_name: String,
    
    /// Indices of cells involved in the match
    pub cell_indices: Vec<usize>,
    
    /// The unit where the match was found (if applicable)
    pub unit_type: Option<UnitType>,
    
    /// The unit index (row/column/box number)
    pub unit_index: Option<usize>,
    
    /// Candidates involved in the match
    pub candidates: Vec<u8>,
    
    /// Additional context for the match
    pub context: MatchContext,
}

/// Additional context about a match
#[derive(Debug, Clone)]
pub struct MatchContext {
    /// Cells to eliminate candidates from
    pub elimination_targets: Vec<usize>,
    
    /// Candidates to eliminate
    pub candidates_to_eliminate: Vec<u8>,
    
    /// Cell to set value in (if applicable)
    pub cell_to_set: Option<usize>,
    
    /// Value to set (if applicable)
    pub value_to_set: Option<u8>,
}

impl StrategyMatch {
    /// Creates a new strategy match
    pub fn new(
        strategy_name: String,
        cell_indices: Vec<usize>,
        unit_type: Option<UnitType>,
        unit_index: Option<usize>,
        candidates: Vec<u8>,
        context: MatchContext,
    ) -> Self {
        Self {
            strategy_name,
            cell_indices,
            unit_type,
            unit_index,
            candidates,
            context,
        }
    }
}

impl MatchContext {
    /// Creates a new empty match context
    pub fn new() -> Self {
        Self {
            elimination_targets: Vec::new(),
            candidates_to_eliminate: Vec::new(),
            cell_to_set: None,
            value_to_set: None,
        }
    }
    
    /// Creates a context for eliminating candidates
    pub fn elimination(targets: Vec<usize>, candidates: Vec<u8>) -> Self {
        Self {
            elimination_targets: targets,
            candidates_to_eliminate: candidates,
            cell_to_set: None,
            value_to_set: None,
        }
    }
    
    /// Creates a context for setting a cell value
    pub fn set_value(cell: usize, value: u8) -> Self {
        Self {
            elimination_targets: Vec::new(),
            candidates_to_eliminate: Vec::new(),
            cell_to_set: Some(cell),
            value_to_set: Some(value),
        }
    }
}

impl Default for MatchContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_deserialization() {
        let json = r#"{
            "name": "naked_single",
            "difficulty": 1,
            "description": "A cell with only one candidate",
            "pattern": {
                "type": "single_cell",
                "conditions": [
                    {"type": "single_candidate"}
                ]
            },
            "action": {
                "type": "set_cell_value",
                "target": "matched_cells",
                "value": "single_candidate"
            },
            "priority": 100
        }"#;
        
        let strategy: Result<Strategy, _> = serde_json::from_str(json);
        assert!(strategy.is_ok());
        
        let strategy = strategy.unwrap();
        assert_eq!(strategy.metadata.name, "naked_single");
        assert_eq!(strategy.metadata.difficulty, 1);
        assert_eq!(strategy.priority, 100);
    }

    #[test]
    fn test_match_context_creation() {
        let ctx = MatchContext::elimination(vec![0, 1, 2], vec![5, 6]);
        assert_eq!(ctx.elimination_targets.len(), 3);
        assert_eq!(ctx.candidates_to_eliminate.len(), 2);
        assert!(ctx.cell_to_set.is_none());
        
        let ctx = MatchContext::set_value(5, 7);
        assert_eq!(ctx.cell_to_set, Some(5));
        assert_eq!(ctx.value_to_set, Some(7));
        assert!(ctx.elimination_targets.is_empty());
    }
}
