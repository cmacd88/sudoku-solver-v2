//! Sudoku Solver v2 - CLI Application

use sudoku_solver_v2::{Solver, SpeculationConfig, SpeculationMode, io};
use std::env;
use std::path::Path;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let verbosity: u8 = args.iter().map(|a| match a.as_str() {
        "-v" => 1,
        "-vv" => 2,
        "-vvv" => 3,
        _ => 0,
    }).sum();

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
            let config = parse_speculation_config(&args[3..]);
            solve_puzzle(puzzle_input, config, verbosity);
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

fn parse_speculation_config(args: &[String]) -> SpeculationConfig {
    let mut config = SpeculationConfig::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "--speculation-mode" | "-s" => {
                if i + 1 < args.len() {
                    if let Some(mode) = SpeculationMode::from_str(&args[i + 1]) {
                        config.mode = mode;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--speculation-depth" | "-d" => {
                if i + 1 < args.len() {
                    if let Ok(depth) = args[i + 1].parse::<usize>() {
                        config.max_depth = depth;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--no-speculation" => {
                config.enabled = false;
                i += 1;
            }
            "--no-stats" => {
                config.track_statistics = false;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    
    config
}

fn solve_puzzle(input: &str, speculation_config: SpeculationConfig, verbosity: u8) {
    println!("Sudoku Solver v2 - Advanced Strategy System\n");

    let mut board = if Path::new(input).exists() {
        match io::load_puzzle_from_file(Path::new(input)) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading puzzle from file: {}", e);
                return;
            }
        }
    } else {
        match io::load_puzzle_from_string(input) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error parsing puzzle: {}", e);
                return;
            }
        }
    };

        if board.is_valid() {
        if verbosity >= 1 {
            println!("✓ Board is valid (no contradictions)\n");
        }
    } else {
        println!("✗ Board has contradictions!\n");
    }
    if verbosity >= 2 {
        println!("{}\n", io::format_statistics(&board));
    }

    if verbosity >= 1 {
        println!("\n Initial Board:");
        println!("{}", board);
    }


    let mut solver = match Solver::with_speculation("strategies", speculation_config.clone()) {
        Ok(s) => {
            if verbosity >= 2 {
                println!("Solving with advanced strategies...\n"); //this should be loglevel 2
                println!("✓ Loaded strategy system");
                if speculation_config.enabled {
                    println!("✓ Speculation enabled (mode: {:?}, depth: {})\n",
                        speculation_config.mode, speculation_config.max_depth);
                } else {
                    println!("✓ Speculation disabled (using backtracking)\n");
                }
            }
            s
        }
        Err(e) => {
            eprintln!("⚠ Failed to load strategies: {}", e);
            eprintln!("Falling back to basic solver\n");
            Solver::new()
        }
    };

        let start = std::time::Instant::now();
        let solve_result = solver.solve(&mut board);
        let elapsed = start.elapsed();
        
        if verbosity >= 2 {
            if let Some(stats) = solver.get_speculation_stats() {
                println!("{}", stats);
            }
        }
        match solve_result {
        Ok(()) => {
            if board.is_complete() {
                if verbosity >= 1 {
                    println!("✓ Puzzle solved successfully!\n");
                    println!("Time elapsed: {:?}\n", elapsed);
                }
            } else {
                println!("⚠ Partial solution (needs more advanced strategies or guessing)");
            }
        }
        Err(e) => {
            println!("✗ Solving failed: {}", e);
        }
    }

    
    // Printing the final board.
    println!("Final Board:");
    println!("{}", board);
    
    if verbosity >= 2 {
        println!("{}\n", io::format_statistics(&board));
    }    
}

fn print_usage() {
    println!("Sudoku Solver v2 - Advanced Speculation System");
    println!("\nUsage:");
    println!("  sudoku-solver-v2 solve <puzzle> [options]");
    println!("  sudoku-solver-v2 help");
    println!("\nArguments:");
    println!("  <puzzle>    Either a file path or an 81-character string");
    println!("              Use '0' or '.' for empty cells, '1'-'9' for clues");
    println!("\nOptions:");
    println!("  --speculation-mode, -s <mode>   Speculation mode: sequential, parallel, hybrid (default: hybrid)");
    println!("  --speculation-depth, -d <depth> Maximum speculation depth (default: 3)");
    println!("  --no-speculation                Disable speculation (use backtracking)");
    println!("  --no-stats                      Disable statistics tracking");
    println!("\nExamples:");
    println!("  sudoku-solver-v2 solve puzzle.txt");
    println!("  sudoku-solver-v2 solve puzzle.txt -s parallel -d 5");
    println!("  sudoku-solver-v2 solve puzzle.txt --no-speculation");
    println!("  sudoku-solver-v2 solve \"530070000600195000098000060800060003400803001700020006060000280000419005000080079\"");
}
