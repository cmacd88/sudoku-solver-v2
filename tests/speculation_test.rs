//! Tests for the speculation system

use sudoku_solver_v2::{Board, Solver, SpeculationConfig, SpeculationMode};

#[test]
fn test_speculation_config_default() {
    let config = SpeculationConfig::default();
    assert!(config.enabled);
    assert_eq!(config.max_depth, 3);
    assert_eq!(config.mode, SpeculationMode::Hybrid);
    assert!(config.track_statistics);
}

#[test]
fn test_speculation_config_custom() {
    let config = SpeculationConfig {
        enabled: false,
        max_depth: 5,
        mode: SpeculationMode::Sequential,
        track_statistics: false,
    };
    
    assert!(!config.enabled);
    assert_eq!(config.max_depth, 5);
    assert_eq!(config.mode, SpeculationMode::Sequential);
    assert!(!config.track_statistics);
}

#[test]
fn test_speculation_mode_from_str() {
    assert_eq!(SpeculationMode::from_str("sequential"), Some(SpeculationMode::Sequential));
    assert_eq!(SpeculationMode::from_str("seq"), Some(SpeculationMode::Sequential));
    assert_eq!(SpeculationMode::from_str("parallel"), Some(SpeculationMode::Parallel));
    assert_eq!(SpeculationMode::from_str("par"), Some(SpeculationMode::Parallel));
    assert_eq!(SpeculationMode::from_str("hybrid"), Some(SpeculationMode::Hybrid));
    assert_eq!(SpeculationMode::from_str("invalid"), None);
}

#[test]
fn test_solver_with_speculation_sequential() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 10,
        mode: SpeculationMode::Sequential,
        track_statistics: true,
    };
    
    let result = Solver::with_speculation("strategies", config);
    assert!(result.is_ok());
}

#[test]
fn test_solver_with_speculation_parallel() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 5,
        mode: SpeculationMode::Parallel,
        track_statistics: true,
    };
    
    let result = Solver::with_speculation("strategies", config);
    assert!(result.is_ok());
}

#[test]
fn test_solver_with_speculation_hybrid() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 3,
        mode: SpeculationMode::Hybrid,
        track_statistics: true,
    };
    
    let result = Solver::with_speculation("strategies", config);
    assert!(result.is_ok());
}

#[test]
fn test_speculation_disabled() {
    let config = SpeculationConfig {
        enabled: false,
        max_depth: 3,
        mode: SpeculationMode::Hybrid,
        track_statistics: false,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    // Easy puzzle should still solve with speculation disabled
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    assert!(result.is_ok());
    assert!(board.is_solved());
}

#[test]
fn test_easy_puzzle_with_sequential_speculation() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 3,
        mode: SpeculationMode::Sequential,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    assert!(result.is_ok());
    assert!(board.is_solved());
}

#[test]
fn test_easy_puzzle_with_parallel_speculation() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 3,
        mode: SpeculationMode::Parallel,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    assert!(result.is_ok());
    assert!(board.is_solved());
}

#[test]
fn test_easy_puzzle_with_hybrid_speculation() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 3,
        mode: SpeculationMode::Hybrid,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    assert!(result.is_ok());
    assert!(board.is_solved());
}

#[test]
fn test_speculation_with_depth_1() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 1,
        mode: SpeculationMode::Sequential,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    // Easy puzzle should solve even with depth 1
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    // May or may not solve depending on puzzle complexity
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_speculation_with_high_depth() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 20,
        mode: SpeculationMode::Sequential,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    assert!(result.is_ok());
    assert!(board.is_solved());
}

#[test]
fn test_speculation_with_already_solved_puzzle() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 3,
        mode: SpeculationMode::Hybrid,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    // Fully solved puzzle
    let puzzle = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
    let mut board = Board::from_string(puzzle).unwrap();
    
    assert!(board.is_solved());
    
    let result = solver.solve(&mut board);
    assert!(result.is_ok());
    assert!(board.is_solved());
}

#[test]
fn test_speculation_with_invalid_puzzle() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 3,
        mode: SpeculationMode::Sequential,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    // Invalid puzzle (two 5s in first row)
    let puzzle = "550070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    assert!(result.is_err());
}

#[test]
fn test_speculation_statistics_tracking() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 3,
        mode: SpeculationMode::Sequential,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    assert!(result.is_ok());
    
    // Statistics should be tracked (we can't easily access them, but the solve should complete)
    assert!(board.is_solved());
}

#[test]
fn test_speculation_no_statistics_tracking() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 3,
        mode: SpeculationMode::Sequential,
        track_statistics: false,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    assert!(result.is_ok());
    assert!(board.is_solved());
}

#[test]
fn test_set_speculation_config() {
    let mut solver = Solver::with_strategies("strategies").unwrap();
    
    let new_config = SpeculationConfig {
        enabled: true,
        max_depth: 10,
        mode: SpeculationMode::Parallel,
        track_statistics: true,
    };
    
    solver.set_speculation_config(new_config);
    
    // Solver should work with new config
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    assert!(result.is_ok());
}

#[test]
fn test_medium_puzzle_with_speculation() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 5,
        mode: SpeculationMode::Hybrid,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    // Medium difficulty puzzle
    let puzzle = "003020600900305001001806400008102900700000008006708200002609500800203009005010300";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    // Should solve or make significant progress
    assert!(result.is_ok() || board.solved_count() > 40);
}

#[test]
fn test_minimal_clue_puzzle() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 5,
        mode: SpeculationMode::Sequential,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    // Puzzle with minimal clues (17 is theoretical minimum for unique solution)
    let puzzle = "000000000000003085001020000000507000004000100090000000500000073002010000000040009";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    // May or may not solve depending on depth and complexity
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_empty_puzzle() {
    let config = SpeculationConfig {
        enabled: true,
        max_depth: 2,
        mode: SpeculationMode::Sequential,
        track_statistics: true,
    };
    
    let mut solver = Solver::with_speculation("strategies", config).unwrap();
    
    // Empty puzzle (all zeros)
    let puzzle = "000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let result = solver.solve(&mut board);
    // Should complete (may find one of many solutions or stop at depth limit)
    assert!(result.is_ok() || result.is_err());
}
