//! Sudoku Solver v2 - CLI Application

use sudoku_solver_v2::{Solver, SpeculationConfig, SpeculationMode, io, logging};
use std::env;
use std::path::Path;
use log::LevelFilter;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    // Parse command line arguments
    let log_level = parse_log_level(&args);
    let speculation_config = parse_speculation_config(&args);
    
    logging::init_logger_with_level(log_level);
    
    let command = &args[1];
    
    match command.as_str() {
        "solve" => {
            if args.len() < 3 {
                eprintln!("Error: Missing puzzle argument");
                print_usage();
                return;
            }
            
            let puzzle_input = &args[2];
            solve_puzzle(puzzle_input, speculation_config);
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
    
    for i in 0..args.len() {
        match args[i].as_str() {
            "--speculation-mode" | "-s" => {
                if i + 1 < args.len() {
                    if let Some(mode) = SpeculationMode::from_str(&args[i + 1]) {
                        config.mode = mode;
                    } else {
                        eprintln!("Warning: Invalid speculation mode '{}', using default (hybrid)", args[i + 1]);
                    }
                }
            }
            "--speculation-depth" | "-d" => {
                if i + 1 < args.len() {
                    if let Ok(depth) = args[i + 1].parse::<usize>() {
                        config.max_depth = depth;
                    } else {
                        eprintln!("Warning: Invalid depth '{}', using default (3)", args[i + 1]);
                    }
                }
            }
            "--no-speculation" => {
                config.enabled = false;
            }
            "--no-stats" => {
                config.track_statistics = false;
            }
            _ => {}
        }
    }
    
    config
}

fn parse_log_level(args: &[String]) -> LevelFilter {
    for i in 0..args.len() {
        if args[i] == "--log-level" || args[i] == "-l" {
            if i + 1 < args.len() {
                return match args[i + 1].to_lowercase().as_str() {
                    "off" => LevelFilter::Off,
                    "error" => LevelFilter::Error,
                    "warn" => LevelFilter::Warn,
                    "info" => LevelFilter::Info,
                    "debug" => LevelFilter::Debug,
                    "trace" => LevelFilter::Trace,
                    _ => {
                        eprintln!("Warning: Invalid log level '{}', using 'info'", args[i + 1]);
                        LevelFilter::Info
                    }
                };
            }
        }
    }
    
    // Check environment variable
    if let Ok(level) = env::var("RUST_LOG") {
        return match level.to_lowercase().as_str() {
            "off" => LevelFilter::Off,
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        };
    }
    
    // Default to Info
    LevelFilter::Info
}

fn solve_puzzle(input: &str, speculation_config: SpeculationConfig) {
    log::info!("Sudoku Solver v2 - Advanced Strategy System");
    println!("Sudoku Solver v2 - Advanced Strategy System\n");
    
    // Try to load puzzle from file first, then as string
    let mut board = if Path::new(input).exists() {
        match io::load_puzzle_from_file(Path::new(input)) {
            Ok(b) => {
                log::info!("Loaded puzzle from file: {}", input);
                println!("Loaded puzzle from file: {}\n", input);
                b
            }
            Err(e) => {
                log::error!("Error loading puzzle from file: {}", e);
                eprintln!("Error loading puzzle from file: {}", e);
                return;
            }
        }
    } else {
        match io::load_puzzle_from_string(input) {
            Ok(b) => {
                log::info!("Loaded puzzle from string");
                println!("Loaded puzzle from string\n");
                b
            }
            Err(e) => {
                log::error!("Error parsing puzzle: {}", e);
                eprintln!("Error parsing puzzle: {}", e);
                return;
            }
        }
    };
    
    println!("Initial Board:");
    println!("{}", board);
    println!("{}\n", io::format_statistics(&board));
    
    // Solve the puzzle with strategy system
    println!("Solving with advanced strategies...\n");
    log::info!("Starting solve process");
    
    let mut solver = match Solver::with_speculation("strategies", speculation_config.clone()) {
        Ok(s) => {
            println!("✓ Loaded strategy system");
            if speculation_config.enabled {
                println!("✓ Speculation enabled (mode: {:?}, depth: {})\n", 
                        speculation_config.mode, speculation_config.max_depth);
            } else {
                println!("✓ Speculation disabled (using legacy backtracking)\n");
            }
            s
        }
        Err(e) => {
            log::warn!("Failed to load strategies: {}", e);
            eprintln!("⚠ Failed to load strategies: {}", e);
            eprintln!("Falling back to basic solver\n");
            Solver::new()
        }
    };
    
    match solver.solve(&mut board) {
        Ok(()) => {
            if board.is_solved() {
                log::info!("Puzzle solved successfully!");
                println!("✓ Puzzle solved successfully!\n");
            } else {
                log::warn!("Partial solution achieved");
                println!("⚠ Partial solution (needs more advanced strategies or guessing)\n");
            }
        }
        Err(e) => {
            log::error!("Solving failed: {}", e);
            println!("✗ Solving failed: {}\n", e);
        }
    }
    
    println!("Final Board:");
    println!("{}", board);
    println!("{}\n", io::format_statistics(&board));
    
    if board.is_valid() {
        println!("✓ Board is valid (no contradictions)");
    } else {
        log::error!("Board has contradictions!");
        println!("✗ Board has contradictions!");
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
    println!("  --log-level, -l <level>         Set logging level (off, error, warn, info, debug, trace)");
    println!("                                  Default: info");
    println!("                                  Can also be set via RUST_LOG environment variable");
    println!("\n  --speculation-mode, -s <mode>   Set speculation mode (sequential, parallel, hybrid)");
    println!("                                  Default: hybrid");
    println!("                                  - sequential: Traditional backtracking");
    println!("                                  - parallel: Explore all branches in parallel");
    println!("                                  - hybrid: Intelligently choose based on board state");
    println!("\n  --speculation-depth, -d <num>   Set maximum speculation depth");
    println!("                                  Default: 3");
    println!("\n  --no-speculation                Disable speculation (use legacy backtracking)");
    println!("\n  --no-stats                      Disable speculation statistics tracking");
    println!("\nExamples:");
    println!("  sudoku-solver-v2 solve puzzle.txt");
    println!("  sudoku-solver-v2 solve puzzle.txt --log-level debug");
    println!("  sudoku-solver-v2 solve puzzle.txt --speculation-mode parallel");
    println!("  sudoku-solver-v2 solve puzzle.txt -s sequential -d 5");
    println!("  sudoku-solver-v2 solve puzzle.txt --no-speculation");
    println!("  sudoku-solver-v2 solve \"530070000600195000098000060800060003400803001700020006060000280000419005000080079\"");
    println!("  RUST_LOG=trace sudoku-solver-v2 solve puzzle.txt");
}
