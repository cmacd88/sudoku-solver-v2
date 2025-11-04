//! I/O module for loading puzzles and formatting output.

use crate::board::Board;
use std::fs;
use std::path::Path;

/// Loads a puzzle from a file
pub fn load_puzzle_from_file(path: &Path) -> Result<Board, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    Board::from_string(&content)
}

/// Loads a puzzle from a string
pub fn load_puzzle_from_string(s: &str) -> Result<Board, String> {
    Board::from_string(s)
}

/// Formats a board for display with candidates
pub fn format_board_with_candidates(board: &Board) -> String {
    let mut output = String::new();
    
    output.push_str("Board State:\n");
    output.push_str(&format!("{}\n", board));
    
    output.push_str("\nCandidates:\n");
    for row in 0..9 {
        if row % 3 == 0 && row != 0 {
            output.push_str("------+-------+------\n");
        }
        
        for col in 0..9 {
            if col % 3 == 0 && col != 0 {
                output.push_str("| ");
            }
            
            let idx = row * 9 + col;
            if let Some(cell) = board.get_cell(idx) {
                if cell.is_solved() {
                    output.push_str(&format!("{} ", cell.value.unwrap()));
                } else {
                    output.push_str(&format!("{} ", cell.candidates));
                }
            }
        }
        output.push('\n');
    }
    
    output
}

/// Formats solving statistics
pub fn format_statistics(board: &Board) -> String {
    format!(
        "Statistics:\n  Solved cells: {}/81\n  Unsolved cells: {}\n  Completion: {:.1}%",
        board.solved_count(),
        board.unsolved_count(),
        (board.solved_count() as f64 / 81.0) * 100.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_puzzle_from_string() {
        let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let board = load_puzzle_from_string(puzzle);
        assert!(board.is_ok());
    }

    #[test]
    fn test_format_statistics() {
        let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let board = Board::from_string(puzzle).unwrap();
        let stats = format_statistics(&board);
        assert!(stats.contains("Solved cells:"));
    }
}
