# Logging System Documentation

## Overview

The Sudoku Solver v2 includes a comprehensive logging system that provides detailed insights into the solving process, strategy application, and performance metrics. The logging system uses the `log` and `env_logger` crates with timestamp support via `chrono`.

## Features

- **Multiple Log Levels**: OFF, ERROR, WARN, INFO, DEBUG, TRACE
- **Timestamps**: Millisecond-precision timestamps for all log messages
- **Performance Tracking**: Built-in timer utilities for measuring operation duration
- **Statistics Tracking**: Comprehensive solver statistics (iterations, strategies applied, backtracks, etc.)
- **Flexible Configuration**: Configure via CLI arguments or environment variables

## Log Levels

### OFF
No logging output.

### ERROR
Critical errors that prevent solving or indicate serious problems.
- Board validation failures
- Strategy loading errors
- Contradictions detected

### WARN
Warnings about potential issues or fallback behaviors.
- Failed strategy loading (with fallback)
- Partial solutions
- Basic solver limitations

### INFO (Default)
High-level progress information suitable for end users.
- Solver initialization
- Strategy loading summary
- Iteration progress
- Solution status
- Performance statistics

### DEBUG
Detailed information useful for debugging.
- Strategy loading details
- Individual strategy applications
- Cell solving via propagation
- Board state changes
- Performance timings

### TRACE
Very detailed information for deep debugging.
- Individual candidate removals
- Backtracking attempts
- Directory scanning
- File loading
- Pattern matching details

## Usage

### Command Line

```bash
# Using --log-level flag
cargo run -- solve puzzle.txt --log-level debug

# Using -l shorthand
cargo run -- solve puzzle.txt -l trace

# Default (INFO level)
cargo run -- solve puzzle.txt
```

### Environment Variable

```bash
# Set log level via environment variable
RUST_LOG=debug cargo run -- solve puzzle.txt

# Trace level for maximum detail
RUST_LOG=trace cargo run -- solve puzzle.txt
```

## Example Output

### INFO Level
```
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2] Sudoku Solver v2 - Advanced Strategy System
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2] Loaded puzzle from file: puzzles/easy1.txt
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::solver] Initializing solver with strategy system
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::strategy::bank] Successfully loaded 7 strategies
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::solver] Starting solve process
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::solver] After initial propagation: 30/81 cells solved
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::solver] Iteration 1: 81/81 cells solved
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::solver] Solve complete: 81/81 cells solved
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::logging] Solver Statistics:
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::logging]   Iterations: 1
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::logging]   Cells solved: 81
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::logging]   Strategies applied: 51
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::logging]   Backtracks: 0
[2025-12-01T17:53:20.722Z INFO  sudoku_solver_v2::logging]   Constraint propagations: 51
```

### DEBUG Level
```
[2025-12-01T17:53:28.070Z DEBUG sudoku_solver_v2::strategy::bank] Loading strategies from directory: strategies
[2025-12-01T17:53:28.070Z DEBUG sudoku_solver_v2::strategy::bank] Added strategy: pointing_pair (priority: 60, difficulty: 4)
[2025-12-01T17:53:28.070Z DEBUG sudoku_solver_v2::strategy::bank] Added strategy: naked_pair (priority: 70, difficulty: 3)
[2025-12-01T17:53:28.071Z DEBUG sudoku_solver_v2::strategy::bank] Added strategy: naked_single (priority: 100, difficulty: 1)
[2025-12-01T17:53:28.071Z DEBUG sudoku_solver_v2::logging] Loading strategies took 966μs
[2025-12-01T17:53:28.071Z DEBUG sudoku_solver_v2::solver] Initial board state: 30/81 cells solved
[2025-12-01T17:53:28.071Z DEBUG sudoku_solver_v2::solver] Propagating initial constraints
[2025-12-01T17:53:28.071Z DEBUG sudoku_solver_v2::solver] Cell 40 solved with value 5 via propagation
[2025-12-01T17:53:28.071Z DEBUG sudoku_solver_v2::solver] Cell 59 solved with value 7 via propagation
[2025-12-01T17:53:28.071Z DEBUG sudoku_solver_v2::solver] Starting iteration 1
[2025-12-01T17:53:28.071Z DEBUG sudoku_solver_v2::solver] Applying strategy: naked_single (priority: 100)
```

### TRACE Level
```
[2025-12-01T17:53:35.123Z TRACE sudoku_solver_v2::strategy::bank] Scanning directory: strategies
[2025-12-01T17:53:35.123Z TRACE sudoku_solver_v2::strategy::bank] Loading strategy file: strategies/basic/naked_single.json
[2025-12-01T17:53:35.123Z TRACE sudoku_solver_v2::strategy::bank] Successfully loaded strategy from strategies/basic/naked_single.json
[2025-12-01T17:53:35.124Z TRACE sudoku_solver_v2::solver] Propagating initial constraints
[2025-12-01T17:53:35.124Z TRACE sudoku_solver_v2::solver] Removed candidate 5 from cell 41
[2025-12-01T17:53:35.124Z TRACE sudoku_solver_v2::solver] Removed candidate 5 from cell 42
[2025-12-01T17:53:35.124Z TRACE sudoku_solver_v2::solver] Backtracking attempt #1
[2025-12-01T17:53:35.124Z TRACE sudoku_solver_v2::solver] Trying cell 23 with 2 candidates
[2025-12-01T17:53:35.124Z TRACE sudoku_solver_v2::solver] Trying value 3 at cell 23
```

## Performance Tracking

The logging system includes built-in performance tracking:

### Timer Utility
```rust
use sudoku_solver_v2::Timer;

let timer = Timer::new("Operation name");
// ... do work ...
timer.log_elapsed(); // Logs at INFO level
// Or timer.log_elapsed_debug(); for DEBUG level
```

### Automatic Timing
The solver automatically tracks and logs:
- Strategy loading time
- Total solve time
- Individual iteration times (at DEBUG level)

### Statistics Tracking
The `SolverStats` struct tracks:
- Number of iterations
- Cells solved
- Strategies applied
- Backtracking attempts
- Constraint propagations

## Integration Points

### Solver Module (`src/solver/mod.rs`)
- Solver initialization
- Iteration progress
- Strategy application
- Backtracking decisions
- Constraint propagation
- Solution status

### Strategy Bank (`src/strategy/bank.rs`)
- Directory scanning
- Strategy file loading
- Strategy validation
- Strategy addition

### Main Application (`src/main.rs`)
- Application startup
- Puzzle loading
- Solve process initiation
- Final results

## Best Practices

1. **Development**: Use DEBUG or TRACE level for detailed debugging
2. **Production**: Use INFO level for normal operation
3. **Performance Testing**: Use INFO level to see timing statistics
4. **Troubleshooting**: Use DEBUG level to see strategy applications
5. **Deep Debugging**: Use TRACE level to see every operation

## Performance Impact

- **OFF/ERROR/WARN**: Negligible impact
- **INFO**: Minimal impact (< 1% overhead)
- **DEBUG**: Low impact (< 5% overhead)
- **TRACE**: Moderate impact (5-15% overhead due to volume of messages)

## Future Enhancements

Potential improvements to the logging system:
- File output support
- Structured logging (JSON format)
- Log rotation
- Per-module log level configuration
- Performance profiling integration
- Visualization tools for log analysis
