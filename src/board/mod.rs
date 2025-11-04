//! Board module for Sudoku puzzle representation.
//!
//! This module provides the core Board structure with pre-computed view abstractions
//! for efficient constraint access. The board maintains cells, their candidates,
//! and pre-computed indices for rows, columns, and boxes.

pub mod candidates;
pub mod views;

use views::{Cell, RowView, ColumnView, BoxView, CellConstraints};
use std::fmt;

/// The main Sudoku board structure
/// For MVP, this is hardcoded to 9x9 with 3x3 boxes
#[derive(Debug, Clone)]
pub struct Board {
    /// All cells in the board (81 cells for 9x9)
    cells: Vec<Cell>,
    
    /// Pre-computed row views (9 rows)
    rows: Vec<RowView>,
    
    /// Pre-computed column views (9 columns)
    columns: Vec<ColumnView>,
    
    /// Pre-computed box views (9 boxes)
    boxes: Vec<BoxView>,
    
    /// Pre-computed constraints for each cell
    cell_constraints: Vec<CellConstraints>,
    
    /// Bitmask tracking which cells are solved (for quick checks)
    solved_mask: u128, // 128 bits is enough for 81 cells
}

impl Board {
    /// Creates a new empty 9x9 Sudoku board
    pub fn new() -> Self {
        let mut cells = Vec::with_capacity(81);
        for i in 0..81 {
            cells.push(Cell::new(i));
        }
        
        let rows = Self::compute_rows();
        let columns = Self::compute_columns();
        let boxes = Self::compute_boxes();
        let cell_constraints = Self::compute_cell_constraints(&rows, &columns, &boxes);
        
        Self {
            cells,
            rows,
            columns,
            boxes,
            cell_constraints,
            solved_mask: 0,
        }
    }
    
    /// Creates a board from a string representation
    /// Format: 81 characters, '0' or '.' for empty cells, '1'-'9' for clues
    /// Example: "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
    pub fn from_string(s: &str) -> Result<Self, String> {
        let chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
        
        if chars.len() != 81 {
            return Err(format!("Expected 81 characters, got {}", chars.len()));
        }
        
        let mut board = Self::new();
        
        for (i, &ch) in chars.iter().enumerate() {
            match ch {
                '0' | '.' => {
                    // Empty cell, already initialized
                }
                '1'..='9' => {
                    let value = ch.to_digit(10).unwrap() as u8;
                    board.set_cell_value(i, value)?;
                }
                _ => {
                    return Err(format!("Invalid character '{}' at position {}", ch, i));
                }
            }
        }
        
        Ok(board)
    }
    
    /// Pre-computes row views (indices of cells in each row)
    fn compute_rows() -> Vec<RowView> {
        let mut rows = Vec::with_capacity(9);
        for row in 0..9 {
            let mut indices = Vec::with_capacity(9);
            for col in 0..9 {
                indices.push(row * 9 + col);
            }
            rows.push(RowView::new(row, indices));
        }
        rows
    }
    
    /// Pre-computes column views (indices of cells in each column)
    fn compute_columns() -> Vec<ColumnView> {
        let mut columns = Vec::with_capacity(9);
        for col in 0..9 {
            let mut indices = Vec::with_capacity(9);
            for row in 0..9 {
                indices.push(row * 9 + col);
            }
            columns.push(ColumnView::new(col, indices));
        }
        columns
    }
    
    /// Pre-computes box views (indices of cells in each 3x3 box)
    fn compute_boxes() -> Vec<BoxView> {
        let mut boxes = Vec::with_capacity(9);
        for box_idx in 0..9 {
            let box_row = box_idx / 3;
            let box_col = box_idx % 3;
            let mut indices = Vec::with_capacity(9);
            
            for r in 0..3 {
                for c in 0..3 {
                    let row = box_row * 3 + r;
                    let col = box_col * 3 + c;
                    indices.push(row * 9 + col);
                }
            }
            boxes.push(BoxView::new(box_idx, indices));
        }
        boxes
    }
    
    /// Pre-computes constraint relationships for each cell
    fn compute_cell_constraints(
        rows: &[RowView],
        columns: &[ColumnView],
        boxes: &[BoxView],
    ) -> Vec<CellConstraints> {
        let mut constraints = Vec::with_capacity(81);
        
        for cell_idx in 0..81 {
            let row_idx = cell_idx / 9;
            let col_idx = cell_idx % 9;
            let box_idx = (row_idx / 3) * 3 + (col_idx / 3);
            
            // Collect all peer indices (cells that constrain this cell)
            let mut peer_indices = std::collections::HashSet::new();
            
            // Add row peers
            for &idx in &rows[row_idx].cell_indices {
                if idx != cell_idx {
                    peer_indices.insert(idx);
                }
            }
            
            // Add column peers
            for &idx in &columns[col_idx].cell_indices {
                if idx != cell_idx {
                    peer_indices.insert(idx);
                }
            }
            
            // Add box peers
            for &idx in &boxes[box_idx].cell_indices {
                if idx != cell_idx {
                    peer_indices.insert(idx);
                }
            }
            
            let peer_vec: Vec<usize> = peer_indices.into_iter().collect();
            
            constraints.push(CellConstraints::new(
                cell_idx,
                row_idx,
                col_idx,
                box_idx,
                peer_vec,
            ));
        }
        
        constraints
    }
    
    /// Sets a cell value (for initial clues or solving)
    pub fn set_cell_value(&mut self, index: usize, value: u8) -> Result<(), String> {
        if index >= 81 {
            return Err(format!("Cell index {} out of bounds", index));
        }
        
        if !(1..=9).contains(&value) {
            return Err(format!("Value {} must be between 1 and 9", value));
        }
        
        self.cells[index].set_value(value);
        self.solved_mask |= 1 << index;
        
        Ok(())
    }
    
    /// Gets a reference to a cell
    pub fn get_cell(&self, index: usize) -> Option<&Cell> {
        self.cells.get(index)
    }
    
    /// Gets a mutable reference to a cell
    pub fn get_cell_mut(&mut self, index: usize) -> Option<&mut Cell> {
        self.cells.get_mut(index)
    }
    
    /// Gets the row view for a given row index
    pub fn get_row(&self, index: usize) -> Option<&RowView> {
        self.rows.get(index)
    }
    
    /// Gets the column view for a given column index
    pub fn get_column(&self, index: usize) -> Option<&ColumnView> {
        self.columns.get(index)
    }
    
    /// Gets the box view for a given box index
    pub fn get_box(&self, index: usize) -> Option<&BoxView> {
        self.boxes.get(index)
    }
    
    /// Gets the constraints for a given cell
    pub fn get_cell_constraints(&self, index: usize) -> Option<&CellConstraints> {
        self.cell_constraints.get(index)
    }
    
    /// Checks if a cell is solved
    pub fn is_cell_solved(&self, index: usize) -> bool {
        (self.solved_mask & (1 << index)) != 0
    }
    
    /// Checks if the entire board is solved
    pub fn is_solved(&self) -> bool {
        self.solved_mask == (1u128 << 81) - 1
    }
    
    /// Returns the number of solved cells
    pub fn solved_count(&self) -> u32 {
        self.solved_mask.count_ones()
    }
    
    /// Returns the number of unsolved cells
    pub fn unsolved_count(&self) -> u32 {
        81 - self.solved_count()
    }
    
    /// Validates the current board state (checks for contradictions)
    pub fn is_valid(&self) -> bool {
        // Check all cells have at least one candidate
        for cell in &self.cells {
            if cell.candidates.is_empty() {
                return false;
            }
        }
        
        // Check no duplicate values in rows, columns, boxes
        for row in &self.rows {
            if !self.is_unit_valid(&row.cell_indices) {
                return false;
            }
        }
        
        for col in &self.columns {
            if !self.is_unit_valid(&col.cell_indices) {
                return false;
            }
        }
        
        for box_view in &self.boxes {
            if !self.is_unit_valid(&box_view.cell_indices) {
                return false;
            }
        }
        
        true
    }
    
    /// Checks if a unit (row, column, or box) has no duplicate values
    fn is_unit_valid(&self, indices: &[usize]) -> bool {
        let mut seen = [false; 10]; // Index 0 unused, 1-9 for values
        
        for &idx in indices {
            if let Some(value) = self.cells[idx].value {
                if seen[value as usize] {
                    return false; // Duplicate found
                }
                seen[value as usize] = true;
            }
        }
        
        true
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..9 {
            if row % 3 == 0 && row != 0 {
                writeln!(f, "------+-------+------")?;
            }
            
            for col in 0..9 {
                if col % 3 == 0 && col != 0 {
                    write!(f, "| ")?;
                }
                
                let idx = row * 9 + col;
                let cell = &self.cells[idx];
                
                if let Some(value) = cell.value {
                    write!(f, "{} ", value)?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_creation() {
        let board = Board::new();
        assert_eq!(board.cells.len(), 81);
        assert_eq!(board.rows.len(), 9);
        assert_eq!(board.columns.len(), 9);
        assert_eq!(board.boxes.len(), 9);
        assert_eq!(board.solved_count(), 0);
    }

    #[test]
    fn test_board_from_string() {
        let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let board = Board::from_string(puzzle).unwrap();
        
        // Check some known values
        assert_eq!(board.get_cell(0).unwrap().value, Some(5));
        assert_eq!(board.get_cell(1).unwrap().value, Some(3));
        assert_eq!(board.get_cell(2).unwrap().value, None);
    }

    #[test]
    fn test_set_cell_value() {
        let mut board = Board::new();
        board.set_cell_value(0, 5).unwrap();
        
        assert_eq!(board.get_cell(0).unwrap().value, Some(5));
        assert!(board.is_cell_solved(0));
        assert_eq!(board.solved_count(), 1);
    }

    #[test]
    fn test_row_view() {
        let board = Board::new();
        let row = board.get_row(0).unwrap();
        
        assert_eq!(row.cell_indices.len(), 9);
        assert_eq!(row.cell_indices[0], 0);
        assert_eq!(row.cell_indices[8], 8);
    }

    #[test]
    fn test_column_view() {
        let board = Board::new();
        let col = board.get_column(0).unwrap();
        
        assert_eq!(col.cell_indices.len(), 9);
        assert_eq!(col.cell_indices[0], 0);
        assert_eq!(col.cell_indices[8], 72);
    }

    #[test]
    fn test_box_view() {
        let board = Board::new();
        let box_view = board.get_box(0).unwrap();
        
        assert_eq!(box_view.cell_indices.len(), 9);
        assert_eq!(box_view.cell_indices[0], 0);
        assert_eq!(box_view.cell_indices[8], 20);
    }

    #[test]
    fn test_cell_constraints() {
        let board = Board::new();
        let constraints = board.get_cell_constraints(0).unwrap();
        
        // Cell 0 should have 20 peers (8 in row + 8 in column + 4 in box)
        assert_eq!(constraints.peer_indices.len(), 20);
        assert_eq!(constraints.row_index, 0);
        assert_eq!(constraints.col_index, 0);
        assert_eq!(constraints.box_index, 0);
    }

    #[test]
    fn test_board_validation() {
        let mut board = Board::new();
        assert!(board.is_valid());
        
        // Set two cells in the same row to the same value
        board.set_cell_value(0, 5).unwrap();
        board.set_cell_value(1, 5).unwrap();
        
        assert!(!board.is_valid());
    }
}
