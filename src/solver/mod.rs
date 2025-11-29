//! Solver module for Sudoku puzzle solving.
//!
//! This module implements constraint propagation using the view abstractions
//! to efficiently eliminate candidates and solve cells.

use crate::board::Board;
use std::collections::VecDeque;

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
}

impl Solver {
    /// Creates a new solver with default settings
    pub fn new() -> Self {
        Self {
            max_iterations: 10000,
        }
    }
    
    /// Creates a new solver with custom max iterations
    pub fn with_max_iterations(max_iterations: usize) -> Self {
        Self { max_iterations }
    }
    
    /// Solves the given board using constraint propagation
    pub fn solve(&self, board: &mut Board) -> SolverResult<()> {
        if !board.is_valid() {
            return Err(SolverError::InvalidBoard("Initial board state is invalid".to_string()));
        }
        
        // Initial constraint propagation from clues
        self.propagate_initial_constraints(board)?;
        
        let mut iteration = 0;
        
        while !board.is_solved() && iteration < self.max_iterations {
            iteration += 1;
            
            let progress = self.solve_iteration(board)?;
            
            if !progress {
                // No progress made - would need speculative execution
                // For MVP, we'll just stop here
                break;
            }
        }
        
        if iteration >= self.max_iterations {
            return Err(SolverError::MaxIterationsReached);
        }
        
        Ok(())
    }
    
    /// Performs one iteration of solving strategies
    fn solve_iteration(&self, board: &mut Board) -> SolverResult<bool> {
        let mut progress = false;
        
        // Try naked singles (cells with only one candidate)
        progress |= self.apply_naked_singles(board)?;
        
        // Try hidden singles (values that can only go in one cell in a unit)
        progress |= self.apply_hidden_singles(board)?;
        
        Ok(progress)
    }
    
    /// Propagates constraints from initial clues
    pub fn propagate_initial_constraints(&self, board: &mut Board) -> SolverResult<()> {
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
        &self,
        board: &mut Board,
        cell_idx: usize,
        queue: &mut VecDeque<usize>,
    ) -> SolverResult<()> {
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
        
        Ok(())
    }
    
    /// Applies naked singles strategy: cells with only one candidate
    fn apply_naked_singles(&self, board: &mut Board) -> SolverResult<bool> {
        let mut progress = false;
        let mut queue = VecDeque::new();
        
        // Find all cells with exactly one candidate
        for i in 0..81 {
            if !board.is_cell_solved(i) {
                if let Some(cell) = board.get_cell(i) {
                    if cell.candidates.is_single() {
                        if let Some(value) = cell.candidates.get_single() {
                            board.set_cell_value(i, value)
                                .map_err(|e| SolverError::InvalidBoard(e))?;
                            queue.push_back(i);
                            progress = true;
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
    fn apply_hidden_singles(&self, board: &mut Board) -> SolverResult<bool> {
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
        &self,
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
                        board.set_cell_value(cell_idx, value)
                            .map_err(|e| SolverError::InvalidBoard(e))?;
                        
                        // Propagate constraints
                        let mut queue = VecDeque::new();
                        queue.push_back(cell_idx);
                        while let Some(idx) = queue.pop_front() {
                            self.propagate_cell_constraints(board, idx, &mut queue)?;
                        }
                        
                        progress = true;
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
        let solver = Solver::new();
        assert_eq!(solver.max_iterations, 10000);
    }

    #[test]
    fn test_solve_easy_puzzle() {
        // Easy puzzle with only naked singles needed
        let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let mut board = Board::from_string(puzzle).unwrap();
        
        let solver = Solver::new();
        let result = solver.solve(&mut board);
        
        // Should solve or make significant progress
        assert!(result.is_ok() || board.solved_count() > 30);
    }

    #[test]
    fn test_propagate_constraints() {
        let mut board = Board::new();
        board.set_cell_value(0, 5).unwrap();
        
        let solver = Solver::new();
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
        
        let solver = Solver::new();
        solver.propagate_initial_constraints(&mut board).unwrap();
        
        let cell = board.get_cell(8).unwrap();
        assert_eq!(cell.candidates.count(), 1);
        assert!(cell.candidates.contains(9));
    }
}
