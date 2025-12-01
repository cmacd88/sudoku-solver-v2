# Speculative Execution Implementation Plan

## Overview
Implement a hybrid speculation system that intelligently chooses between bifurcation and backtracking based on board state.

## Architecture

### 1. Solver Configuration
```rust
pub struct Solver {
    max_iterations: usize,
    strategy_bank: Option<StrategyBank>,
    use_strategies: bool,
    speculation_config: SpeculationConfig,
}

pub struct SpeculationConfig {
    enabled: bool,
    max_depth: usize,
    mode: SpeculationMode,
    track_statistics: bool,
}

pub enum SpeculationMode {
    Sequential,      // Try candidates one by one
    Parallel,        // Explore all branches simultaneously
    Hybrid,          // Choose based on board state (default)
}
```

### 2. Core Speculation Logic

#### Cell Selection Heuristic
```rust
fn find_best_speculation_cell(&self, board: &Board) -> Option<(usize, Vec<u8>)> {
    // Score cells based on:
    // 1. Number of candidates (prefer 2-3)
    // 2. Constraint density (cells in more constrained units)
    // 3. Impact potential (cells that affect many unsolved cells)
    
    // Return: (cell_index, candidates)
}
```

#### Contradiction Detection
```rust
fn is_contradiction(&self, board: &Board) -> bool {
    // Fast checks:
    // 1. Any cell with zero candidates
    // 2. Any unit with duplicate values
    // 3. Board validity check
}
```

#### Branch Exploration
```rust
fn solve_with_speculation(&self, board: &mut Board, depth: usize) -> SolverResult<()> {
    // 1. Apply logical strategies
    // 2. If solved, return success
    // 3. If contradiction, return failure
    // 4. If depth limit reached, fall back to backtracking
    // 5. Find best cell for speculation
    // 6. Choose mode based on board state:
    //    - If many 2-candidate cells: use bifurcation
    //    - If few constrained cells: use backtracking
    // 7. Explore branches and prune contradictions
}
```

### 3. Hybrid Mode Decision Logic

```rust
fn choose_speculation_strategy(&self, board: &Board) -> SpeculationStrategy {
    let cells_with_2_candidates = count_cells_with_n_candidates(board, 2);
    let cells_with_3_candidates = count_cells_with_n_candidates(board, 3);
    let total_unsolved = board.unsolved_count();
    
    if cells_with_2_candidates >= 5 {
        // Many binary choices - bifurcation is efficient
        SpeculationStrategy::Bifurcation
    } else if total_unsolved < 20 {
        // Nearly solved - backtracking is fine
        SpeculationStrategy::Backtracking
    } else {
        // Mixed state - use limited bifurcation
        SpeculationStrategy::LimitedBifurcation(2) // max 2 levels
    }
}
```

### 4. Statistics Tracking

```rust
pub struct SpeculationStatistics {
    branches_explored: usize,
    branches_pruned: usize,
    max_depth_reached: usize,
    contradictions_found: usize,
    speculation_mode_used: HashMap<String, usize>,
}
```

## Implementation Steps

### Phase 1: Core Infrastructure
- [ ] Add SpeculationConfig to Solver
- [ ] Implement cell selection heuristic
- [ ] Implement fast contradiction detection
- [ ] Add statistics tracking structure

### Phase 2: Sequential Bifurcation
- [ ] Implement basic bifurcation logic
- [ ] Add constraint propagation after each guess
- [ ] Implement branch pruning
- [ ] Test with hard puzzles

### Phase 3: Hybrid Mode
- [ ] Implement board state analysis
- [ ] Add decision logic for mode selection
- [ ] Integrate with existing backtracking
- [ ] Test mode switching

### Phase 4: Configuration & CLI
- [ ] Add CLI flags for speculation mode
- [ ] Add depth limit configuration
- [ ] Add statistics reporting option
- [ ] Update documentation

### Phase 5: Optimization (Optional)
- [ ] Implement parallel bifurcation with rayon
- [ ] Add caching for board states
- [ ] Optimize candidate selection
- [ ] Performance benchmarking

## Testing Strategy

### Test Cases
1. **Easy puzzles**: Should use logical strategies only
2. **Medium puzzles**: Should use limited speculation
3. **Hard puzzles**: Should use full speculation/bifurcation
4. **Evil puzzles**: Should efficiently prune branches
5. **Multiple solutions**: Should find first valid solution

### Performance Metrics
- Time to solve vs pure backtracking
- Number of branches explored
- Number of branches pruned
- Max depth reached

## Configuration Examples

```rust
// Default: Hybrid mode with depth 3
let solver = Solver::with_strategies("strategies")?;

// Custom: Sequential bifurcation with depth 5
let solver = Solver::with_speculation(
    "strategies",
    SpeculationConfig {
        enabled: true,
        max_depth: 5,
        mode: SpeculationMode::Sequential,
        track_statistics: true,
    }
)?;

// Disable speculation (use backtracking only)
let solver = Solver::with_speculation(
    "strategies",
    SpeculationConfig {
        enabled: false,
        ..Default::default()
    }
)?;
```

## Expected Benefits

1. **Performance**: 2-5x faster on hard puzzles
2. **Efficiency**: Fewer branches explored due to early pruning
3. **Flexibility**: User can choose mode based on needs
4. **Insights**: Statistics show solver behavior

## Risks & Mitigations

1. **Risk**: Speculation explosion with deep recursion
   - **Mitigation**: Strict depth limits (default: 3)

2. **Risk**: Slower on easy puzzles due to overhead
   - **Mitigation**: Only trigger after logical strategies fail

3. **Risk**: Complex code harder to maintain
   - **Mitigation**: Clear separation of concerns, good tests

## Timeline

- Phase 1: 30 minutes (infrastructure)
- Phase 2: 45 minutes (sequential bifurcation)
- Phase 3: 30 minutes (hybrid mode)
- Phase 4: 15 minutes (configuration)
- Testing: 30 minutes
- **Total**: ~2.5 hours

## Success Criteria

- [ ] All existing tests pass
- [ ] Hard puzzle solves faster than pure backtracking
- [ ] Statistics show branch pruning is effective
- [ ] User can configure speculation mode
- [ ] Code is well-documented and maintainable
