//! Sudoku Solver v2 - CLI Application

use sudoku_solver_v2::{Solver, io};
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "solve" => {
            if args.len() < 3 {
                eprintln!("Error: Missing puzzle argument");
                print_usage();
                return;
            }
            
            let puzzle_input = &args[2];
            solve_puzzle(puzzle_input);
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage();
        }
    }
}

fn solve_puzzle(input: &str) {
    println!("Sudoku Solver v2 - MVP\n");
    
    // Try to load puzzle from file first, then as string
    let mut board = if Path::new(input).exists() {
        match io::load_puzzle_from_file(Path::new(input)) {
            Ok(b) => {
                println!("Loaded puzzle from file: {}\n", input);
                b
            }
            Err(e) => {
                eprintln!("Error loading puzzle from file: {}", e);
                return;
            }
        }
    } else {
        match io::load_puzzle_from_string(input) {
            Ok(b) => {
                println!("Loaded puzzle from string\n");
                b
            }
            Err(e) => {
                eprintln!("Error parsing puzzle: {}", e);
                return;
            }
        }
    };
    
    println!("Initial Board:");
    println!("{}", board);
    println!("{}\n", io::format_statistics(&board));
    
    // Solve the puzzle
    println!("Solving...\n");
    let solver = Solver::new();
    
    match solver.solve(&mut board) {
        Ok(()) => {
            if board.is_solved() {
                println!("✓ Puzzle solved successfully!\n");
            } else {
                println!("⚠ Partial solution (needs advanced strategies)\n");
            }
        }
        Err(e) => {
            println!("✗ Solving failed: {}\n", e);
        }
    }
    
    println!("Final Board:");
    println!("{}", board);
    println!("{}\n", io::format_statistics(&board));
    
    if board.is_valid() {
        println!("✓ Board is valid (no contradictions)");
    } else {
        println!("✗ Board has contradictions!");
    }
}

fn print_usage() {
    println!("Sudoku Solver v2 - MVP");
    println!("\nUsage:");
    println!("  sudoku-solver-v2 solve <puzzle>");
    println!("  sudoku-solver-v2 help");
    println!("\nArguments:");
    println!("  <puzzle>    Either a file path or an 81-character string");
    println!("              Use '0' or '.' for empty cells, '1'-'9' for clues");
    println!("\nExample:");
    println!("  sudoku-solver-v2 solve puzzle.txt");
    println!("  sudoku-solver-v2 solve \"530070000600195000098000060800060003400803001700020006060000280000419005000080079\"");
}
