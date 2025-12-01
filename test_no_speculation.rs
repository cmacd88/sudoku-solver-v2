use sudoku_solver_v2::{Board, Solver};

fn main() {
    let puzzle = "800000000003600000070090200050007000000045700000100030001000068008500010090000400";
    let mut board = Board::from_string(puzzle).unwrap();
    
    println!("Testing with backtracking only (no speculation)...\n");
    
    // Create solver with strategies but speculation disabled
    let mut solver = Solver::with_strategies("strategies").unwrap();
    
    match solver.solve(&mut board) {
        Ok(()) => {
            if board.is_solved() && board.is_valid() {
                println!("✓ Solved successfully!\n");
                println!("{}", board);
            } else {
                println!("⚠ Partial solution\n");
                println!("{}", board);
            }
        }
        Err(e) => {
            println!("✗ Failed: {}\n", e);
            println!("{}", board);
        }
    }
}
