//! Strategy selection for choosing which strategy to apply next.
//!
//! The StrategySelector analyzes the current board state and selects
//! the most appropriate strategy to apply based on various policies.

use super::types::{Strategy, StrategyMatch};
use super::matcher::{PatternMatcher, create_matcher};
use crate::board::Board;
use std::collections::HashMap;

/// Policy for selecting strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionPolicy {
    /// Select by priority (highest priority first)
    Priority,
    
    /// Select by difficulty (easiest first)
    Difficulty,
    
    /// Select the first strategy that finds a match
    FirstMatch,
}

/// Statistics about strategy usage
#[derive(Debug, Clone)]
pub struct StrategyStatistics {
    /// Number of times each strategy was applied
    pub application_count: HashMap<String, usize>,
    
    /// Number of times each strategy found matches
    pub match_count: HashMap<String, usize>,
    
    /// Total number of eliminations per strategy
    pub elimination_count: HashMap<String, usize>,
}

impl StrategyStatistics {
    /// Creates new empty statistics
    pub fn new() -> Self {
        Self {
            application_count: HashMap::new(),
            match_count: HashMap::new(),
            elimination_count: HashMap::new(),
        }
    }
    
    /// Records that a strategy was applied
    pub fn record_application(&mut self, strategy_name: &str) {
        *self.application_count.entry(strategy_name.to_string()).or_insert(0) += 1;
    }
    
    /// Records that a strategy found matches
    pub fn record_match(&mut self, strategy_name: &str, match_count: usize) {
        *self.match_count.entry(strategy_name.to_string()).or_insert(0) += match_count;
    }
    
    /// Records eliminations made by a strategy
    pub fn record_eliminations(&mut self, strategy_name: &str, count: usize) {
        *self.elimination_count.entry(strategy_name.to_string()).or_insert(0) += count;
    }
    
    /// Gets the total number of applications
    pub fn total_applications(&self) -> usize {
        self.application_count.values().sum()
    }
    
    /// Gets the most used strategy
    pub fn most_used_strategy(&self) -> Option<(&String, &usize)> {
        self.application_count.iter().max_by_key(|(_, &count)| count)
    }
}

impl Default for StrategyStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Selects strategies to apply to a board
pub struct StrategySelector {
    /// The selection policy to use
    policy: SelectionPolicy,
    
    /// Statistics about strategy usage
    statistics: StrategyStatistics,
    
    /// Cache of pattern matchers
    matcher_cache: HashMap<String, Box<dyn PatternMatcher>>,
}

impl StrategySelector {
    /// Creates a new strategy selector with the given policy
    pub fn new(policy: SelectionPolicy) -> Self {
        Self {
            policy,
            statistics: StrategyStatistics::new(),
            matcher_cache: HashMap::new(),
        }
    }
    
    /// Creates a selector with default policy (Priority)
    pub fn default() -> Self {
        Self::new(SelectionPolicy::Priority)
    }
    
    /// Gets the selection policy
    pub fn policy(&self) -> SelectionPolicy {
        self.policy
    }
    
    /// Sets the selection policy
    pub fn set_policy(&mut self, policy: SelectionPolicy) {
        self.policy = policy;
    }
    
    /// Gets the statistics
    pub fn statistics(&self) -> &StrategyStatistics {
        &self.statistics
    }
    
    /// Selects the next strategy to apply
    /// 
    /// Returns the strategy and its matches, or None if no strategy applies
    pub fn select_strategy<'a>(
        &mut self,
        board: &Board,
        strategies: &'a [Strategy],
    ) -> Option<(&'a Strategy, Vec<StrategyMatch>)> {
        match self.policy {
            SelectionPolicy::Priority => self.select_by_priority(board, strategies),
            SelectionPolicy::Difficulty => self.select_by_difficulty(board, strategies),
            SelectionPolicy::FirstMatch => self.select_first_match(board, strategies),
        }
    }
    
    /// Selects strategy by priority (highest first)
    fn select_by_priority<'a>(
        &mut self,
        board: &Board,
        strategies: &'a [Strategy],
    ) -> Option<(&'a Strategy, Vec<StrategyMatch>)> {
        // Sort strategies by priority (highest first)
        let mut sorted_strategies: Vec<&Strategy> = strategies.iter().collect();
        sorted_strategies.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Try each strategy in priority order
        for strategy in sorted_strategies {
            if let Some(matches) = self.find_matches(board, strategy) {
                if !matches.is_empty() {
                    self.statistics.record_match(&strategy.metadata.name, matches.len());
                    return Some((strategy, matches));
                }
            }
        }
        
        None
    }
    
    /// Selects strategy by difficulty (easiest first)
    fn select_by_difficulty<'a>(
        &mut self,
        board: &Board,
        strategies: &'a [Strategy],
    ) -> Option<(&'a Strategy, Vec<StrategyMatch>)> {
        // Sort strategies by difficulty (lowest first)
        let mut sorted_strategies: Vec<&Strategy> = strategies.iter().collect();
        sorted_strategies.sort_by_key(|s| s.metadata.difficulty);
        
        // Try each strategy in difficulty order
        for strategy in sorted_strategies {
            if let Some(matches) = self.find_matches(board, strategy) {
                if !matches.is_empty() {
                    self.statistics.record_match(&strategy.metadata.name, matches.len());
                    return Some((strategy, matches));
                }
            }
        }
        
        None
    }
    
    /// Selects the first strategy that finds a match
    fn select_first_match<'a>(
        &mut self,
        board: &Board,
        strategies: &'a [Strategy],
    ) -> Option<(&'a Strategy, Vec<StrategyMatch>)> {
        for strategy in strategies {
            if let Some(matches) = self.find_matches(board, strategy) {
                if !matches.is_empty() {
                    self.statistics.record_match(&strategy.metadata.name, matches.len());
                    return Some((strategy, matches));
                }
            }
        }
        
        None
    }
    
    /// Finds matches for a strategy using the appropriate matcher
    fn find_matches(&mut self, board: &Board, strategy: &Strategy) -> Option<Vec<StrategyMatch>> {
        // Get or create matcher for this strategy
        let matcher = if let Some(matcher) = self.matcher_cache.get(&strategy.metadata.name) {
            matcher
        } else {
            // Create new matcher
            if let Some(matcher) = create_matcher(&strategy.metadata.name) {
                self.matcher_cache.insert(strategy.metadata.name.clone(), matcher);
                self.matcher_cache.get(&strategy.metadata.name)?
            } else {
                return None;
            }
        };
        
        // Find matches
        let matches = matcher.find_matches(board, strategy);
        Some(matches)
    }
    
    /// Applies a strategy match to the board
    pub fn apply_match(
        &mut self,
        board: &mut Board,
        strategy_match: &StrategyMatch,
    ) -> Result<bool, String> {
        let mut progress = false;
        
        // Record application
        self.statistics.record_application(&strategy_match.strategy_name);
        
        // Apply based on context
        if let Some(cell_idx) = strategy_match.context.cell_to_set {
            if let Some(value) = strategy_match.context.value_to_set {
                // Set cell value
                board.set_cell_value(cell_idx, value)?;
                progress = true;
            }
        }
        
        // Eliminate candidates
        if !strategy_match.context.elimination_targets.is_empty() {
            let mut elimination_count = 0;
            
            for &cell_idx in &strategy_match.context.elimination_targets {
                if let Some(cell) = board.get_cell_mut(cell_idx) {
                    for &candidate in &strategy_match.context.candidates_to_eliminate {
                        if cell.remove_candidate(candidate) {
                            elimination_count += 1;
                            progress = true;
                        }
                    }
                }
            }
            
            if elimination_count > 0 {
                self.statistics.record_eliminations(
                    &strategy_match.strategy_name,
                    elimination_count,
                );
            }
        }
        
        Ok(progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_creation() {
        let selector = StrategySelector::new(SelectionPolicy::Priority);
        assert_eq!(selector.policy(), SelectionPolicy::Priority);
    }

    #[test]
    fn test_policy_change() {
        let mut selector = StrategySelector::new(SelectionPolicy::Priority);
        selector.set_policy(SelectionPolicy::Difficulty);
        assert_eq!(selector.policy(), SelectionPolicy::Difficulty);
    }

    #[test]
    fn test_statistics() {
        let mut stats = StrategyStatistics::new();
        
        stats.record_application("test_strategy");
        stats.record_application("test_strategy");
        stats.record_match("test_strategy", 3);
        stats.record_eliminations("test_strategy", 5);
        
        assert_eq!(stats.application_count.get("test_strategy"), Some(&2));
        assert_eq!(stats.match_count.get("test_strategy"), Some(&3));
        assert_eq!(stats.elimination_count.get("test_strategy"), Some(&5));
        assert_eq!(stats.total_applications(), 2);
    }

    #[test]
    fn test_most_used_strategy() {
        let mut stats = StrategyStatistics::new();
        
        stats.record_application("strategy1");
        stats.record_application("strategy2");
        stats.record_application("strategy2");
        stats.record_application("strategy2");
        
        let most_used = stats.most_used_strategy();
        assert!(most_used.is_some());
        let (name, count) = most_used.unwrap();
        assert_eq!(name, "strategy2");
        assert_eq!(*count, 3);
    }
}
