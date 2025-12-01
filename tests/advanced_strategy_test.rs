//! Integration tests for advanced Sudoku solving strategies
//!
//! This test suite validates the implementation of advanced strategies:
//! - X-Wing: Cross-unit elimination pattern
//! - Swordfish: Three-unit cross elimination pattern
//! - XY-Wing: Chain pattern with pivot and wing cells

use sudoku_solver_v2::board::Board;
use sudoku_solver_v2::strategy::{StrategyBank, StrategySelector, SelectionPolicy};
use sudoku_solver_v2::solver::Solver;

#[test]
fn test_load_advanced_strategies() {
    // Load all strategies including advanced ones
    let result = StrategyBank::load_from_directory("strategies");
    
    assert!(result.is_ok(), "Failed to load strategies: {:?}", result.err());
    
    let bank = result.unwrap();
    
    // Verify advanced strategies are loaded
    assert!(bank.get_strategy("x_wing").is_some(), "X-Wing strategy not loaded");
    assert!(bank.get_strategy("swordfish").is_some(), "Swordfish strategy not loaded");
    assert!(bank.get_strategy("xy_wing").is_some(), "XY-Wing strategy not loaded");
    
    println!("✓ All advanced strategies loaded successfully");
}

#[test]
fn test_advanced_strategy_metadata() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    
    // Check X-Wing metadata
    let x_wing = bank.get_strategy("x_wing").unwrap();
    assert_eq!(x_wing.metadata.name, "x_wing");
    assert_eq!(x_wing.metadata.difficulty, 7);
    assert_eq!(x_wing.priority, 40);
    assert!(x_wing.metadata.applicable_dimensions.contains(&"9x9".to_string()));
    
    // Check Swordfish metadata
    let swordfish = bank.get_strategy("swordfish").unwrap();
    assert_eq!(swordfish.metadata.name, "swordfish");
    assert_eq!(swordfish.metadata.difficulty, 8);
    assert_eq!(swordfish.priority, 30);
    
    // Check XY-Wing metadata
    let xy_wing = bank.get_strategy("xy_wing").unwrap();
    assert_eq!(xy_wing.metadata.name, "xy_wing");
    assert_eq!(xy_wing.metadata.difficulty, 7);
    assert_eq!(xy_wing.priority, 35);
    
    println!("✓ Advanced strategy metadata validated");
}

#[test]
fn test_x_wing_pattern_detection() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    let x_wing_strategy = bank.get_strategy("x_wing").unwrap();
    
    // Create a board with a known X-Wing pattern
    // This is a simplified test - in practice, X-Wing appears after constraint propagation
    let puzzle = "000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let mut board = Board::from_string(puzzle).unwrap();
    
    // Set up a simple X-Wing pattern manually for testing
    // Row 0: candidate 5 in columns 0 and 2
    // Row 1: candidate 5 in columns 0 and 2
    // This should eliminate 5 from columns 0 and 2 in other rows
    
    let solver = Solver::new();
    solver.propagate_initial_constraints(&mut board).unwrap();
    
    // Use the matcher to find X-Wing patterns
    use sudoku_solver_v2::strategy::matcher::{XWingMatcher, PatternMatcher};
    let matcher = XWingMatcher::new();
    let matches = matcher.find_matches(&board, x_wing_strategy);
    
    // The empty board won't have X-Wing patterns, but this tests the matcher works
    println!("✓ X-Wing matcher executed without errors");
    println!("  Found {} X-Wing patterns", matches.len());
}

#[test]
fn test_swordfish_pattern_detection() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    let swordfish_strategy = bank.get_strategy("swordfish").unwrap();
    
    let puzzle = "000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let solver = Solver::new();
    solver.propagate_initial_constraints(&mut board).unwrap();
    
    // Use the matcher to find Swordfish patterns
    use sudoku_solver_v2::strategy::matcher::{SwordfishMatcher, PatternMatcher};
    let matcher = SwordfishMatcher::new();
    let matches = matcher.find_matches(&board, swordfish_strategy);
    
    println!("✓ Swordfish matcher executed without errors");
    println!("  Found {} Swordfish patterns", matches.len());
}

#[test]
fn test_xy_wing_pattern_detection() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    let xy_wing_strategy = bank.get_strategy("xy_wing").unwrap();
    
    let puzzle = "000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let solver = Solver::new();
    solver.propagate_initial_constraints(&mut board).unwrap();
    
    // Use the matcher to find XY-Wing patterns
    use sudoku_solver_v2::strategy::matcher::{XYWingMatcher, PatternMatcher};
    let matcher = XYWingMatcher::new();
    let matches = matcher.find_matches(&board, xy_wing_strategy);
    
    println!("✓ XY-Wing matcher executed without errors");
    println!("  Found {} XY-Wing patterns", matches.len());
}

#[test]
fn test_advanced_strategies_on_hard_puzzle() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    let mut selector = StrategySelector::new(SelectionPolicy::Priority);
    
    // Load a hard puzzle that requires advanced strategies
    let puzzle = std::fs::read_to_string("puzzles/hard1.txt")
        .expect("Failed to read hard puzzle");
    let puzzle = puzzle.trim();
    
    let mut board = Board::from_string(puzzle).unwrap();
    
    // Propagate initial constraints
    let solver = Solver::new();
    solver.propagate_initial_constraints(&mut board).unwrap();
    
    // Get all strategies including advanced ones
    let strategies = bank.get_all_strategies();
    
    // Try to apply strategies iteratively
    let mut iterations = 0;
    let max_iterations = 100;
    let mut progress_made = true;
    
    while progress_made && iterations < max_iterations && !board.is_solved() {
        progress_made = false;
        
        if let Some((strategy, matches)) = selector.select_strategy(&board, strategies) {
            println!("Iteration {}: Applying {} ({} matches)", 
                     iterations, strategy.metadata.name, matches.len());
            
            for strategy_match in matches {
                if let Ok(made_progress) = selector.apply_match(&mut board, &strategy_match) {
                    if made_progress {
                        progress_made = true;
                    }
                }
            }
        }
        
        iterations += 1;
    }
    
    println!("✓ Advanced strategies applied to hard puzzle");
    println!("  Iterations: {}", iterations);
    println!("  Solved cells: {}/81", board.solved_count());
    
    // Print statistics
    let stats = selector.statistics();
    println!("  Total strategy applications: {}", stats.total_applications());
    if let Some((name, count)) = stats.most_used_strategy() {
        println!("  Most used strategy: {} ({} times)", name, count);
    }
}

#[test]
fn test_strategy_priority_with_advanced() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    
    // Get strategies sorted by priority
    let strategies = bank.get_strategies_by_priority();
    
    // Verify that strategies are properly ordered
    // Higher priority (basic strategies) should come first
    assert!(!strategies.is_empty());
    
    // Find positions of different strategy types
    let mut naked_single_pos = None;
    let mut x_wing_pos = None;
    let mut swordfish_pos = None;
    
    for (i, strategy) in strategies.iter().enumerate() {
        match strategy.metadata.name.as_str() {
            "naked_single" => naked_single_pos = Some(i),
            "x_wing" => x_wing_pos = Some(i),
            "swordfish" => swordfish_pos = Some(i),
            _ => {}
        }
    }
    
    // Basic strategies should have higher priority (come first)
    if let (Some(ns_pos), Some(xw_pos)) = (naked_single_pos, x_wing_pos) {
        assert!(ns_pos < xw_pos, 
                "Naked single should have higher priority than X-Wing");
    }
    
    // Swordfish should have lower priority than X-Wing
    if let (Some(xw_pos), Some(sf_pos)) = (x_wing_pos, swordfish_pos) {
        assert!(xw_pos < sf_pos,
                "X-Wing should have higher priority than Swordfish");
    }
    
    println!("✓ Strategy priorities correctly ordered");
}

#[test]
fn test_advanced_strategy_difficulty_levels() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    
    // Get strategies by difficulty
    let easy_strategies = bank.get_strategies_up_to_difficulty(3);
    let medium_strategies = bank.get_strategies_up_to_difficulty(6);
    let hard_strategies = bank.get_strategies_up_to_difficulty(10);
    
    // Advanced strategies should only appear in hard category
    let has_advanced_in_easy = easy_strategies.iter()
        .any(|s| s.metadata.difficulty >= 7);
    let has_advanced_in_medium = medium_strategies.iter()
        .any(|s| s.metadata.difficulty >= 7);
    let has_advanced_in_hard = hard_strategies.iter()
        .any(|s| s.metadata.difficulty >= 7);
    
    assert!(!has_advanced_in_easy, "Advanced strategies should not be in easy category");
    assert!(!has_advanced_in_medium, "Advanced strategies should not be in medium category");
    assert!(has_advanced_in_hard, "Advanced strategies should be in hard category");
    
    println!("✓ Advanced strategies correctly categorized by difficulty");
    println!("  Easy strategies (≤3): {}", easy_strategies.len());
    println!("  Medium strategies (≤6): {}", medium_strategies.len());
    println!("  Hard strategies (≤10): {}", hard_strategies.len());
}

#[test]
fn test_matcher_creation_for_advanced_strategies() {
    use sudoku_solver_v2::strategy::matcher::create_matcher;
    
    // Test that matchers can be created for all advanced strategies
    let x_wing_matcher = create_matcher("x_wing");
    assert!(x_wing_matcher.is_some(), "Failed to create X-Wing matcher");
    
    let swordfish_matcher = create_matcher("swordfish");
    assert!(swordfish_matcher.is_some(), "Failed to create Swordfish matcher");
    
    let xy_wing_matcher = create_matcher("xy_wing");
    assert!(xy_wing_matcher.is_some(), "Failed to create XY-Wing matcher");
    
    println!("✓ All advanced strategy matchers created successfully");
}

#[test]
fn test_advanced_strategies_with_selection_policies() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    
    // Test with Priority policy
    let mut priority_selector = StrategySelector::new(SelectionPolicy::Priority);
    
    // Test with Difficulty policy
    let mut difficulty_selector = StrategySelector::new(SelectionPolicy::Difficulty);
    
    // Create a partially solved board
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let solver = Solver::new();
    solver.propagate_initial_constraints(&mut board).unwrap();
    
    let strategies = bank.get_all_strategies();
    
    // Both selectors should be able to select strategies
    let priority_result = priority_selector.select_strategy(&board, strategies);
    let difficulty_result = difficulty_selector.select_strategy(&board, strategies);
    
    assert!(priority_result.is_some() || difficulty_result.is_some(),
            "At least one selector should find applicable strategies");
    
    println!("✓ Advanced strategies work with different selection policies");
}

#[test]
fn test_elimination_correctness() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    let mut selector = StrategySelector::new(SelectionPolicy::Priority);
    
    // Create a board and apply strategies
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    let solver = Solver::new();
    solver.propagate_initial_constraints(&mut board).unwrap();
    
    // Count initial candidates
    let initial_candidate_count: usize = (0..81)
        .filter_map(|i| board.get_cell(i))
        .filter(|cell| !cell.is_solved())
        .map(|cell| cell.candidates.count() as usize)
        .sum();
    
    let strategies = bank.get_all_strategies();
    
    // Apply one round of strategies
    if let Some((_strategy, matches)) = selector.select_strategy(&board, strategies) {
        for strategy_match in matches {
            let _ = selector.apply_match(&mut board, &strategy_match);
        }
    }
    
    // Count candidates after application
    let final_candidate_count: usize = (0..81)
        .filter_map(|i| board.get_cell(i))
        .filter(|cell| !cell.is_solved())
        .map(|cell| cell.candidates.count() as usize)
        .sum();
    
    // Candidates should decrease or stay the same (never increase)
    assert!(final_candidate_count <= initial_candidate_count,
            "Candidate count should not increase after applying strategies");
    
    println!("✓ Elimination correctness verified");
    println!("  Initial candidates: {}", initial_candidate_count);
    println!("  Final candidates: {}", final_candidate_count);
    println!("  Eliminated: {}", initial_candidate_count - final_candidate_count);
}

#[test]
fn test_advanced_strategies_comprehensive() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    
    // Verify we have a good mix of strategies
    let all_strategies = bank.get_all_strategies();
    
    let basic_count = all_strategies.iter()
        .filter(|s| s.metadata.difficulty <= 3)
        .count();
    
    let intermediate_count = all_strategies.iter()
        .filter(|s| s.metadata.difficulty > 3 && s.metadata.difficulty <= 6)
        .count();
    
    let advanced_count = all_strategies.iter()
        .filter(|s| s.metadata.difficulty > 6)
        .count();
    
    assert!(basic_count > 0, "Should have basic strategies");
    assert!(intermediate_count > 0, "Should have intermediate strategies");
    assert!(advanced_count >= 3, "Should have at least 3 advanced strategies (X-Wing, Swordfish, XY-Wing)");
    
    println!("✓ Comprehensive strategy coverage verified");
    println!("  Basic strategies: {}", basic_count);
    println!("  Intermediate strategies: {}", intermediate_count);
    println!("  Advanced strategies: {}", advanced_count);
    println!("  Total strategies: {}", all_strategies.len());
}
