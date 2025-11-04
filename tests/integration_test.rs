//! Integration tests for the Sudoku solver

use sudoku_solver_v2::{Board, Solver};

#[test]
fn test_solve_easy_puzzle_1() {
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let initial_solved = board.solved_count();
    assert!(initial_solved > 0, "Puzzle should have initial clues");
    
    let solver = Solver::new();
    let result = solver.solve(&mut board);
    
    assert!(result.is_ok(), "Solver should not error");
    assert!(board.is_valid(), "Board should remain valid");
    assert!(board.solved_count() > initial_solved, "Should make progress");
}

#[test]
fn test_solve_easy_puzzle_2() {
    let puzzle = "003020600900305001001806400008102900700000008006708200002609500800203009005010300";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let initial_solved = board.solved_count();
    
    let solver = Solver::new();
    let result = solver.solve(&mut board);
    
    assert!(result.is_ok(), "Solver should not error");
    assert!(board.is_valid(), "Board should remain valid");
    assert!(board.solved_count() > initial_solved, "Should make progress");
}

#[test]
fn test_board_validation() {
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let board = Board::from_string(puzzle).unwrap();
    
    assert!(board.is_valid(), "Valid puzzle should pass validation");
}

#[test]
fn test_invalid_puzzle() {
    // Two 5s in the first row
    let puzzle = "550070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let board = Board::from_string(puzzle).unwrap();
    
    assert!(!board.is_valid(), "Invalid puzzle should fail validation");
}

#[test]
fn test_constraint_propagation() {
    let mut board = Board::new();
    
    // Set up a simple scenario
    board.set_cell_value(0, 1).unwrap();
    board.set_cell_value(1, 2).unwrap();
    board.set_cell_value(2, 3).unwrap();
    
    let solver = Solver::new();
    let result = solver.solve(&mut board);
    
    assert!(result.is_ok());
    
    // Check that constraints were propagated
    let constraints = board.get_cell_constraints(0).unwrap();
    for &peer_idx in &constraints.peer_indices {
        let cell = board.get_cell(peer_idx).unwrap();
        if !cell.is_solved() {
            assert!(!cell.candidates.contains(1), "Peer should not have value 1 as candidate");
        }
    }
}

#[test]
fn test_empty_board() {
    let puzzle = "000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let mut board = Board::from_string(puzzle).unwrap();
    
    assert_eq!(board.solved_count(), 0);
    assert!(board.is_valid());
    
    let solver = Solver::new();
    let result = solver.solve(&mut board);
    
    // Empty board can't be solved without guessing
    assert!(result.is_ok());
}
