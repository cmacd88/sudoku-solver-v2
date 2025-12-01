# Sudoku Solver v2 - MVP

A high-performance Sudoku solver written in Rust, featuring constraint propagation with zero-cost view abstractions.

**NB! I don't really know Rust as a language, so this is a way to test how well AI LLMs can implement my own ideas.
This was vibe coded with blackbox ai, and currently is missing some features because I ran out of credits before I could have it implement everything.
So far it covers a lot of puzzles, but gets very confused when there are multiple solutions to the same problem.**

## Features (MVP)

- ✅ **View Abstractions**: Zero-cost access to constraint groups (rows, columns, boxes)
- ✅ **Constraint Propagation**: Efficient candidate elimination using pre-computed views
- ✅ **Bitset Candidates**: Fast set operations using bitwise operations
- ✅ **JSON Strategy System**: Load and apply strategies from JSON files
- ✅ **Multiple Strategies**: Naked singles, hidden singles, naked pairs, pointing pairs
- ✅ **Strategy Selection**: Multiple policies (Priority, Difficulty, FirstMatch)
- ✅ **CLI Interface**: Simple command-line interface for solving puzzles
- ✅ **Validation**: Detects contradictions and validates board state
- ✅ **Comprehensive Testing**: 71 tests covering all functionality

## Architecture Highlights

### View Abstractions
Instead of iterating over the entire board to find cells in a row, column, or box, we pre-compute indices and provide lightweight view objects. This enables:
- **O(1) access** to constraint groups
- **Direct peer access** for constraint propagation
- **Zero runtime overhead** using Rust's lifetime system

### Bitset Representation
Each cell's candidates are stored as a bitset (u16), enabling:
- Fast set operations (union, intersection, difference)
- Efficient solved cell detection (popcount == 1)
- Memory-efficient storage

## Installation

```bash
# Clone the repository
git clone <repo-url>
cd sudoku-solver-v2

# Build the project
cargo build --release
```

## Usage

### Solve from String
```bash
cargo run --release -- solve "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
```

### Solve from File
```bash
cargo run --release -- solve puzzles/easy1.txt
```

### Puzzle Format
- 81 characters representing the 9x9 grid
- Use `0` or `.` for empty cells
- Use `1`-`9` for clues
- Whitespace is ignored

Example:
```
530070000
600195000
098000060
800060003
400803001
700020006
060000280
000419005
000080079
```

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_solve_easy_puzzle_1
```

## Project Structure

```
sudoku-solver-v2/
├── src/
│   ├── main.rs              # CLI application
│   ├── lib.rs               # Library root
│   ├── board/
│   │   ├── mod.rs           # Board with pre-computed views
│   │   ├── candidates.rs    # Bitset candidate operations
│   │   └── views.rs         # View abstractions
│   ├── solver/
│   │   └── mod.rs           # Constraint propagation solver
│   ├── strategy/
│   │   ├── mod.rs           # Strategy system exports
│   │   ├── types.rs         # Strategy type definitions
│   │   ├── bank.rs          # Strategy loading and management
│   │   ├── matcher.rs       # Pattern matching implementations
│   │   └── selector.rs      # Strategy selection logic
│   └── io/
│       └── mod.rs           # Puzzle loading and formatting
├── strategies/              # JSON strategy definitions
│   ├── README.md            # Strategy documentation
│   ├── basic/
│   │   ├── naked_single.json
│   │   └── hidden_single.json
│   └── intermediate/
│       ├── naked_pair.json
│       └── pointing_pair.json
├── tests/
│   ├── integration_test.rs      # Integration tests
│   ├── edge_cases_test.rs       # Edge case tests
│   └── strategy_system_test.rs  # Strategy system tests
├── puzzles/
│   ├── easy1.txt
│   ├── easy2.txt
│   └── hard1.txt
└── Cargo.toml
```

## JSON Strategy System

The solver now includes a flexible JSON-based strategy system that allows defining solving strategies in JSON files without code changes.

### Strategy Files

Strategies are organized in the `strategies/` directory:

```
strategies/
├── basic/
│   ├── naked_single.json      # Cells with one candidate
│   └── hidden_single.json     # Values with one position
└── intermediate/
    ├── naked_pair.json        # Two cells, same two candidates
    └── pointing_pair.json     # Candidates pointing to a line
```

### Using Strategies

```rust
use sudoku_solver_v2::strategy::{StrategyBank, StrategySelector, SelectionPolicy};

// Load strategies from directory
let bank = StrategyBank::load_from_directory("strategies")?;

// Create a selector with a policy
let mut selector = StrategySelector::new(SelectionPolicy::Priority);

// Select and apply strategies
if let Some((strategy, matches)) = selector.select_strategy(&board, bank.get_all_strategies()) {
    for strategy_match in matches {
        selector.apply_match(&mut board, &strategy_match)?;
    }
}

// View statistics
let stats = selector.statistics();
println!("Total applications: {}", stats.total_applications());
```

### Adding New Strategies

See `strategies/README.md` for detailed documentation on creating new strategy JSON files.

## Current Limitations

- Only supports 9x9 boards (hardcoded)
- ~~No JSON strategy loading~~ ✅ **Implemented**
- No logging system
- No speculative execution for hard puzzles
- Limited to basic/intermediate strategies (no X-Wing, Swordfish, etc. yet)

## Future Enhancements

1. **JSON Strategy System**: Load strategies dynamically from JSON files
2. **Logging**: Comprehensive event logging for debugging and ML training
3. **Multiple Board Sizes**: Support 6x6, 16x16, and other variants
4. **Speculative Execution**: Parallel branch exploration for hard puzzles
5. **Advanced Strategies**: X-Wing, Swordfish, XY-Wing, etc.
6. **Performance Profiling**: Built-in profiler for strategy timing
7. **Visualization**: Real-time solving visualization via event streaming
8. **ML Integration**: Strategy selection using machine learning

## Performance

The MVP focuses on correctness and demonstrating the view abstraction concept. Performance optimizations include:
- Pre-computed constraint graphs
- Bitset operations for candidates
- Zero-cost view abstractions
- Early contradiction detection

## Examples

### Example 1: Easy Puzzle
```bash
$ cargo run --release -- solve puzzles/easy1.txt

Sudoku Solver v2 - MVP

Loaded puzzle from file: puzzles/easy1.txt

Initial Board:
5 3 . | . 7 . | . . . 
6 . . | 1 9 5 | . . . 
. 9 8 | . . . | . 6 . 
------+-------+------
8 . . | . 6 . | . . 3 
4 . . | 8 . 3 | . . 1 
7 . . | . 2 . | . . 6 
------+-------+------
. 6 . | . . . | 2 8 . 
. . . | 4 1 9 | . . 5 
. . . | . 8 . | . 7 9 

Statistics:
  Solved cells: 30/81
  Unsolved cells: 51
  Completion: 37.0%

Solving...

✓ Puzzle solved successfully!

Final Board:
5 3 4 | 6 7 8 | 9 1 2 
6 7 2 | 1 9 5 | 3 4 8 
1 9 8 | 3 4 2 | 5 6 7 
------+-------+------
8 5 9 | 7 6 1 | 4 2 3 
4 2 6 | 8 5 3 | 7 9 1 
7 1 3 | 9 2 4 | 8 5 6 
------+-------+------
9 6 1 | 5 3 7 | 2 8 4 
2 8 7 | 4 1 9 | 6 3 5 
3 4 5 | 2 8 6 | 1 7 9 

Statistics:
  Solved cells: 81/81
  Unsolved cells: 0
  Completion: 100.0%

✓ Board is valid (no contradictions)
```

## Contributing

This is an MVP demonstrating the core architecture. Future contributions will focus on:
- Implementing the full strategy system
- Adding comprehensive logging
- Supporting multiple board sizes
- Performance optimizations

## License

MIT License (or your preferred license)

## Acknowledgments

Based on the architecture document that emphasizes:
- Zero-cost view abstractions for efficient constraint access
- Modular design with clear separation of concerns
- Performance-focused implementation using Rust
