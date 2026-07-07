//! Edge case tests for the Sudoku solver

use sudoku_solver_v2::board::Board;
use sudoku_solver_v2::solver::Solver;
use sudoku_solver_v2::SpeculationConfig;

#[test]
fn test_invalid_length_too_short() {
    let result = Board::from_string("12345");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Expected 81 characters"));
}

#[test]
fn test_invalid_length_too_long() {
    let puzzle = "0".repeat(100);
    let result = Board::from_string(&puzzle);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Expected 81 characters"));
}

#[test]
fn test_invalid_characters() {
    let puzzle = "A".repeat(81);
    let result = Board::from_string(&puzzle);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Invalid character"));
}

#[test]
fn test_mixed_invalid_characters() {
    let mut puzzle = "0".repeat(40);
    puzzle.push('X');
    puzzle.push_str(&"0".repeat(40));
    let result = Board::from_string(&puzzle);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Invalid character"));
}

#[test]
fn test_empty_puzzle() {
    let puzzle = "0".repeat(81);
    let result = Board::from_string(&puzzle);
    assert!(result.is_ok());
    
    let board = result.unwrap();
    assert_eq!(board.solved_count(), 0);
    assert_eq!(board.unsolved_count(), 81);
    
    // Should be valid (no contradictions)
    assert!(board.is_valid());
}

#[test]
fn test_puzzle_with_contradiction_same_row() {
    // Two 5s in the first row
    let puzzle = "550070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let result = Board::from_string(puzzle);
    assert!(result.is_ok());
    
    let board = result.unwrap();
    // Board should be invalid due to duplicate in row
    assert!(!board.is_valid());
}

#[test]
fn test_puzzle_with_contradiction_same_column() {
    // Two 5s in the first column
    let puzzle = "530070000500195000098000060800060003400803001700020006060000280000419005000080079";
    let result = Board::from_string(puzzle);
    assert!(result.is_ok());
    
    let board = result.unwrap();
    // Board should be invalid due to duplicate in column
    assert!(!board.is_valid());
}

#[test]
fn test_puzzle_with_contradiction_same_box() {
    // Two 5s in the top-left box
    let puzzle = "530570000600195000098000060800060003400803001700020006060000280000419005000080079";
    let result = Board::from_string(puzzle);
    assert!(result.is_ok());
    
    let board = result.unwrap();
    // Board should be invalid due to duplicate in box
    assert!(!board.is_valid());
}

#[test]
fn test_minimal_clues_puzzle() {
    // A puzzle with very few clues (17 is theoretical minimum for unique solution)
    let puzzle = "000000000000003085001020000000507000004000100090000000500000073002010000000040009";
    let result = Board::from_string(puzzle);
    assert!(result.is_ok());
    
    let board = result.unwrap();
    assert!(board.is_valid());
    assert!(board.solved_count() < 20);
}

#[test]
fn test_almost_solved_puzzle() {
    // Puzzle with only one empty cell
    let puzzle = "534678912672195348198342567859761423426853791713924856961537284287419635345286170";
    let result = Board::from_string(puzzle);
    assert!(result.is_ok());
    
    let mut board = result.unwrap();
    assert_eq!(board.solved_count(), 80);
    assert_eq!(board.unsolved_count(), 1);
    
    // Should solve easily
    let mut solver = Solver::new();
    let result = &mut solver.solve(&mut board);
    assert!(result.is_ok());
    assert!(board.is_solved());
}

#[test]
fn test_dots_as_empty_cells() {
    // Test that dots work as empty cells
    let puzzle = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    let result = Board::from_string(puzzle);
    assert!(result.is_ok());
    
    let board = result.unwrap();
    assert!(board.is_valid());
}

#[test]
fn test_zeros_as_empty_cells() {
    // Test that zeros work as empty cells
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let result = Board::from_string(puzzle);
    assert!(result.is_ok());
    
    let board = result.unwrap();
    assert!(board.is_valid());
}

#[test]
fn test_mixed_dots_and_zeros() {
    // Test mixing dots and zeros
    let puzzle = "53..70000600195...098000060800060003400803001700020006060000280...419005000080079";
    let result = Board::from_string(puzzle);
    assert!(result.is_ok());
    
    let board = result.unwrap();
    assert!(board.is_valid());
}

#[test]
fn test_hard_puzzle_partial_solve() {
    // A harder puzzle that won't fully solve with basic strategies
    let puzzle = "800000000003600000070090200050007000000045700000100030001000068008500010090000400";
    let result = Board::from_string(puzzle);
    assert!(result.is_ok());
    
    let mut board = result.unwrap();
    assert!(board.is_valid());
    
    // Should not fully solve but should make some progress
    let mut solver = Solver::new();
    solver.set_speculation_config(SpeculationConfig { enabled: false, ..Default::default() });
    let result = solver.solve(&mut board);
    assert!(result.is_ok());
    assert!(!board.is_complete());
    assert!(board.is_valid());
}

#[test]
fn test_all_nines() {
    // Invalid puzzle with all 9s
    let puzzle = "9".repeat(81);
    let result = Board::from_string(&puzzle);
    assert!(result.is_ok());
    
    let board = result.unwrap();
    // Should be invalid due to duplicates everywhere
    assert!(!board.is_valid());
}

#[test]
fn test_diagonal_pattern() {
    // Puzzle with values only on diagonal
    let puzzle = "100000000020000000003000000000400000000050000000006000000000700000000080000000009";
    let result = Board::from_string(puzzle);
    assert!(result.is_ok());
    
    let board = result.unwrap();
    assert!(board.is_valid());
    assert_eq!(board.solved_count(), 9);
}
