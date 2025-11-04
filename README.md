# Sudoku Solver v2 - MVP

A high-performance Sudoku solver written in Rust, featuring constraint propagation with zero-cost view abstractions.

## Features (MVP)

- ✅ **View Abstractions**: Zero-cost access to constraint groups (rows, columns, boxes)
- ✅ **Constraint Propagation**: Efficient candidate elimination using pre-computed views
- ✅ **Bitset Candidates**: Fast set operations using bitwise operations
- ✅ **Basic Strategies**: Naked singles and hidden singles
- ✅ **CLI Interface**: Simple command-line interface for solving puzzles
- ✅ **Validation**: Detects contradictions and validates board state

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
│   │   └── mod.rs           # Strategy system (placeholder)
│   └── io/
│       └── mod.rs           # Puzzle loading and formatting
├── tests/
│   └── integration_test.rs  # Integration tests
├── puzzles/
│   ├── easy1.txt
│   └── easy2.txt
└── Cargo.toml
```

## Current Limitations (MVP)

- Only supports 9x9 boards (hardcoded)
- Limited to basic strategies (naked singles, hidden singles)
- No JSON strategy loading yet
- No logging system
- No speculative execution for hard puzzles
- No advanced strategies (X-Wing, Swordfish, etc.)

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
