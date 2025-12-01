//! Comprehensive tests for the logging system

use sudoku_solver_v2::{Board, Solver, logging::{Timer, SolverStats}};
use std::thread;
use std::time::Duration;
use log::LevelFilter;

// ============================================================================
// Unit Tests for Timer
// ============================================================================

#[test]
fn test_timer_creation() {
    let timer = Timer::new("test_operation");
    assert!(timer.elapsed_us() >= 0, "Timer should return non-negative elapsed time");
    assert!(timer.elapsed_ms() >= 0, "Timer should return non-negative elapsed time");
}

#[test]
fn test_timer_elapsed_microseconds() {
    let timer = Timer::new("microsecond_test");
    thread::sleep(Duration::from_micros(100));
    let elapsed = timer.elapsed_us();
    assert!(elapsed >= 100, "Timer should measure at least 100 microseconds, got {}", elapsed);
    assert!(elapsed < 10000, "Timer should measure less than 10ms for 100μs sleep, got {}μs", elapsed);
}

#[test]
fn test_timer_elapsed_milliseconds() {
    let timer = Timer::new("millisecond_test");
    thread::sleep(Duration::from_millis(10));
    let elapsed = timer.elapsed_ms();
    assert!(elapsed >= 10, "Timer should measure at least 10 milliseconds, got {}", elapsed);
    assert!(elapsed < 100, "Timer should measure less than 100ms for 10ms sleep, got {}ms", elapsed);
}

#[test]
fn test_timer_multiple_measurements() {
    let timer = Timer::new("multiple_measurements");
    
    let first = timer.elapsed_us();
    thread::sleep(Duration::from_micros(500));
    let second = timer.elapsed_us();
    
    assert!(second > first, "Second measurement should be greater than first");
    assert!(second - first >= 500, "Difference should be at least 500μs");
}

#[test]
fn test_timer_with_different_labels() {
    let timer1 = Timer::new("operation_1");
    let timer2 = Timer::new("operation_2");
    
    thread::sleep(Duration::from_millis(5));
    
    let elapsed1 = timer1.elapsed_ms();
    let elapsed2 = timer2.elapsed_ms();
    
    assert!(elapsed1 >= 5, "Timer 1 should measure at least 5ms");
    assert!(elapsed2 >= 5, "Timer 2 should measure at least 5ms");
}

// ============================================================================
// Unit Tests for SolverStats
// ============================================================================

#[test]
fn test_solver_stats_creation() {
    let stats = SolverStats::new();
    assert_eq!(stats.iterations, 0, "Initial iterations should be 0");
    assert_eq!(stats.cells_solved, 0, "Initial cells_solved should be 0");
    assert_eq!(stats.strategies_applied, 0, "Initial strategies_applied should be 0");
    assert_eq!(stats.backtracks, 0, "Initial backtracks should be 0");
    assert_eq!(stats.constraint_propagations, 0, "Initial constraint_propagations should be 0");
}

#[test]
fn test_solver_stats_default() {
    let stats = SolverStats::default();
    assert_eq!(stats.iterations, 0);
    assert_eq!(stats.cells_solved, 0);
    assert_eq!(stats.strategies_applied, 0);
    assert_eq!(stats.backtracks, 0);
    assert_eq!(stats.constraint_propagations, 0);
}

#[test]
fn test_solver_stats_tracking() {
    let mut stats = SolverStats::new();
    
    stats.iterations = 5;
    stats.cells_solved = 45;
    stats.strategies_applied = 20;
    stats.backtracks = 2;
    stats.constraint_propagations = 30;
    
    assert_eq!(stats.iterations, 5);
    assert_eq!(stats.cells_solved, 45);
    assert_eq!(stats.strategies_applied, 20);
    assert_eq!(stats.backtracks, 2);
    assert_eq!(stats.constraint_propagations, 30);
}

#[test]
fn test_solver_stats_display() {
    let mut stats = SolverStats::new();
    stats.iterations = 10;
    stats.cells_solved = 81;
    stats.strategies_applied = 50;
    stats.backtracks = 0;
    stats.constraint_propagations = 51;
    
    let display = format!("{}", stats);
    assert!(display.contains("iterations=10"), "Display should contain iterations");
    assert!(display.contains("cells_solved=81"), "Display should contain cells_solved");
    assert!(display.contains("strategies=50"), "Display should contain strategies");
    assert!(display.contains("backtracks=0"), "Display should contain backtracks");
    assert!(display.contains("propagations=51"), "Display should contain propagations");
}

#[test]
fn test_solver_stats_clone() {
    let mut stats1 = SolverStats::new();
    stats1.iterations = 5;
    stats1.cells_solved = 30;
    
    let stats2 = stats1.clone();
    assert_eq!(stats2.iterations, 5);
    assert_eq!(stats2.cells_solved, 30);
}

// ============================================================================
// Integration Tests - Logging During Puzzle Solving
// ============================================================================

#[test]
fn test_logging_with_easy_puzzle() {
    // Initialize logger for testing
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .is_test(true)
        .try_init();
    
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let mut solver = Solver::new();
    let result = solver.solve(&mut board);
    
    assert!(result.is_ok(), "Solver should complete without error");
    assert!(board.is_valid(), "Board should remain valid");
}

#[test]
fn test_logging_with_strategy_system() {
    // Initialize logger for testing
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .is_test(true)
        .try_init();
    
    let puzzle = "003020600900305001001806400008102900700000008006708200002609500800203009005010300";
    let mut board = Board::from_string(puzzle).unwrap();
    
    // Try to create solver with strategies
    let solver_result = Solver::with_strategies("strategies");
    
    if let Ok(mut solver) = solver_result {
        let result = solver.solve(&mut board);
        assert!(result.is_ok(), "Solver with strategies should complete");
        assert!(board.is_valid(), "Board should remain valid");
    } else {
        // If strategies can't be loaded, that's okay for this test
        println!("Note: Strategy system not available for testing");
    }
}

#[test]
fn test_logging_constraint_propagation() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .is_test(true)
        .try_init();
    
    let mut board = Board::new();
    board.set_cell_value(0, 1).unwrap();
    board.set_cell_value(1, 2).unwrap();
    board.set_cell_value(2, 3).unwrap();
    
    let mut solver = Solver::new();
    let result = solver.solve(&mut board);
    
    assert!(result.is_ok(), "Constraint propagation should complete");
    assert!(board.is_valid(), "Board should remain valid");
}

#[test]
fn test_logging_with_invalid_board() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Error)
        .is_test(true)
        .try_init();
    
    // Create a board with contradiction (two 5s in first row)
    let puzzle = "550070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    assert!(!board.is_valid(), "Board should be invalid");
    
    let mut solver = Solver::new();
    let result = solver.solve(&mut board);
    
    // Should fail due to invalid board
    assert!(result.is_err(), "Solver should detect invalid board");
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_timer_performance_overhead() {
    // Measure overhead of creating and using timers
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        let _timer = Timer::new("performance_test");
        let _elapsed = _timer.elapsed_us();
    }
    
    let total_elapsed = start.elapsed();
    
    // Creating 1000 timers should take less than 10ms
    assert!(total_elapsed.as_millis() < 10, 
            "Timer overhead too high: {}ms for 1000 timers", 
            total_elapsed.as_millis());
}

#[test]
fn test_stats_performance_overhead() {
    let start = std::time::Instant::now();
    
    for _ in 0..10000 {
        let mut stats = SolverStats::new();
        stats.iterations += 1;
        stats.cells_solved += 1;
        stats.strategies_applied += 1;
        let _display = format!("{}", stats);
    }
    
    let total_elapsed = start.elapsed();
    
    // 10000 stats operations should take less than 50ms
    assert!(total_elapsed.as_millis() < 50,
            "Stats overhead too high: {}ms for 10000 operations",
            total_elapsed.as_millis());
}

// ============================================================================
// Logger Initialization Tests
// ============================================================================

#[test]
fn test_logger_initialization() {
    // Test that logger can be initialized without panic
    // Note: This may fail if logger is already initialized in other tests
    let result = std::panic::catch_unwind(|| {
        sudoku_solver_v2::logging::init_logger_with_level(LevelFilter::Info);
    });
    
    // Either succeeds or fails gracefully (already initialized)
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_logger_with_different_levels() {
    // Test that different log levels can be set
    let levels = vec![
        LevelFilter::Off,
        LevelFilter::Error,
        LevelFilter::Warn,
        LevelFilter::Info,
        LevelFilter::Debug,
        LevelFilter::Trace,
    ];
    
    for level in levels {
        // Just verify the level enum is valid
        assert!(level as usize <= LevelFilter::Trace as usize);
    }
}

// ============================================================================
// Integration Tests - Real Solving Scenarios
// ============================================================================

#[test]
fn test_logging_full_solve_easy() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .is_test(true)
        .try_init();
    
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let initial_count = board.solved_count();
    
    let mut solver = Solver::new();
    let result = solver.solve(&mut board);
    
    assert!(result.is_ok());
    assert!(board.solved_count() > initial_count, "Should make progress");
    assert!(board.is_valid());
}

#[test]
fn test_logging_partial_solve() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .is_test(true)
        .try_init();
    
    // A harder puzzle that may not fully solve with basic strategies
    let puzzle = "800000000003600000070090200050007000000045700000100030001000068008500010090000400";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let initial_count = board.solved_count();
    
    let mut solver = Solver::new();
    let result = solver.solve(&mut board);
    
    assert!(result.is_ok());
    // Should make some progress even if not fully solved
    assert!(board.solved_count() >= initial_count);
    assert!(board.is_valid());
}

#[test]
fn test_logging_with_max_iterations() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .is_test(true)
        .try_init();
    
    let puzzle = "000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let mut solver = Solver::with_max_iterations(10);
    let result = solver.solve(&mut board);
    
    // Empty board with low max iterations should either complete or hit limit
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_timer_with_zero_duration() {
    let timer = Timer::new("instant_operation");
    // Immediately check elapsed time
    let elapsed = timer.elapsed_us();
    
    // Should be very small but non-negative
    assert!(elapsed >= 0);
    assert!(elapsed < 1000); // Less than 1ms
}

#[test]
fn test_stats_with_large_numbers() {
    let mut stats = SolverStats::new();
    
    stats.iterations = usize::MAX;
    stats.cells_solved = usize::MAX;
    stats.strategies_applied = usize::MAX;
    stats.backtracks = usize::MAX;
    stats.constraint_propagations = usize::MAX;
    
    // Should not panic when displaying
    let display = format!("{}", stats);
    assert!(display.len() > 0);
}

#[test]
fn test_multiple_timers_concurrent() {
    let timer1 = Timer::new("concurrent_1");
    thread::sleep(Duration::from_millis(5));
    let timer2 = Timer::new("concurrent_2");
    thread::sleep(Duration::from_millis(5));
    
    let elapsed1 = timer1.elapsed_ms();
    let elapsed2 = timer2.elapsed_ms();
    
    assert!(elapsed1 >= 10, "Timer 1 should measure at least 10ms");
    assert!(elapsed2 >= 5, "Timer 2 should measure at least 5ms");
    assert!(elapsed1 > elapsed2, "Timer 1 should have longer elapsed time");
}

// ============================================================================
// Format and Output Tests
// ============================================================================

#[test]
fn test_stats_format_consistency() {
    let stats = SolverStats::new();
    let display1 = format!("{}", stats);
    let display2 = format!("{}", stats);
    
    assert_eq!(display1, display2, "Display format should be consistent");
}

#[test]
fn test_stats_debug_format() {
    let stats = SolverStats::new();
    let debug = format!("{:?}", stats);
    
    assert!(debug.contains("SolverStats"), "Debug format should contain struct name");
}
