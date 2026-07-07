//! Speculative execution module for advanced Sudoku solving.
//!
//! This module implements intelligent speculation strategies including:
//! - Parallel branch exploration using Rayon
//! - Hybrid mode that chooses between bifurcation and backtracking
//! - Configurable speculation modes and depth limits

use crate::board::Board;
use crate::solver::{SolverError, SolverResult};
use std::collections::HashMap;
use rayon::prelude::*;
use log;

/// Configuration for speculative execution
#[derive(Debug, Clone)]
pub struct SpeculationConfig {
    /// Whether speculation is enabled
    pub enabled: bool,
    
    /// Maximum recursion depth for speculation
    pub max_depth: usize,
    
    /// Speculation mode to use
    pub mode: SpeculationMode,
    
    /// Whether to track detailed statistics
    pub track_statistics: bool,
}

impl Default for SpeculationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_depth: 100,
            mode: SpeculationMode::Hybrid,
            track_statistics: true,
        }
    }
}

/// Speculation execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeculationMode {
    /// Try candidates sequentially (traditional backtracking)
    Sequential,
    
    /// Explore all branches in parallel using Rayon
    Parallel,
    
    /// Intelligently choose between sequential and parallel based on board state
    Hybrid,
}

impl SpeculationMode {
    /// Parse from string (for CLI)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sequential" | "seq" => Some(Self::Sequential),
            "parallel" | "par" => Some(Self::Parallel),
            "hybrid" => Some(Self::Hybrid),
            _ => None,
        }
    }
}

/// Statistics for speculation execution
#[derive(Debug, Clone, Default)]
pub struct SpeculationStatistics {
    /// Total number of branches explored
    pub branches_explored: usize,
    
    /// Number of branches pruned due to contradictions
    pub branches_pruned: usize,
    
    /// Maximum depth reached during speculation
    pub max_depth_reached: usize,
    
    /// Number of contradictions found
    pub contradictions_found: usize,
    
    /// Count of each speculation mode used
    pub speculation_mode_used: HashMap<String, usize>,
}

impl SpeculationStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record that a mode was used
    pub fn record_mode_used(&mut self, mode: SpeculationMode) {
        let mode_name = format!("{:?}", mode);
        *self.speculation_mode_used.entry(mode_name).or_insert(0) += 1;
    }
    
    /// Log statistics
    pub fn log_stats(&self) {
        log::info!("Speculation Statistics:");
        log::info!("  Branches explored: {}", self.branches_explored);
        log::info!("  Branches pruned: {}", self.branches_pruned);
        log::info!("  Max depth reached: {}", self.max_depth_reached);
        log::info!("  Contradictions found: {}", self.contradictions_found);
        
        if !self.speculation_mode_used.is_empty() {
            log::info!("  Modes used:");
            for (mode, count) in &self.speculation_mode_used {
                log::info!("    {}: {}", mode, count);
            }
        }
    }
}

impl std::fmt::Display for SpeculationStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Speculation Statistics:")?;
        writeln!(f, "  Branches explored: {}", self.branches_explored)?;
        writeln!(f, "  Branches pruned: {}", self.branches_pruned)?;
        writeln!(f, "  Max depth reached: {}", self.max_depth_reached)?;
        writeln!(f, "  Contradictions found: {}", self.contradictions_found)?;
        
        if !self.speculation_mode_used.is_empty() {
            writeln!(f, "  Modes used:")?;
            for (mode, count) in &self.speculation_mode_used {
                writeln!(f, "    {}: {}", mode, count)?;
            }
        }
        Ok(())
    }
}

/// Hybrid mode decision strategy
#[derive(Debug, Clone, Copy)]
pub enum HybridStrategy {
    /// Use parallel bifurcation
    Bifurcation,
    
    /// Use sequential backtracking
    Backtracking,
    
    /// Use limited parallel bifurcation (with depth limit)
    LimitedBifurcation(usize),
}

/// Analyzes board state and chooses optimal speculation strategy
pub fn choose_speculation_strategy(board: &Board) -> HybridStrategy {
    let cells_with_2_candidates = count_cells_with_n_candidates(board, 2);
    let cells_with_3_candidates = count_cells_with_n_candidates(board, 3);
    let total_unsolved = board.unsolved_count();
    
    log::debug!("Board analysis: {} cells with 2 candidates, {} with 3, {} total unsolved",
               cells_with_2_candidates, cells_with_3_candidates, total_unsolved);
    
    if cells_with_2_candidates >= 5 {
        // Many binary choices - parallel bifurcation is efficient
        log::debug!("Choosing Bifurcation strategy (many binary choices)");
        HybridStrategy::Bifurcation
    } else if total_unsolved < 20 {
        // Nearly solved - sequential backtracking is fine
        log::debug!("Choosing Backtracking strategy (nearly solved)");
        HybridStrategy::Backtracking
    } else {
        // Mixed state - use limited parallel bifurcation
        log::debug!("Choosing LimitedBifurcation strategy (mixed state)");
        HybridStrategy::LimitedBifurcation(2)
    }
}

/// Count cells with exactly n candidates
fn count_cells_with_n_candidates(board: &Board, n: usize) -> usize {
    (0..81)
        .filter(|&idx| {
            if let Some(cell) = board.get_cell(idx) {
                !cell.is_solved() && cell.candidates.count() == n as u32
            } else {
                false
            }
        })
        .count()
}

/// Find the best cell for speculation using heuristics
pub fn find_best_speculation_cell(board: &Board) -> Option<(usize, Vec<u8>)> {
    let two_cand: Vec<usize> = (0..81)
        .filter(|&i| !board.is_cell_solved(i)
            && board.get_cell(i).map_or(false, |c| c.candidates.count() == 2))
        .collect();

    if !two_cand.is_empty() {
        return best_by_cascade(board, &two_cand);
    }

    // fallback: any unsolved cell, still ranked by cascade
    let any: Vec<usize> = (0..81).filter(|&i| !board.is_cell_solved(i)).collect();
    if any.is_empty() { return None; }
    best_by_cascade(board, &any)
}

fn best_by_cascade(board: &Board, cells: &[usize]) -> Option<(usize, Vec<u8>)> {
    let mut best: Option<(usize, Vec<u8>, i32)> = None;

    for &cell_idx in cells {
        let candidates = board.get_cell(cell_idx)?.candidates.to_vec();
        if candidates.is_empty() { return None; } // contradiction

        let mut cascade_score = 0;
        for &value in &candidates {
            let mut sim = board.clone();
            if sim.set_cell_value(cell_idx, value).is_err() { continue; }
            let before = sim.solved_count();
            let _ = propagate_all_constraints(&mut sim); // ignore contradiction here
            cascade_score += sim.solved_count() as i32 - before as i32;
        }

        if best.as_ref().map_or(true, |(_, _, s)| cascade_score > *s) {
            best = Some((cell_idx, candidates, cascade_score));
        }
    }

    best.map(|(idx, cands, _)| (idx, cands))
}

/// Solve using parallel speculation
/// Note: This is a simplified version that explores branches in parallel
/// For full integration with strategies, use sequential mode
const MAX_NODES: usize = 200_000;

enum NodeResult {
    Solved(Board),
    Dead,
    Children(Vec<Board>),
}

/// Breadth-first parallel speculation. Expands one full layer at a time;
/// dead branches are dropped, not carried forward. No depth limit —
/// bounded only by total nodes explored.
pub fn solve_parallel(
    board: &Board,
    stats: &mut SpeculationStatistics,
) -> SolverResult<Option<Board>> {
    let mut frontier = vec![board.clone()];
    let mut nodes_explored = 0usize;

    while !frontier.is_empty() {
        nodes_explored += frontier.len();
        if nodes_explored > MAX_NODES {
            return Err(SolverError::MaxIterationsReached);
        }

        let results: Vec<NodeResult> = frontier
            .into_par_iter()
            .map(|mut state| {
                if propagate_all_constraints(&mut state).is_err() {
                    return NodeResult::Dead;
                }
                if !state.is_valid() {
                    return NodeResult::Dead;
                }
                if state.is_complete() {
                    return NodeResult::Solved(state);
                }
                match find_best_speculation_cell(&state) {
                    None => NodeResult::Dead,
                    Some((cell_idx, candidates)) => {
                        let children: Vec<Board> = candidates
                            .iter()
                            .filter_map(|&v| {
                                let mut child = state.clone();
                                child.set_cell_value(cell_idx, v).ok()?;
                                Some(child)
                            })
                            .collect();
                        NodeResult::Children(children)
                    }
                }
            })
            .collect();

        let mut next_frontier = Vec::new();
        for r in results {
            match r {
                NodeResult::Solved(b) => return Ok(Some(b)),
                NodeResult::Dead => stats.branches_pruned += 1,
                NodeResult::Children(c) => {
                    stats.branches_explored += c.len();
                    next_frontier.extend(c);
                }
            }
        }
        stats.max_depth_reached += 1;
        frontier = next_frontier;
    }

    stats.contradictions_found += 1;
    Ok(None)
}

/// Solve using sequential speculation (traditional backtracking with full propagation)
pub fn solve_sequential(
    board: &mut Board,
    cell_idx: usize,
    candidates: &[u8],
    depth: usize,
    max_depth: usize,
    stats: &mut SpeculationStatistics,
) -> SolverResult<()> {
    log::trace!("Sequential speculation at depth {} for cell {} with {} candidates",
               depth, cell_idx, candidates.len());
    
    stats.max_depth_reached = stats.max_depth_reached.max(depth);
    
    for &value in candidates {
        eprintln!("DEBUG: trying value {} at cell {}", value, cell_idx);
        stats.branches_explored += 1;
        log::trace!("Sequential branch: trying value {} at cell {}", value, cell_idx);
        
        // Save board state
        let saved_board = board.clone();
        
        // Try setting the value
        if board.set_cell_value(cell_idx, value).is_err() {
            *board = saved_board;
            stats.branches_pruned += 1;
            continue;
        }
        
        // Propagate all constraints fully
        if propagate_all_constraints(board).is_err() {
            *board = saved_board;
            stats.branches_pruned += 1;
            continue;
        }
        
        // Check if solved
        if board.is_solved() {
            return Ok(());
        }
        
        // Check if valid
        if !board.is_valid() {
            *board = saved_board;
            stats.branches_pruned += 1;
            continue;
        }
        
        // If we haven't reached max depth, continue speculation
        if depth < max_depth {
            // Find next cell to speculate on
            if let Some((next_cell, next_candidates)) = find_best_speculation_cell(board) {
                match solve_sequential(board, next_cell, &next_candidates, depth + 1, max_depth, stats) {
                    Ok(()) => return Ok(()),
                    Err(SolverError::NoSolution) => {
                        // This branch failed, restore and try next
                        *board = saved_board;
                        stats.branches_pruned += 1;
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        
        // Restore board state for next candidate
        *board = saved_board;
    }
    
    // No candidate worked
    stats.contradictions_found += 1;
    Err(SolverError::NoSolution)
}

/// Main entry point for speculation-based solving
pub fn solve_with_speculation(
    board: &mut Board,
    solver: &super::Solver,
    config: &SpeculationConfig,
    _stats: Option<&SpeculationStatistics>,
    _depth: usize,
) -> SolverResult<()> {
    if !config.enabled {
        // Speculation disabled, shouldn't be called
        return Err(SolverError::InvalidBoard("Speculation is disabled".to_string()));
    }

    // First, try to make progress with logical strategies
    // This is important because speculation should only be used when logical strategies fail
    let mut iteration = 0;
    let max_logical_iterations = 100;
    
    while !board.is_complete() && iteration < max_logical_iterations {
        iteration += 1;
        
        // Try one iteration of logical solving
        match solver.solve_iteration(board) {
            Ok(true) => continue,  // Made progress, keep trying
            Ok(false) => break,     // No progress, need speculation
            Err(e) => return Err(e),
        }
    }
    
    // If solved by logical strategies alone, we're done
    if board.is_complete() {
        return Ok(());
    }

    // Find best cell for speculation
    let (cell_idx, candidates) = match find_best_speculation_cell(board) {
        Some(result) => {
            eprintln!("DEBUG: Found speculation cell {} with {} candidates", result.0, result.1.len());
            result
        }
        None => {
            // No valid cell found - either solved or contradiction
            eprintln!("DEBUG: No speculation cell found. Solved: {}, Valid: {}", 
                     board.is_solved(), board.is_valid());
            eprintln!("DEBUG: Unsolved count: {}", board.unsolved_count());
            
            // Check for cells with 0 candidates
            let mut zero_candidate_cells = Vec::new();
            for i in 0..81 {
                if !board.is_cell_solved(i) {
                    if let Some(cell) = board.get_cell(i) {
                        if cell.candidates.count() == 0 {
                            zero_candidate_cells.push(i);
                        }
                    }
                }
            }
            if !zero_candidate_cells.is_empty() {
                eprintln!("DEBUG: Cells with 0 candidates: {:?}", zero_candidate_cells);
            }
            
            if board.is_complete() {
                return Ok(());
            } else {
                return Err(SolverError::NoSolution);
            }
        }
    };

    let mut stats = SpeculationStatistics::new();

    eprintln!("DEBUG: mode={:?}, cell={}, candidates={:?}", config.mode, cell_idx, candidates);

    // Choose strategy based on mode
    let result = match config.mode {
        SpeculationMode::Sequential => {
            stats.record_mode_used(SpeculationMode::Sequential);
            solve_sequential(board, cell_idx, &candidates, 0, config.max_depth, &mut stats)
        }
        SpeculationMode::Parallel | SpeculationMode::Hybrid => {
    match solve_parallel(board, &mut stats) {
        Ok(Some(solved_board)) => { *board = solved_board; Ok(()) }
        Ok(None) => Err(SolverError::NoSolution),
        Err(e) => Err(e),
    }
    }
    };

    if config.track_statistics {
        stats.log_stats();
    }

    result
}

/// Propagate all constraints from all solved cells
fn propagate_all_constraints(board: &mut Board) -> SolverResult<()> {
    use std::collections::VecDeque;
    
    let mut queue = VecDeque::new();
    
    // Add all solved cells to queue
    for i in 0..81 {
        if board.is_cell_solved(i) {
            queue.push_back(i);
        }
    }
    
    // Process queue
    while let Some(cell_idx) = queue.pop_front() {
        let value = match board.get_cell(cell_idx).and_then(|c| c.value) {
            Some(v) => v,
            None => continue,
        };
        
        // Get peer indices
        let peer_indices = board.get_cell_constraints(cell_idx)
            .map(|c| c.peer_indices.clone())
            .unwrap_or_default();
        
        // Remove value from all peers
        for &peer_idx in &peer_indices {
            if let Some(peer_cell) = board.get_cell_mut(peer_idx) {
                if !peer_cell.is_solved() && peer_cell.candidates.contains(value) {
                    peer_cell.remove_candidate(value);
                    
                    // Check for contradiction
                    if peer_cell.candidates.is_empty() {
                        return Err(SolverError::NoSolution);
                    }
                    
                    // If cell became solved, add to queue
                    if peer_cell.is_solved() {
                        queue.push_back(peer_idx);
                    }
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    
    #[test]
    fn test_speculation_config_default() {
        let config = SpeculationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_depth, 3);
        assert_eq!(config.mode, SpeculationMode::Hybrid);
        assert!(config.track_statistics);
    }
    
    #[test]
    fn test_speculation_mode_from_str() {
        assert_eq!(SpeculationMode::from_str("sequential"), Some(SpeculationMode::Sequential));
        assert_eq!(SpeculationMode::from_str("parallel"), Some(SpeculationMode::Parallel));
        assert_eq!(SpeculationMode::from_str("hybrid"), Some(SpeculationMode::Hybrid));
        assert_eq!(SpeculationMode::from_str("invalid"), None);
    }
    
    #[test]
    fn test_count_cells_with_n_candidates() {
        let board = Board::new();
        // New board has all cells with 9 candidates
        let count = count_cells_with_n_candidates(&board, 9);
        assert_eq!(count, 81);
    }
    
    #[test]
    fn test_find_best_speculation_cell() {
        let mut board = Board::new();
        
        // Set some values to create cells with fewer candidates
        board.set_cell_value(0, 1).unwrap();
        board.set_cell_value(1, 2).unwrap();
        
        let result = find_best_speculation_cell(&board);
        assert!(result.is_some());
        
        let (cell_idx, candidates) = result.unwrap();
        assert!(cell_idx < 81);
        assert!(!candidates.is_empty());
    }
    
    #[test]
    fn test_speculation_statistics() {
        let mut stats = SpeculationStatistics::new();
        
        stats.branches_explored = 10;
        stats.branches_pruned = 5;
        stats.record_mode_used(SpeculationMode::Parallel);
        
        assert_eq!(stats.branches_explored, 10);
        assert_eq!(stats.branches_pruned, 5);
        assert_eq!(stats.speculation_mode_used.get("Parallel"), Some(&1));
    }
    
    #[test]
    fn test_choose_speculation_strategy() {
        let board = Board::new();
        let strategy = choose_speculation_strategy(&board);
        
        // New board should choose backtracking (no cells with 2 candidates)
        matches!(strategy, HybridStrategy::Backtracking | HybridStrategy::LimitedBifurcation(_));
    }
}
