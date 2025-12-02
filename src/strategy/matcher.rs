//! Pattern matching for Sudoku strategies.
//!
//! This module provides the PatternMatcher trait and concrete implementations
//! for various Sudoku solving strategies. Matchers use the board's view
//! abstractions for efficient pattern detection.

use super::types::{Strategy, StrategyMatch, MatchContext, UnitType};
use crate::board::Board;

/// Trait for pattern matching strategies
pub trait PatternMatcher {
    /// Finds all matches for this pattern in the given board
    fn find_matches(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch>;
}

/// Matcher for naked singles (cells with only one candidate)
pub struct NakedSingleMatcher;

impl NakedSingleMatcher {
    pub fn new() -> Self {
        Self
    }
}

impl PatternMatcher for NakedSingleMatcher {
    fn find_matches(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // Check all unsolved cells
        for cell_idx in 0..81 {
            if board.is_cell_solved(cell_idx) {
                continue;
            }
            
            if let Some(cell) = board.get_cell(cell_idx) {
                // Check if cell has exactly one candidate
                if cell.candidates.is_single() {
                    if let Some(value) = cell.candidates.get_single() {
                        let context = MatchContext::set_value(cell_idx, value);
                        
                        matches.push(StrategyMatch::new(
                            strategy.metadata.name.clone(),
                            vec![cell_idx],
                            None,
                            None,
                            vec![value],
                            context,
                        ));
                    }
                }
            }
        }
        
        matches
    }
}

impl Default for NakedSingleMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Matcher for hidden singles (values that can only go in one cell in a unit)
pub struct HiddenSingleMatcher;

impl HiddenSingleMatcher {
    pub fn new() -> Self {
        Self
    }
    
    /// Finds hidden singles in a specific unit
    fn find_in_unit(
        &self,
        board: &Board,
        strategy: &Strategy,
        unit_type: UnitType,
        unit_idx: usize,
        cell_indices: &[usize],
    ) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // For each value 1-9, check if it can only go in one cell
        for value in 1..=9 {
            let mut possible_cells = Vec::new();
            
            for &cell_idx in cell_indices {
                if let Some(cell) = board.get_cell(cell_idx) {
                    if !cell.is_solved() && cell.candidates.contains(value) {
                        possible_cells.push(cell_idx);
                    }
                }
            }
            
            // If value can only go in one cell, it's a hidden single
            if possible_cells.len() == 1 {
                let cell_idx = possible_cells[0];
                let context = MatchContext::set_value(cell_idx, value);
                
                matches.push(StrategyMatch::new(
                    strategy.metadata.name.clone(),
                    vec![cell_idx],
                    Some(unit_type),
                    Some(unit_idx),
                    vec![value],
                    context,
                ));
            }
        }
        
        matches
    }
}

impl PatternMatcher for HiddenSingleMatcher {
    fn find_matches(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // Check all rows
        for row_idx in 0..9 {
            if let Some(row) = board.get_row(row_idx) {
                matches.extend(self.find_in_unit(
                    board,
                    strategy,
                    UnitType::Row,
                    row_idx,
                    &row.cell_indices,
                ));
            }
        }
        
        // Check all columns
        for col_idx in 0..9 {
            if let Some(col) = board.get_column(col_idx) {
                matches.extend(self.find_in_unit(
                    board,
                    strategy,
                    UnitType::Column,
                    col_idx,
                    &col.cell_indices,
                ));
            }
        }
        
        // Check all boxes
        for box_idx in 0..9 {
            if let Some(box_view) = board.get_box(box_idx) {
                matches.extend(self.find_in_unit(
                    board,
                    strategy,
                    UnitType::Box,
                    box_idx,
                    &box_view.cell_indices,
                ));
            }
        }
        
        matches
    }
}

impl Default for HiddenSingleMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Matcher for naked pairs (two cells with the same two candidates)
pub struct NakedPairMatcher;

impl NakedPairMatcher {
    pub fn new() -> Self {
        Self
    }
    
    /// Finds naked pairs in a specific unit
    fn find_in_unit(
        &self,
        board: &Board,
        strategy: &Strategy,
        unit_type: UnitType,
        unit_idx: usize,
        cell_indices: &[usize],
    ) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // Find all cells with exactly 2 candidates
        let mut two_candidate_cells = Vec::new();
        for &cell_idx in cell_indices {
            if let Some(cell) = board.get_cell(cell_idx) {
                if !cell.is_solved() && cell.candidates.count() == 2 {
                    two_candidate_cells.push(cell_idx);
                }
            }
        }
        
        // Check all pairs of cells
        for i in 0..two_candidate_cells.len() {
            for j in (i + 1)..two_candidate_cells.len() {
                let cell1_idx = two_candidate_cells[i];
                let cell2_idx = two_candidate_cells[j];
                
                if let (Some(cell1), Some(cell2)) = (board.get_cell(cell1_idx), board.get_cell(cell2_idx)) {
                    // Check if they have the same candidates
                    if cell1.candidates == cell2.candidates {
                        // Found a naked pair!
                        let candidates: Vec<u8> = cell1.candidates.to_vec();
                        
                        // Find cells to eliminate from (other cells in unit)
                        let mut elimination_targets = Vec::new();
                        for &cell_idx in cell_indices {
                            if cell_idx != cell1_idx && cell_idx != cell2_idx {
                                if let Some(cell) = board.get_cell(cell_idx) {
                                    if !cell.is_solved() {
                                        // Check if this cell has any of the pair candidates
                                        let has_candidates = candidates.iter()
                                            .any(|&c| cell.candidates.contains(c));
                                        if has_candidates {
                                            elimination_targets.push(cell_idx);
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Only create a match if there are cells to eliminate from
                        if !elimination_targets.is_empty() {
                            let context = MatchContext::elimination(
                                elimination_targets,
                                candidates.clone(),
                            );
                            
                            matches.push(StrategyMatch::new(
                                strategy.metadata.name.clone(),
                                vec![cell1_idx, cell2_idx],
                                Some(unit_type),
                                Some(unit_idx),
                                candidates,
                                context,
                            ));
                        }
                    }
                }
            }
        }
        
        matches
    }
}

impl PatternMatcher for NakedPairMatcher {
    fn find_matches(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // Check all rows
        for row_idx in 0..9 {
            if let Some(row) = board.get_row(row_idx) {
                matches.extend(self.find_in_unit(
                    board,
                    strategy,
                    UnitType::Row,
                    row_idx,
                    &row.cell_indices,
                ));
            }
        }
        
        // Check all columns
        for col_idx in 0..9 {
            if let Some(col) = board.get_column(col_idx) {
                matches.extend(self.find_in_unit(
                    board,
                    strategy,
                    UnitType::Column,
                    col_idx,
                    &col.cell_indices,
                ));
            }
        }
        
        // Check all boxes
        for box_idx in 0..9 {
            if let Some(box_view) = board.get_box(box_idx) {
                matches.extend(self.find_in_unit(
                    board,
                    strategy,
                    UnitType::Box,
                    box_idx,
                    &box_view.cell_indices,
                ));
            }
        }
        
        matches
    }
}

impl Default for NakedPairMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Matcher for pointing pairs (candidates in a box pointing to a row/column)
pub struct PointingPairMatcher;

impl PointingPairMatcher {
    pub fn new() -> Self {
        Self
    }
}

impl PatternMatcher for PointingPairMatcher {
    fn find_matches(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // For each box
        for box_idx in 0..9 {
            if let Some(box_view) = board.get_box(box_idx) {
                // For each value
                for value in 1..=9 {
                    // Find cells in this box that can contain this value
                    let mut cells_with_value = Vec::new();
                    for &cell_idx in &box_view.cell_indices {
                        if let Some(cell) = board.get_cell(cell_idx) {
                            if !cell.is_solved() && cell.candidates.contains(value) {
                                cells_with_value.push(cell_idx);
                            }
                        }
                    }
                    
                    if cells_with_value.len() >= 2 && cells_with_value.len() <= 3 {
                        // Check if all cells are in the same row
                        let rows: Vec<usize> = cells_with_value.iter()
                            .map(|&idx| idx / 9)
                            .collect();
                        let all_same_row = rows.iter().all(|&r| r == rows[0]);
                        
                        if all_same_row {
                            let row_idx = rows[0];
                            // Find cells in the same row but different box
                            let mut elimination_targets = Vec::new();
                            if let Some(row) = board.get_row(row_idx) {
                                for &cell_idx in &row.cell_indices {
                                    // Check if cell is in a different box
                                    let cell_box = (cell_idx / 9 / 3) * 3 + (cell_idx % 9 / 3);
                                    if cell_box != box_idx {
                                        if let Some(cell) = board.get_cell(cell_idx) {
                                            if !cell.is_solved() && cell.candidates.contains(value) {
                                                elimination_targets.push(cell_idx);
                                            }
                                        }
                                    }
                                }
                            }
                            
                            if !elimination_targets.is_empty() {
                                let context = MatchContext::elimination(
                                    elimination_targets,
                                    vec![value],
                                );
                                
                                matches.push(StrategyMatch::new(
                                    strategy.metadata.name.clone(),
                                    cells_with_value.clone(),
                                    Some(UnitType::Box),
                                    Some(box_idx),
                                    vec![value],
                                    context,
                                ));
                            }
                        }
                        
                        // Check if all cells are in the same column
                        let cols: Vec<usize> = cells_with_value.iter()
                            .map(|&idx| idx % 9)
                            .collect();
                        let all_same_col = cols.iter().all(|&c| c == cols[0]);
                        
                        if all_same_col {
                            let col_idx = cols[0];
                            // Find cells in the same column but different box
                            let mut elimination_targets = Vec::new();
                            if let Some(col) = board.get_column(col_idx) {
                                for &cell_idx in &col.cell_indices {
                                    // Check if cell is in a different box
                                    let cell_box = (cell_idx / 9 / 3) * 3 + (cell_idx % 9 / 3);
                                    if cell_box != box_idx {
                                        if let Some(cell) = board.get_cell(cell_idx) {
                                            if !cell.is_solved() && cell.candidates.contains(value) {
                                                elimination_targets.push(cell_idx);
                                            }
                                        }
                                    }
                                }
                            }
                            
                            if !elimination_targets.is_empty() {
                                let context = MatchContext::elimination(
                                    elimination_targets,
                                    vec![value],
                                );
                                
                                matches.push(StrategyMatch::new(
                                    strategy.metadata.name.clone(),
                                    cells_with_value.clone(),
                                    Some(UnitType::Box),
                                    Some(box_idx),
                                    vec![value],
                                    context,
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        matches
    }
}

impl Default for PointingPairMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Matcher for X-Wing (candidate in 2 positions across 2 parallel units)
pub struct XWingMatcher;

impl XWingMatcher {
    pub fn new() -> Self {
        Self
    }
    
    /// Finds X-Wing patterns in rows (eliminates from columns)
    fn find_in_rows(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // For each candidate value
        for value in 1..=9 {
            // Find rows where this value appears in exactly 2 positions
            let mut rows_with_two_positions: Vec<(usize, Vec<usize>)> = Vec::new();
            
            for row_idx in 0..9 {
                if let Some(row) = board.get_row(row_idx) {
                    let mut positions = Vec::new();
                    
                    for &cell_idx in &row.cell_indices {
                        if let Some(cell) = board.get_cell(cell_idx) {
                            if !cell.is_solved() && cell.candidates.contains(value) {
                                positions.push(cell_idx % 9); // column index
                            }
                        }
                    }
                    
                    if positions.len() == 2 {
                        rows_with_two_positions.push((row_idx, positions));
                    }
                }
            }
            
            // Check all pairs of rows
            for i in 0..rows_with_two_positions.len() {
                for j in (i + 1)..rows_with_two_positions.len() {
                    let (row1, cols1) = &rows_with_two_positions[i];
                    let (row2, cols2) = &rows_with_two_positions[j];
                    
                    // Check if columns align
                    if cols1 == cols2 {
                        // Found an X-Wing! Eliminate from these columns in other rows
                        let col1 = cols1[0];
                        let col2 = cols1[1];
                        
                        let mut elimination_targets = Vec::new();
                        
                        // Check both columns
                        for &col_idx in &[col1, col2] {
                            if let Some(col) = board.get_column(col_idx) {
                                for &cell_idx in &col.cell_indices {
                                    let cell_row = cell_idx / 9;
                                    // Skip the X-Wing rows
                                    if cell_row != *row1 && cell_row != *row2 {
                                        if let Some(cell) = board.get_cell(cell_idx) {
                                            if !cell.is_solved() && cell.candidates.contains(value) {
                                                elimination_targets.push(cell_idx);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        if !elimination_targets.is_empty() {
                            let pattern_cells = vec![
                                row1 * 9 + col1,
                                row1 * 9 + col2,
                                row2 * 9 + col1,
                                row2 * 9 + col2,
                            ];
                            
                            let context = MatchContext::elimination(
                                elimination_targets,
                                vec![value],
                            );
                            
                            matches.push(StrategyMatch::new(
                                strategy.metadata.name.clone(),
                                pattern_cells,
                                Some(UnitType::Row),
                                None,
                                vec![value],
                                context,
                            ));
                        }
                    }
                }
            }
        }
        
        matches
    }
    
    /// Finds X-Wing patterns in columns (eliminates from rows)
    fn find_in_columns(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // For each candidate value
        for value in 1..=9 {
            // Find columns where this value appears in exactly 2 positions
            let mut cols_with_two_positions: Vec<(usize, Vec<usize>)> = Vec::new();
            
            for col_idx in 0..9 {
                if let Some(col) = board.get_column(col_idx) {
                    let mut positions = Vec::new();
                    
                    for &cell_idx in &col.cell_indices {
                        if let Some(cell) = board.get_cell(cell_idx) {
                            if !cell.is_solved() && cell.candidates.contains(value) {
                                positions.push(cell_idx / 9); // row index
                            }
                        }
                    }
                    
                    if positions.len() == 2 {
                        cols_with_two_positions.push((col_idx, positions));
                    }
                }
            }
            
            // Check all pairs of columns
            for i in 0..cols_with_two_positions.len() {
                for j in (i + 1)..cols_with_two_positions.len() {
                    let (col1, rows1) = &cols_with_two_positions[i];
                    let (col2, rows2) = &cols_with_two_positions[j];
                    
                    // Check if rows align
                    if rows1 == rows2 {
                        // Found an X-Wing! Eliminate from these rows in other columns
                        let row1 = rows1[0];
                        let row2 = rows1[1];
                        
                        let mut elimination_targets = Vec::new();
                        
                        // Check both rows
                        for &row_idx in &[row1, row2] {
                            if let Some(row) = board.get_row(row_idx) {
                                for &cell_idx in &row.cell_indices {
                                    let cell_col = cell_idx % 9;
                                    // Skip the X-Wing columns
                                    if cell_col != *col1 && cell_col != *col2 {
                                        if let Some(cell) = board.get_cell(cell_idx) {
                                            if !cell.is_solved() && cell.candidates.contains(value) {
                                                elimination_targets.push(cell_idx);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        if !elimination_targets.is_empty() {
                            let pattern_cells = vec![
                                row1 * 9 + col1,
                                row1 * 9 + col2,
                                row2 * 9 + col1,
                                row2 * 9 + col2,
                            ];
                            
                            let context = MatchContext::elimination(
                                elimination_targets,
                                vec![value],
                            );
                            
                            matches.push(StrategyMatch::new(
                                strategy.metadata.name.clone(),
                                pattern_cells,
                                Some(UnitType::Column),
                                None,
                                vec![value],
                                context,
                            ));
                        }
                    }
                }
            }
        }
        
        matches
    }
}

impl PatternMatcher for XWingMatcher {
    fn find_matches(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        matches.extend(self.find_in_rows(board, strategy));
        matches.extend(self.find_in_columns(board, strategy));
        matches
    }
}

impl Default for XWingMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Matcher for Swordfish (candidate in 2-3 positions across 3 parallel units)
pub struct SwordfishMatcher;

impl SwordfishMatcher {
    pub fn new() -> Self {
        Self
    }
    
    /// Finds Swordfish patterns in rows (eliminates from columns)
    fn find_in_rows(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // For each candidate value
        for value in 1..=9 {
            // Find rows where this value appears in 2-3 positions
            let mut rows_with_positions: Vec<(usize, Vec<usize>)> = Vec::new();
            
            for row_idx in 0..9 {
                if let Some(row) = board.get_row(row_idx) {
                    let mut positions = Vec::new();
                    
                    for &cell_idx in &row.cell_indices {
                        if let Some(cell) = board.get_cell(cell_idx) {
                            if !cell.is_solved() && cell.candidates.contains(value) {
                                positions.push(cell_idx % 9); // column index
                            }
                        }
                    }
                    
                    if positions.len() >= 2 && positions.len() <= 3 {
                        rows_with_positions.push((row_idx, positions));
                    }
                }
            }
            
            // Check all triplets of rows
            for i in 0..rows_with_positions.len() {
                for j in (i + 1)..rows_with_positions.len() {
                    for k in (j + 1)..rows_with_positions.len() {
                        let (row1, cols1) = &rows_with_positions[i];
                        let (row2, cols2) = &rows_with_positions[j];
                        let (row3, cols3) = &rows_with_positions[k];
                        
                        // Collect all unique columns
                        let mut all_cols: Vec<usize> = Vec::new();
                        all_cols.extend(cols1);
                        all_cols.extend(cols2);
                        all_cols.extend(cols3);
                        all_cols.sort_unstable();
                        all_cols.dedup();
                        
                        // Swordfish: exactly 3 columns total
                        if all_cols.len() == 3 {
                            // Each row must use only these 3 columns
                            let valid = cols1.iter().all(|c| all_cols.contains(c))
                                && cols2.iter().all(|c| all_cols.contains(c))
                                && cols3.iter().all(|c| all_cols.contains(c));
                            
                            if valid {
                                let mut elimination_targets = Vec::new();
                                
                                // Eliminate from these columns in other rows
                                for &col_idx in &all_cols {
                                    if let Some(col) = board.get_column(col_idx) {
                                        for &cell_idx in &col.cell_indices {
                                            let cell_row = cell_idx / 9;
                                            // Skip the Swordfish rows
                                            if cell_row != *row1 && cell_row != *row2 && cell_row != *row3 {
                                                if let Some(cell) = board.get_cell(cell_idx) {
                                                    if !cell.is_solved() && cell.candidates.contains(value) {
                                                        elimination_targets.push(cell_idx);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                if !elimination_targets.is_empty() {
                                    let mut pattern_cells = Vec::new();
                                    for &row in &[row1, row2, row3] {
                                        for &col in &all_cols {
                                            let cell_idx = row * 9 + col;
                                            if let Some(cell) = board.get_cell(cell_idx) {
                                                if !cell.is_solved() && cell.candidates.contains(value) {
                                                    pattern_cells.push(cell_idx);
                                                }
                                            }
                                        }
                                    }
                                    
                                    let context = MatchContext::elimination(
                                        elimination_targets,
                                        vec![value],
                                    );
                                    
                                    matches.push(StrategyMatch::new(
                                        strategy.metadata.name.clone(),
                                        pattern_cells,
                                        Some(UnitType::Row),
                                        None,
                                        vec![value],
                                        context,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        matches
    }
    
    /// Finds Swordfish patterns in columns (eliminates from rows)
    fn find_in_columns(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // For each candidate value
        for value in 1..=9 {
            // Find columns where this value appears in 2-3 positions
            let mut cols_with_positions: Vec<(usize, Vec<usize>)> = Vec::new();
            
            for col_idx in 0..9 {
                if let Some(col) = board.get_column(col_idx) {
                    let mut positions = Vec::new();
                    
                    for &cell_idx in &col.cell_indices {
                        if let Some(cell) = board.get_cell(cell_idx) {
                            if !cell.is_solved() && cell.candidates.contains(value) {
                                positions.push(cell_idx / 9); // row index
                            }
                        }
                    }
                    
                    if positions.len() >= 2 && positions.len() <= 3 {
                        cols_with_positions.push((col_idx, positions));
                    }
                }
            }
            
            // Check all triplets of columns
            for i in 0..cols_with_positions.len() {
                for j in (i + 1)..cols_with_positions.len() {
                    for k in (j + 1)..cols_with_positions.len() {
                        let (col1, rows1) = &cols_with_positions[i];
                        let (col2, rows2) = &cols_with_positions[j];
                        let (col3, rows3) = &cols_with_positions[k];
                        
                        // Collect all unique rows
                        let mut all_rows: Vec<usize> = Vec::new();
                        all_rows.extend(rows1);
                        all_rows.extend(rows2);
                        all_rows.extend(rows3);
                        all_rows.sort_unstable();
                        all_rows.dedup();
                        
                        // Swordfish: exactly 3 rows total
                        if all_rows.len() == 3 {
                            // Each column must use only these 3 rows
                            let valid = rows1.iter().all(|r| all_rows.contains(r))
                                && rows2.iter().all(|r| all_rows.contains(r))
                                && rows3.iter().all(|r| all_rows.contains(r));
                            
                            if valid {
                                let mut elimination_targets = Vec::new();
                                
                                // Eliminate from these rows in other columns
                                for &row_idx in &all_rows {
                                    if let Some(row) = board.get_row(row_idx) {
                                        for &cell_idx in &row.cell_indices {
                                            let cell_col = cell_idx % 9;
                                            // Skip the Swordfish columns
                                            if cell_col != *col1 && cell_col != *col2 && cell_col != *col3 {
                                                if let Some(cell) = board.get_cell(cell_idx) {
                                                    if !cell.is_solved() && cell.candidates.contains(value) {
                                                        elimination_targets.push(cell_idx);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                if !elimination_targets.is_empty() {
                                    let mut pattern_cells = Vec::new();
                                    for &col in &[col1, col2, col3] {
                                        for &row in &all_rows {
                                            let cell_idx = row * 9 + col;
                                            if let Some(cell) = board.get_cell(cell_idx) {
                                                if !cell.is_solved() && cell.candidates.contains(value) {
                                                    pattern_cells.push(cell_idx);
                                                }
                                            }
                                        }
                                    }
                                    
                                    let context = MatchContext::elimination(
                                        elimination_targets,
                                        vec![value],
                                    );
                                    
                                    matches.push(StrategyMatch::new(
                                        strategy.metadata.name.clone(),
                                        pattern_cells,
                                        Some(UnitType::Column),
                                        None,
                                        vec![value],
                                        context,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        matches
    }
}

impl PatternMatcher for SwordfishMatcher {
    fn find_matches(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        matches.extend(self.find_in_rows(board, strategy));
        matches.extend(self.find_in_columns(board, strategy));
        matches
    }
}

impl Default for SwordfishMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Matcher for XY-Wing (pivot cell with XY, two wing cells with XZ and YZ)
pub struct XYWingMatcher;

impl XYWingMatcher {
    pub fn new() -> Self {
        Self
    }
    
    /// Helper to get all peer cells of a given cell
    fn get_peers(&self, board: &Board, cell_idx: usize) -> Vec<usize> {
        let mut peers = Vec::new();
        let row = cell_idx / 9;
        let col = cell_idx % 9;
        let box_idx = (row / 3) * 3 + (col / 3);
        
        // Add row peers
        if let Some(row_view) = board.get_row(row) {
            for &idx in &row_view.cell_indices {
                if idx != cell_idx {
                    peers.push(idx);
                }
            }
        }
        
        // Add column peers
        if let Some(col_view) = board.get_column(col) {
            for &idx in &col_view.cell_indices {
                if idx != cell_idx && !peers.contains(&idx) {
                    peers.push(idx);
                }
            }
        }
        
        // Add box peers
        if let Some(box_view) = board.get_box(box_idx) {
            for &idx in &box_view.cell_indices {
                if idx != cell_idx && !peers.contains(&idx) {
                    peers.push(idx);
                }
            }
        }
        
        peers
    }
    
    /// Check if two cells are peers (see each other)
    fn are_peers(&self, cell1: usize, cell2: usize) -> bool {
        let row1 = cell1 / 9;
        let col1 = cell1 % 9;
        let box1 = (row1 / 3) * 3 + (col1 / 3);
        
        let row2 = cell2 / 9;
        let col2 = cell2 % 9;
        let box2 = (row2 / 3) * 3 + (col2 / 3);
        
        row1 == row2 || col1 == col2 || box1 == box2
    }
}

impl PatternMatcher for XYWingMatcher {
    fn find_matches(&self, board: &Board, strategy: &Strategy) -> Vec<StrategyMatch> {
        let mut matches = Vec::new();
        
        // Find all cells with exactly 2 candidates (potential pivot and wings)
        let mut bi_value_cells: Vec<(usize, Vec<u8>)> = Vec::new();
        
        for cell_idx in 0..81 {
            if let Some(cell) = board.get_cell(cell_idx) {
                if !cell.is_solved() && cell.candidates.count() == 2 {
                    let candidates = cell.candidates.to_vec();
                    bi_value_cells.push((cell_idx, candidates));
                }
            }
        }
        
        // Try each bi-value cell as a pivot
        for (pivot_idx, pivot_cands) in &bi_value_cells {
            let x = pivot_cands[0];
            let y = pivot_cands[1];
            
            // Find potential wing cells that share exactly one candidate with pivot
            let pivot_peers = self.get_peers(board, *pivot_idx);
            
            let mut wing1_candidates: Vec<(usize, Vec<u8>)> = Vec::new();
            let mut wing2_candidates: Vec<(usize, Vec<u8>)> = Vec::new();
            
            for (wing_idx, wing_cands) in &bi_value_cells {
                if wing_idx == pivot_idx {
                    continue;
                }
                
                // Wing must be a peer of pivot
                if !pivot_peers.contains(wing_idx) {
                    continue;
                }
                
                // Check if wing shares exactly one candidate with pivot
                let shares_x = wing_cands.contains(&x);
                let shares_y = wing_cands.contains(&y);
                
                if shares_x && !shares_y {
                    // Wing has X and some other value Z
                    wing1_candidates.push((*wing_idx, wing_cands.clone()));
                } else if shares_y && !shares_x {
                    // Wing has Y and some other value Z
                    wing2_candidates.push((*wing_idx, wing_cands.clone()));
                }
            }
            
            // Try all combinations of wing1 and wing2
            for (wing1_idx, wing1_cands) in &wing1_candidates {
                for (wing2_idx, wing2_cands) in &wing2_candidates {
                    // Wings must not be peers of each other
                    if self.are_peers(*wing1_idx, *wing2_idx) {
                        continue;
                    }
                    
                    // Find the Z value (the common candidate between wings)
                    let z_values: Vec<u8> = wing1_cands.iter()
                        .filter(|&&c| c != x && wing2_cands.contains(&c))
                        .copied()
                        .collect();
                    
                    if z_values.len() != 1 {
                        continue;
                    }
                    
                    let z = z_values[0];
                    
                    // Find cells that can see both wings and have Z as a candidate
                    let wing1_peers = self.get_peers(board, *wing1_idx);
                    let wing2_peers = self.get_peers(board, *wing2_idx);
                    
                    let mut elimination_targets = Vec::new();
                    
                    for cell_idx in 0..81 {
                        if cell_idx == *pivot_idx || cell_idx == *wing1_idx || cell_idx == *wing2_idx {
                            continue;
                        }
                        
                        // Cell must see both wings
                        if wing1_peers.contains(&cell_idx) && wing2_peers.contains(&cell_idx) {
                            if let Some(cell) = board.get_cell(cell_idx) {
                                if !cell.is_solved() && cell.candidates.contains(z) {
                                    elimination_targets.push(cell_idx);
                                }
                            }
                        }
                    }
                    
                    if !elimination_targets.is_empty() {
                        let context = MatchContext::elimination(
                            elimination_targets,
                            vec![z],
                        );
                        
                        matches.push(StrategyMatch::new(
                            strategy.metadata.name.clone(),
                            vec![*pivot_idx, *wing1_idx, *wing2_idx],
                            None,
                            None,
                            vec![x, y, z],
                            context,
                        ));
                    }
                }
            }
        }
        
        matches
    }
}

impl Default for XYWingMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a matcher for the given strategy name
pub fn create_matcher(strategy_name: &str) -> Option<Box<dyn PatternMatcher>> {
    match strategy_name {
        "naked_single" => Some(Box::new(NakedSingleMatcher::new())),
        "hidden_single" => Some(Box::new(HiddenSingleMatcher::new())),
        "naked_pair" => Some(Box::new(NakedPairMatcher::new())),
        "pointing_pair" => Some(Box::new(PointingPairMatcher::new())),
        "x_wing" => Some(Box::new(XWingMatcher::new())),
        "swordfish" => Some(Box::new(SwordfishMatcher::new())),
        "xy_wing" => Some(Box::new(XYWingMatcher::new())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naked_single_matcher() {
        use crate::solver::Solver;
        
        let mut board = Board::new();
        
        // Set up a cell with only one candidate by filling most of a row
        for i in 0..8 {
            board.set_cell_value(i, (i + 1) as u8).unwrap();
        }
        
        // Propagate constraints so cell 8 has only candidate 9
        let mut solver = Solver::new();
        solver.propagate_initial_constraints(&mut board).unwrap();
        
        let matcher = NakedSingleMatcher::new();
        let strategy = Strategy {
            metadata: super::super::types::StrategyMetadata {
                name: "naked_single".to_string(),
                difficulty: 1,
                description: "Test".to_string(),
                applicable_dimensions: vec!["9x9".to_string()],
            },
            pattern: super::super::types::StrategyPattern::SingleCell {
                conditions: vec![],
            },
            action: super::super::types::StrategyAction::SetCellValue {
                target: super::super::types::TargetCells::MatchedCells,
                value: super::super::types::CandidateSource::SingleCandidate,
            },
            priority: 100,
        };
        
        let matches = matcher.find_matches(&board, &strategy);
        
        // Cell 8 should have only candidate 9
        assert!(!matches.is_empty());
        assert_eq!(matches[0].cell_indices[0], 8);
        assert_eq!(matches[0].candidates[0], 9);
    }

    #[test]
    fn test_create_matcher() {
        assert!(create_matcher("naked_single").is_some());
        assert!(create_matcher("hidden_single").is_some());
        assert!(create_matcher("naked_pair").is_some());
        assert!(create_matcher("pointing_pair").is_some());
        assert!(create_matcher("unknown_strategy").is_none());
    }
}
