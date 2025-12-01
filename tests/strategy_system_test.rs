//! Integration tests for the JSON strategy system

use sudoku_solver_v2::board::Board;
use sudoku_solver_v2::strategy::{StrategyBank, StrategySelector, SelectionPolicy};

#[test]
fn test_load_strategies_from_directory() {
    // Load strategies from the strategies directory
    let result = StrategyBank::load_from_directory("strategies");
    
    assert!(result.is_ok(), "Failed to load strategies: {:?}", result.err());
    
    let bank = result.unwrap();
    assert!(bank.len() > 0, "No strategies loaded");
    
    // Check that basic strategies are loaded
    assert!(bank.get_strategy("naked_single").is_some());
    assert!(bank.get_strategy("hidden_single").is_some());
    assert!(bank.get_strategy("naked_pair").is_some());
    assert!(bank.get_strategy("pointing_pair").is_some());
}

#[test]
fn test_strategy_metadata() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    
    // Check naked single metadata
    let naked_single = bank.get_strategy("naked_single").unwrap();
    assert_eq!(naked_single.metadata.name, "naked_single");
    assert_eq!(naked_single.metadata.difficulty, 1);
    assert_eq!(naked_single.priority, 100);
    
    // Check hidden single metadata
    let hidden_single = bank.get_strategy("hidden_single").unwrap();
    assert_eq!(hidden_single.metadata.name, "hidden_single");
    assert_eq!(hidden_single.metadata.difficulty, 2);
    assert_eq!(hidden_single.priority, 90);
}

#[test]
fn test_strategy_selection_by_priority() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    let mut selector = StrategySelector::new(SelectionPolicy::Priority);
    
    // Create a board with a naked single
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    // Propagate initial constraints
    use sudoku_solver_v2::solver::Solver;
    let mut solver = Solver::new();
    solver.propagate_initial_constraints(&mut board).unwrap();
    
    // Get all strategies
    let strategies = bank.get_all_strategies();
    
    // Select a strategy
    let result = selector.select_strategy(&board, strategies);
    
    // Should find at least one applicable strategy
    assert!(result.is_some(), "No strategy selected");
    
    let (strategy, matches) = result.unwrap();
    assert!(!matches.is_empty(), "No matches found");
    
    println!("Selected strategy: {}", strategy.metadata.name);
    println!("Found {} matches", matches.len());
}

#[test]
fn test_strategy_selection_by_difficulty() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    let mut selector = StrategySelector::new(SelectionPolicy::Difficulty);
    
    // Create a board with multiple applicable strategies
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    // Propagate initial constraints
    use sudoku_solver_v2::solver::Solver;
    let mut solver = Solver::new();
    solver.propagate_initial_constraints(&mut board).unwrap();
    
    // Get all strategies
    let strategies = bank.get_all_strategies();
    
    // Select a strategy
    let result = selector.select_strategy(&board, strategies);
    
    assert!(result.is_some());
    let (strategy, _) = result.unwrap();
    
    // Should select the easiest strategy (naked_single with difficulty 1)
    println!("Selected strategy: {} (difficulty: {})", 
             strategy.metadata.name, strategy.metadata.difficulty);
}

#[test]
fn test_apply_strategy_match() {
    use sudoku_solver_v2::solver::Solver;
    
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    let mut selector = StrategySelector::new(SelectionPolicy::Priority);
    
    // Create a simple board
    let mut board = Board::new();
    
    // Fill first 8 cells of first row
    for i in 0..8 {
        board.set_cell_value(i, (i + 1) as u8).unwrap();
    }
    
    // Propagate constraints
    let mut solver = Solver::new();
    solver.propagate_initial_constraints(&mut board).unwrap();
    
    // Get strategies
    let strategies = bank.get_all_strategies();
    
    // Select and apply strategy
    if let Some((strategy, matches)) = selector.select_strategy(&board, strategies) {
        assert!(!matches.is_empty());
        
        let strategy_match = &matches[0];
        let result = selector.apply_match(&mut board, strategy_match);
        
        assert!(result.is_ok());
        assert!(result.unwrap(), "Strategy should make progress");
        
        // Cell 8 should now be solved with value 9
        assert!(board.is_cell_solved(8));
        assert_eq!(board.get_cell(8).unwrap().value, Some(9));
    } else {
        panic!("No strategy selected");
    }
}

#[test]
fn test_strategy_statistics() {
    use sudoku_solver_v2::solver::Solver;
    
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    let mut selector = StrategySelector::new(SelectionPolicy::Priority);
    
    // Create a board
    let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    let mut board = Board::from_string(puzzle).unwrap();
    
    // Propagate constraints
    let mut solver = Solver::new();
    solver.propagate_initial_constraints(&mut board).unwrap();
    
    // Get strategies
    let strategies = bank.get_all_strategies();
    
    // Apply strategies multiple times
    for _ in 0..5 {
        if let Some((_, matches)) = selector.select_strategy(&board, strategies) {
            for strategy_match in matches {
                let _ = selector.apply_match(&mut board, &strategy_match);
            }
        }
    }
    
    // Check statistics
    let stats = selector.statistics();
    assert!(stats.total_applications() > 0);
    
    println!("Total applications: {}", stats.total_applications());
    if let Some((name, count)) = stats.most_used_strategy() {
        println!("Most used strategy: {} ({} times)", name, count);
    }
}

#[test]
fn test_filter_strategies_by_dimensions() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    
    // All strategies should support 9x9
    let strategies_9x9 = bank.get_strategies_for_dimensions("9x9");
    assert!(strategies_9x9.len() > 0);
    
    // Check that all returned strategies support 9x9
    for strategy in strategies_9x9 {
        assert!(strategy.metadata.applicable_dimensions.contains(&"9x9".to_string()));
    }
}

#[test]
fn test_strategies_sorted_by_priority() {
    let bank = StrategyBank::load_from_directory("strategies").unwrap();
    
    let strategies = bank.get_strategies_by_priority();
    
    // Check that strategies are sorted by priority (highest first)
    for i in 1..strategies.len() {
        assert!(strategies[i-1].priority >= strategies[i].priority,
                "Strategies not sorted by priority");
    }
}
