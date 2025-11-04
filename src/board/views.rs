//! View abstractions for zero-cost access to constraint groups.
//!
//! Instead of iterating over the entire board to find cells in a row, column, or box,
//! we pre-compute indices and provide lightweight view objects that hold references
//! to the relevant cells. This enables O(1) access to constraint groups.

use super::candidates::CandidateSet;

/// Represents a cell in the Sudoku board
#[derive(Debug, Clone)]
pub struct Cell {
    /// Index of the cell in the board (0-80 for 9x9)
    pub index: usize,
    /// Current value if solved, None otherwise
    pub value: Option<u8>,
    /// Set of possible candidate values
    pub candidates: CandidateSet,
}

impl Cell {
    /// Creates a new unsolved cell with all candidates
    pub fn new(index: usize) -> Self {
        Self {
            index,
            value: None,
            candidates: CandidateSet::full(),
        }
    }

    /// Creates a new cell with a given value (clue)
    pub fn with_value(index: usize, value: u8) -> Self {
        Self {
            index,
            value: Some(value),
            candidates: CandidateSet::single(value),
        }
    }

    /// Checks if the cell is solved
    pub fn is_solved(&self) -> bool {
        self.value.is_some()
    }

    /// Sets the cell value and updates candidates
    pub fn set_value(&mut self, value: u8) {
        self.value = Some(value);
        self.candidates = CandidateSet::single(value);
    }

    /// Removes a candidate from the cell
    pub fn remove_candidate(&mut self, value: u8) -> bool {
        if self.candidates.contains(value) {
            self.candidates.remove(value);
            
            // If only one candidate remains, solve the cell
            if let Some(single) = self.candidates.get_single() {
                self.value = Some(single);
            }
            
            true
        } else {
            false
        }
    }
}

/// A view into a row of cells (zero-cost abstraction)
/// Holds indices of cells in the row for direct access
#[derive(Debug, Clone)]
pub struct RowView {
    /// Index of this row (0-8 for 9x9)
    pub index: usize,
    /// Indices of cells in this row
    pub cell_indices: Vec<usize>,
}

impl RowView {
    /// Creates a new row view
    pub fn new(index: usize, cell_indices: Vec<usize>) -> Self {
        Self { index, cell_indices }
    }

    /// Returns the number of cells in this row
    pub fn len(&self) -> usize {
        self.cell_indices.len()
    }

    /// Checks if the row is empty (should never be true)
    pub fn is_empty(&self) -> bool {
        self.cell_indices.is_empty()
    }
}

/// A view into a column of cells (zero-cost abstraction)
#[derive(Debug, Clone)]
pub struct ColumnView {
    /// Index of this column (0-8 for 9x9)
    pub index: usize,
    /// Indices of cells in this column
    pub cell_indices: Vec<usize>,
}

impl ColumnView {
    /// Creates a new column view
    pub fn new(index: usize, cell_indices: Vec<usize>) -> Self {
        Self { index, cell_indices }
    }

    /// Returns the number of cells in this column
    pub fn len(&self) -> usize {
        self.cell_indices.len()
    }

    /// Checks if the column is empty (should never be true)
    pub fn is_empty(&self) -> bool {
        self.cell_indices.is_empty()
    }
}

/// A view into a 3x3 box of cells (zero-cost abstraction)
#[derive(Debug, Clone)]
pub struct BoxView {
    /// Index of this box (0-8 for 9x9)
    pub index: usize,
    /// Indices of cells in this box
    pub cell_indices: Vec<usize>,
}

impl BoxView {
    /// Creates a new box view
    pub fn new(index: usize, cell_indices: Vec<usize>) -> Self {
        Self { index, cell_indices }
    }

    /// Returns the number of cells in this box
    pub fn len(&self) -> usize {
        self.cell_indices.len()
    }

    /// Checks if the box is empty (should never be true)
    pub fn is_empty(&self) -> bool {
        self.cell_indices.is_empty()
    }
}

/// Pre-computed constraint graph for a cell
/// Provides O(1) access to all cells that constrain this cell
#[derive(Debug, Clone)]
pub struct CellConstraints {
    /// Index of the cell
    pub cell_index: usize,
    /// Index of the row this cell belongs to
    pub row_index: usize,
    /// Index of the column this cell belongs to
    pub col_index: usize,
    /// Index of the box this cell belongs to
    pub box_index: usize,
    /// Indices of all peer cells (cells that constrain this cell)
    /// This is the union of cells in the same row, column, and box (excluding self)
    pub peer_indices: Vec<usize>,
}

impl CellConstraints {
    /// Creates a new cell constraints structure
    pub fn new(
        cell_index: usize,
        row_index: usize,
        col_index: usize,
        box_index: usize,
        peer_indices: Vec<usize>,
    ) -> Self {
        Self {
            cell_index,
            row_index,
            col_index,
            box_index,
            peer_indices,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_creation() {
        let cell = Cell::new(0);
        assert_eq!(cell.index, 0);
        assert!(!cell.is_solved());
        assert_eq!(cell.candidates.count(), 9);
    }

    #[test]
    fn test_cell_with_value() {
        let cell = Cell::with_value(0, 5);
        assert_eq!(cell.index, 0);
        assert!(cell.is_solved());
        assert_eq!(cell.value, Some(5));
        assert_eq!(cell.candidates.count(), 1);
    }

    #[test]
    fn test_cell_remove_candidate() {
        let mut cell = Cell::new(0);
        assert!(cell.remove_candidate(5));
        assert_eq!(cell.candidates.count(), 8);
        assert!(!cell.candidates.contains(5));
    }

    #[test]
    fn test_cell_auto_solve() {
        let mut cell = Cell::new(0);
        // Remove all candidates except 5
        for i in 1..=9 {
            if i != 5 {
                cell.remove_candidate(i);
            }
        }
        assert!(cell.is_solved());
        assert_eq!(cell.value, Some(5));
    }

    #[test]
    fn test_row_view() {
        let indices = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        let row = RowView::new(0, indices.clone());
        assert_eq!(row.index, 0);
        assert_eq!(row.len(), 9);
        assert_eq!(row.cell_indices, indices);
    }

    #[test]
    fn test_column_view() {
        let indices = vec![0, 9, 18, 27, 36, 45, 54, 63, 72];
        let col = ColumnView::new(0, indices.clone());
        assert_eq!(col.index, 0);
        assert_eq!(col.len(), 9);
        assert_eq!(col.cell_indices, indices);
    }

    #[test]
    fn test_box_view() {
        let indices = vec![0, 1, 2, 9, 10, 11, 18, 19, 20];
        let box_view = BoxView::new(0, indices.clone());
        assert_eq!(box_view.index, 0);
        assert_eq!(box_view.len(), 9);
        assert_eq!(box_view.cell_indices, indices);
    }
}
