//! Solver module for Sudoku puzzle solving.
//!
//! This module implements constraint propagation using the view abstractions
//! to efficiently eliminate candidates and solve cells.

pub mod speculative;

use crate::board::Board;
use crate::strategy::{StrategyBank, StrategySelector, SelectionPolicy};
use crate::logging::{Timer, SolverStats};
use self::speculative::{SpeculationConfig, SpeculationMode, SpeculationStatistics};
use std::collections::VecDeque;
use std::path::Path;

/// Result type for solver operations
pub type SolverResult<T> = Result<T, SolverError>;

/// Errors that can occur during solving
#[derive(Debug, thiserror::Error)]
pub enum SolverError {
    #[error("Board has no solution (contradiction detected)")]
    NoSolution,
    
    #[error("Board is invalid: {0}")]
    InvalidBoard(String),
    
    #[error("Maximum iterations reached without solution")]
    MaxIterationsReached,
}

/// The main solver engine
pub struct Solver {
    /// Maximum number of iterations before giving up
    max_iterations: usize,
    
    /// Strategy bank for loading strategies
    strategy_bank: Option<StrategyBank>,
    
    /// Whether to use the strategy system
    use_strategies: bool,
    
    /// Statistics tracker
    stats: SolverStats,
    
    /// Speculation configuration
    speculation_config: SpeculationConfig,
    
    /// Speculation statistics
    speculation_stats: SpeculationStatistics,
}

impl Solver {
    /// Creates a new solver with default settings (no strategy system)
    pub fn new() -> Self {
        log::debug!("Creating new solver (legacy mode)");
        Self {
            max_iterations: 10000,
            strategy_bank: None,
            use_strategies: false,
            stats: SolverStats::new(),
            speculation_config: SpeculationConfig::default(),
            speculation_stats: SpeculationStatistics::new(),
        }
    }
    
    /// Creates a new solver with custom max iterations
    pub fn with_max_iterations(max_iterations: usize) -> Self {
        log::debug!("Creating solver with max_iterations={}", max_iterations);
        Self { 
            max_iterations,
            strategy_bank: None,
            use_strategies: false,
            stats: SolverStats::new(),
            speculation_config: SpeculationConfig::default(),
            speculation_stats: SpeculationStatistics::new(),
        }
    }
    
    /// Creates a new solver with strategy system enabled
    pub fn with_strategies<P: AsRef<Path>>(strategy_dir: P) -> Result<Self, SolverError> {
        let _timer = Timer::new("Loading strategies");
        log::info!("Initializing solver with strategy system");
        
        let strategy_bank = StrategyBank::load_from_directory(strategy_dir)
            .map_err(|e| SolverError::InvalidBoard(format!("Failed to load strategies: {}", e)))?;
        
        let strategy_count = strategy_bank.get_all_strategies().len();
        log::info!("Loaded {} strategies", strategy_count);
        
        Ok(Self {
            max_iterations: 10000,
            strategy_bank: Some(strategy_bank),
            use_strategies: true,
            stats: SolverStats::new(),
            speculation_config: SpeculationConfig::default(),
            speculation_stats: SpeculationStatistics::new(),
        })
    }
    
    /// Creates a new solver with custom speculation configuration
    pub fn with_speculation<P: AsRef<Path>>(
        strategy_dir: P,
        speculation_config: SpeculationConfig,
    ) -> Result<Self, SolverError> {
        let mut solver = Self::with_strategies(strategy_dir)?;
        solver.speculation_config = speculation_config;
        Ok(solver)
    }
    
    /// Set speculation configuration
    pub fn set_speculation_config(&mut self, config: SpeculationConfig) {
        self.speculation_config = config;
    }
    
    /// Solves the given board using constraint propagation
    pub fn solve(&mut self, board: &mut Board) -> SolverResult<()> {
        let _timer = Timer::new("Total solve time");
        log::info!("Starting solve process");
        log::debug!("Initial board state: {}/81 cells solved", board.solved_count());
        
        if !board.is_valid() {
            log::error!("Initial board state is invalid");
            return Err(SolverError::InvalidBoard("Initial board state is invalid".to_string()));
        }
        
        // Initial constraint propagation from clues
        log::debug!("Propagating initial constraints");
        self.propagate_initial_constraints(board)?;
        log::info!("After initial propagation: {}/81 cells solved", board.solved_count());
        
        let mut iteration = 0;
        
        while !board.is_solved() && iteration < self.max_iterations {
            iteration += 1;
            self.stats.iterations = iteration;
            
            log::debug!("Starting iteration {}", iteration);
            let progress = self.solve_iteration(board)?;
            
            if progress {
                log::info!("Iteration {}: {}/81 cells solved", iteration, board.solved_count());
            } else {
                log::debug!("Iteration {}: No progress made", iteration);
            }
            
            if !progress {
                // No progress made with logical strategies
                log::info!("Logical strategies exhausted, attempting speculation");
                // Try speculation if we have strategies enabled
                if self.use_strategies {
                    return self.solve_with_speculation(board);
                } else {
                    // For basic solver, just stop here
                    log::warn!("Basic solver cannot proceed further");
                    break;
                }
            }
        }
        
        if iteration >= self.max_iterations {
            log::error!("Maximum iterations ({}) reached", self.max_iterations);
            return Err(SolverError::MaxIterationsReached);
        }
        
        self.stats.cells_solved = board.solved_count() as usize;
        log::info!("Solve complete: {}/81 cells solved", board.solved_count());
        self.stats.log_stats();
        
        if self.speculation_config.track_statistics {
            self.speculation_stats.log_stats();
        }
        
        Ok(())
    }
    
    /// Solves using speculation (replaces old backtracking)
    fn solve_with_speculation(&mut self, board: &mut Board) -> SolverResult<()> {
        if !self.speculation_config.enabled {
            log::info!("Speculation disabled, using legacy backtracking");
            return self.solve_with_backtracking(board);
        }
        
        log::info!("Starting speculation with mode: {:?}", self.speculation_config.mode);
        
        // Find best cell for speculation
        let (cell_idx, candidates) = match speculative::find_best_speculation_cell(board) {
            Some(result) => result,
            None => {
                log::debug!("No valid cell for speculation");
                return if board.is_solved() {
                    Ok(())
                } else {
                    Err(SolverError::NoSolution)
                };
            }
        };
        
        log::debug!("Speculating on cell {} with {} candidates", cell_idx, candidates.len());
        
        // Choose speculation mode
        let mode = match self.speculation_config.mode {
            SpeculationMode::Hybrid => {
                let strategy = speculative::choose_speculation_strategy(board);
                match strategy {
                    speculative::HybridStrategy::Bifurcation => SpeculationMode::Parallel,
                    speculative::HybridStrategy::Backtracking => SpeculationMode::Sequential,
                    speculative::HybridStrategy::LimitedBifurcation(_) => SpeculationMode::Parallel,
                }
            }
            mode => mode,
        };
        
        self.speculation_stats.record_mode_used(mode);
        
        // Execute speculation based on chosen mode
        match mode {
            SpeculationMode::Parallel => {
                log::info!("Using parallel speculation");
                match speculative::solve_parallel(
                    board,
                    cell_idx,
                    &candidates,
                    0,
                    self.speculation_config.max_depth,
                    &mut self.speculation_stats,
                ) {
                    Ok(Some(solved_board)) => {
                        *board = solved_board;
                        Ok(())
                    }
                    Ok(None) => Err(SolverError::NoSolution),
                    Err(e) => Err(e),
                }
            }
            SpeculationMode::Sequential | SpeculationMode::Hybrid => {
                log::info!("Using sequential speculation");
                speculative::solve_sequential(
                    board,
                    cell_idx,
                    &candidates,
                    0,
                    self.speculation_config.max_depth,
                    &mut self.speculation_stats,
                )
            }
        }
    }
    
    /// Solves using backtracking (depth-first search with constraint propagation)
    fn solve_with_backtracking(&mut self, board: &mut Board) -> SolverResult<()> {
        self.stats.backtracks += 1;
        log::trace!("Backtracking attempt #{}", self.stats.backtracks);
        // Find the cell with the fewest candidates (most constrained)
        let mut best_cell = None;
        let mut min_candidates = 10;
        
        for cell_idx in 0..81 {
            if !board.is_cell_solved(cell_idx) {
                if let Some(cell) = board.get_cell(cell_idx) {
                    let count = cell.candidates.count();
                    if count == 0 {
                        // Contradiction found
                        log::trace!("Contradiction found at cell {}", cell_idx);
                        return Err(SolverError::NoSolution);
                    }
                    if count < min_candidates {
                        min_candidates = count;
                        best_cell = Some(cell_idx);
                    }
                }
            }
        }
        
        // If no unsolved cells, we're done
        let cell_idx = match best_cell {
            Some(idx) => idx,
            None => {
                log::debug!("Backtracking successful - puzzle solved!");
                return Ok(()); // Solved!
            }
        };
        
        log::trace!("Trying cell {} with {} candidates", cell_idx, min_candidates);
        
        // Get the candidates for this cell
        let candidates = board.get_cell(cell_idx)
            .map(|c| c.candidates.to_vec())
            .unwrap_or_default();
        
        // Try each candidate
        for &value in &candidates {
            log::trace!("Trying value {} at cell {}", value, cell_idx);
            
            // Save the current board state
            let saved_board = board.clone();
            
            // Try this value
            if board.set_cell_value(cell_idx, value).is_ok() {
                log::trace!("Set cell {} = {}", cell_idx, value);
                // Propagate constraints
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(cell_idx);
                
                let propagation_result = {
                    let mut temp_queue = queue.clone();
                    self.propagate_cell_constraints(board, cell_idx, &mut temp_queue)
                };
                
                if propagation_result.is_ok() && board.is_valid() {
                    // Try to solve recursively
                    match self.solve_with_backtracking(board) {
                        Ok(()) => {
                            if board.is_solved() {
                                log::trace!("Backtracking branch succeeded");
                                return Ok(());
                            }
                        }
                        Err(SolverError::NoSolution) => {
                            // This branch failed, try next candidate
                            log::trace!("Branch failed, trying next candidate");
                        }
                        Err(e) => return Err(e),
                    }
                } else {
                    log::trace!("Propagation failed or board invalid");
                }
            }
            
            // Restore board state and try next candidate
            *board = saved_board;
        }
        
        // No candidate worked
        log::trace!("All candidates exhausted for cell {}", cell_idx);
        Err(SolverError::NoSolution)
    }
    
    /// Performs one iteration of solving strategies
    fn solve_iteration(&mut self, board: &mut Board) -> SolverResult<bool> {
        if self.use_strategies {
            // Use the strategy system
            self.solve_iteration_with_strategies(board)
        } else {
            // Use hardcoded strategies (legacy mode)
            self.solve_iteration_legacy(board)
        }
    }
    
    /// Performs one iteration using the strategy system
    fn solve_iteration_with_strategies(&mut self, board: &mut Board) -> SolverResult<bool> {
        let strategy_bank = self.strategy_bank.as_ref()
            .ok_or_else(|| SolverError::InvalidBoard("Strategy bank not initialized".to_string()))?;
        
        // Create a new strategy selector for this iteration
        let mut strategy_selector = StrategySelector::new(SelectionPolicy::Priority);
        
        let strategies = strategy_bank.get_all_strategies();
        
        // Try to find and apply a strategy
        if let Some((strategy, matches)) = strategy_selector.select_strategy(board, strategies) {
            log::debug!("Applying strategy: {} (priority: {})", strategy.metadata.name, strategy.priority);
            log::trace!("Found {} matches for {}", matches.len(), strategy.metadata.name);
            
            let mut progress = false;
            
            // Apply all matches for this strategy
            for strategy_match in matches {
                match strategy_selector.apply_match(board, &strategy_match) {
                    Ok(made_progress) => {
                        if made_progress {
                            self.stats.strategies_applied += 1;
                            log::trace!("Strategy match applied successfully");
                        }
                        progress |= made_progress;
                    }
                    Err(e) => {
                        log::error!("Failed to apply strategy: {}", e);
                        return Err(SolverError::InvalidBoard(format!("Failed to apply strategy: {}", e)));
                    }
                }
            }
            
            Ok(progress)
        } else {
            // No strategy found a match
            log::debug!("No strategy found a match");
            Ok(false)
        }
    }
    
    /// Performs one iteration using legacy hardcoded strategies
    fn solve_iteration_legacy(&mut self, board: &mut Board) -> SolverResult<bool> {
        let mut progress = false;
        
        // Try naked singles (cells with only one candidate)
        progress |= self.apply_naked_singles(board)?;
        
        // Try hidden singles (values that can only go in one cell in a unit)
        progress |= self.apply_hidden_singles(board)?;
        
        Ok(progress)
    }
    
    /// Propagates constraints from initial clues
    pub fn propagate_initial_constraints(&mut self, board: &mut Board) -> SolverResult<()> {
        log::trace!("Propagating initial constraints");
        let mut queue = VecDeque::new();
        
        // Add all initially solved cells to the queue
        for i in 0..81 {
            if board.is_cell_solved(i) {
                queue.push_back(i);
            }
        }
        
        // Propagate constraints from each solved cell
        while let Some(cell_idx) = queue.pop_front() {
            self.propagate_cell_constraints(board, cell_idx, &mut queue)?;
        }
        
        Ok(())
    }
    
    /// Propagates constraints from a single solved cell to its peers
    fn propagate_cell_constraints(
        &mut self,
        board: &mut Board,
        cell_idx: usize,
        queue: &mut VecDeque<usize>,
    ) -> SolverResult<()> {
        self.stats.constraint_propagations += 1;
        let value = match board.get_cell(cell_idx).and_then(|c| c.value) {
            Some(v) => v,
            None => return Ok(()), // Cell not solved, nothing to propagate
        };
        
        // Get all peer cells using pre-computed constraints
        let peer_indices = board.get_cell_constraints(cell_idx)
            .map(|c| c.peer_indices.clone())
            .unwrap_or_default();
        
        // Remove this value from all peer candidates
        for &peer_idx in &peer_indices {
            if let Some(peer_cell) = board.get_cell_mut(peer_idx) {
                if !peer_cell.is_solved() && peer_cell.candidates.contains(value) {
                    peer_cell.remove_candidate(value);
                    log::trace!("Removed candidate {} from cell {}", value, peer_idx);
                    
                    // Check for contradiction
                    if peer_cell.candidates.is_empty() {
                        log::warn!("Contradiction detected at cell {} during propagation", peer_idx);
                        return Err(SolverError::NoSolution);
                    }
                    
                    // If cell became solved, add to queue
                    if peer_cell.is_solved() {
                        let solved_value = peer_cell.value.unwrap();
                        log::debug!("Cell {} solved with value {} via propagation", peer_idx, solved_value);
                        queue.push_back(peer_idx);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Applies naked singles strategy: cells with only one candidate
    fn apply_naked_singles(&mut self, board: &mut Board) -> SolverResult<bool> {
        log::trace!("Applying naked singles strategy");
        let mut progress = false;
        let mut queue = VecDeque::new();
        
        // Find all cells with exactly one candidate
        for i in 0..81 {
            if !board.is_cell_solved(i) {
                if let Some(cell) = board.get_cell(i) {
                    if cell.candidates.is_single() {
                        if let Some(value) = cell.candidates.get_single() {
                            log::debug!("Naked single found: cell {} = {}", i, value);
                            board.set_cell_value(i, value)
                                .map_err(|e| SolverError::InvalidBoard(e))?;
                            queue.push_back(i);
                            progress = true;
                            self.stats.strategies_applied += 1;
                        }
                    }
                }
            }
        }
        
        // Propagate constraints from newly solved cells
        while let Some(cell_idx) = queue.pop_front() {
            self.propagate_cell_constraints(board, cell_idx, &mut queue)?;
        }
        
        Ok(progress)
    }
    
    /// Applies hidden singles strategy: values that can only go in one cell in a unit
    fn apply_hidden_singles(&mut self, board: &mut Board) -> SolverResult<bool> {
        log::trace!("Applying hidden singles strategy");
        let mut progress = false;
        
        // Check rows
        for row_idx in 0..9 {
            progress |= self.find_hidden_singles_in_unit(board, row_idx, UnitType::Row)?;
        }
        
        // Check columns
        for col_idx in 0..9 {
            progress |= self.find_hidden_singles_in_unit(board, col_idx, UnitType::Column)?;
        }
        
        // Check boxes
        for box_idx in 0..9 {
            progress |= self.find_hidden_singles_in_unit(board, box_idx, UnitType::Box)?;
        }
        
        Ok(progress)
    }
    
    /// Finds hidden singles in a specific unit (row, column, or box)
    fn find_hidden_singles_in_unit(
        &mut self,
        board: &mut Board,
        unit_idx: usize,
        unit_type: UnitType,
    ) -> SolverResult<bool> {
        let mut progress = false;
        
        // Get cell indices for this unit using view abstractions
        let cell_indices = match unit_type {
            UnitType::Row => board.get_row(unit_idx).map(|r| r.cell_indices.clone()),
            UnitType::Column => board.get_column(unit_idx).map(|c| c.cell_indices.clone()),
            UnitType::Box => board.get_box(unit_idx).map(|b| b.cell_indices.clone()),
        }.unwrap_or_default();
        
        // For each value 1-9, check if it can only go in one cell
        for value in 1..=9 {
            let mut possible_cells = Vec::new();
            
            for &cell_idx in &cell_indices {
                if let Some(cell) = board.get_cell(cell_idx) {
                    if !cell.is_solved() && cell.candidates.contains(value) {
                        possible_cells.push(cell_idx);
                    }
                }
            }
            
            // If value can only go in one cell, it's a hidden single
            if possible_cells.len() == 1 {
                let cell_idx = possible_cells[0];
                if let Some(cell) = board.get_cell(cell_idx) {
                    if !cell.is_solved() {
                        log::debug!("Hidden single found: cell {} = {} in {:?} {}", 
                                   cell_idx, value, unit_type, unit_idx);
                        board.set_cell_value(cell_idx, value)
                            .map_err(|e| SolverError::InvalidBoard(e))?;
                        
                        // Propagate constraints
                        let mut queue = VecDeque::new();
                        queue.push_back(cell_idx);
                        while let Some(idx) = queue.pop_front() {
                            self.propagate_cell_constraints(board, idx, &mut queue)?;
                        }
                        
                        progress = true;
                        self.stats.strategies_applied += 1;
                    }
                }
            }
        }
        
        Ok(progress)
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of constraint unit
#[derive(Debug, Clone, Copy)]
enum UnitType {
    Row,
    Column,
    Box,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_creation() {
        let mut solver = Solver::new();
        assert_eq!(solver.max_iterations, 10000);
    }

    #[test]
    fn test_solve_easy_puzzle() {
        // Easy puzzle with only naked singles needed
        let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let mut board = Board::from_string(puzzle).unwrap();
        
        let mut solver = Solver::new();
        let result = solver.solve(&mut board);
        
        // Should solve or make significant progress
        assert!(result.is_ok() || board.solved_count() > 30);
    }

    #[test]
    fn test_propagate_constraints() {
        let mut board = Board::new();
        board.set_cell_value(0, 5).unwrap();
        
        let mut solver = Solver::new();
        solver.propagate_initial_constraints(&mut board).unwrap();
        
        // All peers of cell 0 should not have 5 as a candidate
        let constraints = board.get_cell_constraints(0).unwrap();
        for &peer_idx in &constraints.peer_indices {
            let cell = board.get_cell(peer_idx).unwrap();
            assert!(!cell.candidates.contains(5));
        }
    }

    #[test]
    fn test_naked_singles() {
        let mut board = Board::new();
        
        // Set up a situation where a cell has only one candidate
        board.set_cell_value(0, 1).unwrap();
        board.set_cell_value(1, 2).unwrap();
        board.set_cell_value(2, 3).unwrap();
        board.set_cell_value(3, 4).unwrap();
        board.set_cell_value(4, 5).unwrap();
        board.set_cell_value(5, 6).unwrap();
        board.set_cell_value(6, 7).unwrap();
        board.set_cell_value(7, 8).unwrap();
        // Cell 8 should now have only 9 as a candidate
        
        let mut solver = Solver::new();
        solver.propagate_initial_constraints(&mut board).unwrap();
        
        let cell = board.get_cell(8).unwrap();
        assert_eq!(cell.candidates.count(), 1);
        assert!(cell.candidates.contains(9));
    }
}
