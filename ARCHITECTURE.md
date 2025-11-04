# Sudoku Solver v2 - Architecture Document

## Overview
A high-performance, modular Sudoku solver written in Rust that uses constraint propagation with a pluggable strategy system, supporting multiple board sizes and featuring comprehensive logging for debugging and machine learning applications.

---

## Core Principles
1. **Modularity**: Clear separation of concerns with well-defined interfaces
2. **Performance**: Zero-cost abstractions, efficient data structures, toggleable logging
3. **Flexibility**: Support for various board sizes and custom strategies
4. **Extensibility**: Easy to add new solving strategies via JSON configuration
5. **Observability**: Comprehensive logging for debugging and ML training

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Interface                        │
│                    (main.rs, cli module)                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                      Solver Engine                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Constraint  │  │   Strategy   │  │  Speculative │      │
│  │ Propagation  │◄─┤   Selector   │  │   Executor   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────┬────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────┐
│                      Core Components                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │    Board     │  │   Strategy   │  │   Logger     │      │
│  │    State     │  │     Bank     │  │   System     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────┐
│                    Output & Visualization                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Solution   │  │  Event Stream│  │  Visualizer  │      │
│  │   Formatter  │  │   (stdout)   │  │  (external)  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Breakdown

### 1. **Board Module** (`src/board/`)

#### 1.1 Board State (`board/state.rs`)
```rust
pub struct Board {
    dimensions: BoardDimensions,
    cells: Vec<Cell>,
    solved_mask: BitBoard,
    constraint_graph: ConstraintGraph,
    
    // Pre-computed views for efficient access
    rows: Vec<Vec<usize>>,      // Cell indices for each row
    columns: Vec<Vec<usize>>,   // Cell indices for each column
    boxes: Vec<Vec<usize>>,     // Cell indices for each box
}

pub struct BoardDimensions {
    size: usize,              // e.g., 9 for 9x9
    subgrid_rows: usize,      // e.g., 3 for 3x3 subgrids
    subgrid_cols: usize,      // e.g., 3 for 3x3 subgrids
}

pub struct Cell {
    index: usize,
    candidates: CandidateSet,  // Bitset of possible values
    value: Option<u8>,
}
```

**Responsibilities:**
- Maintain current board state
- Track cell candidates (possible values)
- **Provide zero-cost view abstractions** for rows, columns, and boxes
- **Enable direct access** to constraint groups without iteration
- Support multiple board dimensions
- Maintain solved/unsolved bitmask for performance
- Pre-compute and cache cell groupings for O(1) access

#### 1.2 Candidate Set (`board/candidates.rs`)
```rust
pub struct CandidateSet {
    bits: u16,  // Bitset for values 1-9 (or more for larger boards)
}
```

**Responsibilities:**
- Efficient storage and manipulation of candidate values
- Set operations (union, intersection, difference)
- Count remaining candidates
- Convert to/from value representations

#### 1.3 Constraint Graph & View Abstractions (`board/constraints.rs`)
```rust
pub struct ConstraintGraph {
    peers: Vec<Vec<usize>>,  // For each cell, list of constrained cells
}

// Zero-cost abstraction: Views hold references to cells without copying
pub struct RowView<'a> {
    cells: Vec<&'a Cell>,
    index: usize,
}

pub struct ColumnView<'a> {
    cells: Vec<&'a Cell>,
    index: usize,
}

pub struct BoxView<'a> {
    cells: Vec<&'a Cell>,
    index: usize,
}

pub struct CellViews<'a> {
    row: RowView<'a>,
    column: ColumnView<'a>,
    box_view: BoxView<'a>,
}
```

**Responsibilities:**
- Pre-compute constraint relationships
- Quick lookup of affected cells
- Support different board topologies
- **Provide zero-cost view abstractions** that hold references to cells
- Enable efficient pattern checking without iterating entire board
- Cross-reference cells through multiple constraint dimensions simultaneously

**Key Design Pattern - View Abstractions:**

Instead of iterating over the entire board, we create lightweight view objects that hold pointers/references to relevant cells. This enables:
- **Direct access** to all cells in a row, column, or box without searching
- **Cross-referencing** - given a cell, instantly access all its constraint views
- **Pattern matching** - check constraints across multiple views simultaneously
- **Zero-cost** - Rust's lifetime system ensures these are compile-time abstractions with no runtime overhead
- **Efficient constraint propagation** - when a cell changes, directly access all affected cells through views

Example usage: To check if a naked pair exists in a row, we get the RowView, which already contains references to all 9 cells in that row, eliminating the need to search through all 81 cells.

---

### 2. **Strategy Module** (`src/strategy/`)

#### 2.1 Strategy Bank (`strategy/bank.rs`)
```rust
pub struct StrategyBank {
    strategies: Vec<Strategy>,
    strategy_cache: HashMap<String, usize>,
}

pub struct Strategy {
    metadata: StrategyMetadata,
    pattern: StrategyPattern,
    action: StrategyAction,
}
```

**Responsibilities:**
- Load strategy JSON files from directory
- Parse and validate strategy definitions
- Maintain strategy registry
- Filter strategies by board dimensions
- Cache compiled patterns for performance

#### 2.2 Strategy Definition (JSON Format)
```json
{
  "name": "naked_pair",
  "difficulty": 2,
  "description": "Eliminates candidates when two cells in a unit have the same two candidates",
  "applicable_dimensions": ["9x9", "6x6", "16x16"],
  "pattern": {
    "type": "cell_group",
    "unit_type": ["row", "column", "box"],
    "conditions": [
      {
        "cell_count": 2,
        "candidate_count": 2,
        "same_candidates": true
      }
    ]
  },
  "action": {
    "type": "eliminate_candidates",
    "target": "other_cells_in_unit",
    "candidates": "matched_candidates"
  },
  "priority": 50
}
```

#### 2.3 Pattern Matcher (`strategy/matcher.rs`)
```rust
pub trait PatternMatcher {
    fn matches(&self, board: &Board, context: &MatchContext) -> Vec<Match>;
}

// Example: Instead of iterating all cells, use views
impl PatternMatcher for NakedPairMatcher {
    fn matches(&self, board: &Board, context: &MatchContext) -> Vec<Match> {
        let mut matches = Vec::new();
        
        // Check each row view (only 9 iterations for 9x9 board)
        for row_view in board.row_views() {
            // row_view already contains references to all cells in this row
            if let Some(pair) = self.find_naked_pair_in_view(row_view) {
                matches.push(pair);
            }
        }
        
        // Similarly for columns and boxes
        // Total: 27 view checks instead of 81+ cell iterations
        matches
    }
}
```

**Responsibilities:**
- Evaluate strategy preconditions
- Find pattern matches in current board state
- **Leverage view abstractions** to avoid full board iteration
- **Cross-reference views** to check multiple constraint dimensions
- Return applicable strategy instances with context
- Optimize pattern detection using pre-computed cell groupings

#### 2.4 Strategy Selector (`strategy/selector.rs`)
```rust
pub struct StrategySelector {
    selection_policy: SelectionPolicy,
    statistics: StrategyStatistics,
}

pub enum SelectionPolicy {
    Priority,           // Use strategy priority
    MostConstrained,    // Target most constrained cells
    MLBased,           // Use learned weights
}
```

**Responsibilities:**
- Choose which strategy to apply next
- Track strategy effectiveness
- Support ML-based selection
- Prioritize based on board state analysis

---

### 3. **Solver Module** (`src/solver/`)

#### 3.1 Solver Engine (`solver/engine.rs`)
```rust
pub struct SolverEngine {
    board: Board,
    strategy_bank: StrategyBank,
    logger: Logger,
    config: SolverConfig,
}

pub struct SolverConfig {
    max_iterations: usize,
    enable_logging: bool,
    enable_profiling: bool,
    difficulty_level: DifficultyLevel,
}
```

**Responsibilities:**
- Orchestrate solving process
- Apply constraint propagation
- Invoke strategy selector
- Handle speculative execution
- Manage solving iterations

#### 3.2 Constraint Propagator (`solver/propagator.rs`)
```rust
pub struct ConstraintPropagator {
    propagation_queue: VecDeque<CellUpdate>,
}

pub struct CellUpdate {
    cell_index: usize,
    eliminated_candidates: CandidateSet,
}

// Example: Efficient propagation using views
impl ConstraintPropagator {
    fn propagate_cell_update(&mut self, board: &mut Board, cell_idx: usize) {
        // Get all constraint views for this cell (zero-cost)
        let views = board.get_cell_views(cell_idx);
        
        // Directly access all affected cells through views
        // No need to search through entire board
        for &peer_idx in views.all_peers() {
            // Update peer candidates
            self.update_peer(board, peer_idx);
        }
    }
}
```

**Responsibilities:**
- Propagate constraint changes through board
- **Use view abstractions** to directly access affected cells
- Maintain propagation queue
- Detect contradictions early
- Update candidate sets efficiently
- **Avoid full board iteration** by leveraging pre-computed peer relationships

#### 3.3 Speculative Executor (`solver/speculative.rs`)
```rust
pub struct SpeculativeExecutor {
    max_depth: usize,
    max_branches: usize,
}

pub struct Speculation {
    board_snapshot: Board,
    assumption: CellAssignment,
    depth: usize,
}
```

**Responsibilities:**
- Handle stalemate situations
- Try multiple candidate assignments in parallel
- Prune invalid branches early
- Avoid traditional backtracking
- Return to valid state when contradictions found

---

### 4. **Logger Module** (`src/logger/`)

#### 4.1 Logger System (`logger/mod.rs`)
```rust
pub struct Logger {
    enabled: bool,
    output: LogOutput,
    level: LogLevel,
    events: Vec<SolveEvent>,
}

pub enum SolveEvent {
    StrategyApplied {
        strategy: String,
        timestamp: Instant,
        board_state: BoardSnapshot,
        changes: Vec<CellUpdate>,
    },
    CandidateEliminated {
        cell: usize,
        value: u8,
        reason: String,
    },
    CellSolved {
        cell: usize,
        value: u8,
    },
    SpeculationStarted {
        cell: usize,
        candidate: u8,
    },
    ContradictionDetected {
        reason: String,
    },
}
```

**Responsibilities:**
- Record solving steps
- Capture strategy applications
- Track performance metrics
- Support ML training data collection
- Toggleable for performance
- Output structured logs (JSON format)

#### 4.2 Performance Profiler (`logger/profiler.rs`)
```rust
pub struct Profiler {
    strategy_timings: HashMap<String, Duration>,
    iteration_count: usize,
    start_time: Instant,
}
```

**Responsibilities:**
- Measure strategy execution time
- Track iteration counts
- Identify bottlenecks
- Generate performance reports

---

### 5. **I/O Module** (`src/io/`)

#### 5.1 Puzzle Loader (`io/loader.rs`)
```rust
pub trait PuzzleLoader {
    fn load(&self, path: &Path) -> Result<Board, LoadError>;
}

pub struct JsonPuzzleLoader;
pub struct TextPuzzleLoader;
```

**Responsibilities:**
- Load puzzles from various formats
- Validate puzzle structure
- Support multiple board sizes
- Parse initial clues

#### 5.2 Solution Formatter (`io/formatter.rs`)
```rust
pub trait SolutionFormatter {
    fn format(&self, board: &Board, log: &SolveLog) -> String;
}

pub struct TextFormatter;
pub struct JsonFormatter;
```

**Responsibilities:**
- Format solved board for output
- Include solving steps if requested
- Support multiple output formats

#### 5.3 Event Stream (`io/stream.rs`)
```rust
pub struct EventStream {
    writer: Box<dyn Write>,
}
```

**Responsibilities:**
- Stream board state updates to stdout
- Enable real-time visualization
- Use structured format (JSON lines)
- Decouple solver from visualizer

---

### 6. **CLI Module** (`src/cli/`)

#### 6.1 Command Interface (`cli/mod.rs`)
```rust
pub struct CliApp {
    config: CliConfig,
}

pub struct CliConfig {
    puzzle_path: PathBuf,
    strategy_dir: PathBuf,
    output_format: OutputFormat,
    log_level: LogLevel,
    enable_profiling: bool,
    enable_visualization: bool,
}
```

**Responsibilities:**
- Parse command-line arguments
- Configure solver engine
- Handle user input
- Display results
- Manage application lifecycle

---

## Data Flow

### Solving Process Flow
```
1. Load Puzzle
   ↓
2. Initialize Board State
   ↓
3. Load Strategy Bank
   ↓
4. Main Solving Loop:
   ├─→ Constraint Propagation
   │   ├─→ Update candidates
   │   ├─→ Detect solved cells
   │   └─→ Check for contradictions
   │
   ├─→ Strategy Selection
   │   ├─→ Analyze board state
   │   ├─→ Find most constrained cells
   │   └─→ Select applicable strategy
   │
   ├─→ Apply Strategy
   │   ├─→ Pattern matching
   │   ├─→ Execute strategy action
   │   └─→ Log changes
   │
   ├─→ Check Progress
   │   ├─→ If solved → Exit
   │   ├─→ If progress → Continue loop
   │   └─→ If stalemate → Speculative Execution
   │
   └─→ Speculative Execution (if needed)
       ├─→ Choose candidate to try
       ├─→ Create board snapshot
       ├─→ Apply assumption
       ├─→ Solve recursively
       └─→ Prune invalid branches
   ↓
5. Output Solution & Logs
```

---

## Key Design Decisions

### 1. **View Abstractions for Zero-Cost Constraint Access**
- **Leverage Rust's zero-cost abstractions** - Views are compile-time constructs with no runtime overhead
- **Pre-compute cell groupings** - Build row, column, and box indices once during initialization
- **Direct access patterns** - Instead of iterating over all cells, access only relevant cells through views
- **Cross-referencing** - Given a cell, instantly get all its constraint views (row, column, box)
- **Efficient pattern matching** - Check constraints across views without walking entire board
- **Example**: To find a naked pair in a row, access the RowView (9 cells) instead of checking all 81 cells
- **Memory layout** - Views hold indices/references, not copies, maintaining cache efficiency

### 2. **Bitset Representation**
- Use `u16` or `u32` bitsets for candidate tracking
- Enables fast set operations
- Memory efficient
- Easy to check if cell is solved (popcount == 1)

### 3. **Strategy as Data**
- JSON-based strategy definitions
- No recompilation needed for new strategies
- Easy to share and version strategies
- Pattern matching engine interprets definitions

### 4. **Speculative Execution over Backtracking**
- More cache-friendly
- Can explore multiple branches in parallel
- Early pruning of invalid states
- Better for modern CPU architectures

### 5. **Decoupled Visualization**
- Solver outputs event stream to stdout
- External visualizer reads stream
- Clean separation of concerns
- Visualizer can be written in any language

### 6. **Toggleable Logging**
- Use feature flags or runtime config
- Zero-cost when disabled (compiler optimization)
- Structured logging for ML training
- Performance profiling separate from debug logging

### 7. **Dimension-Agnostic Core**
- Board dimensions parameterized
- Constraint graph adapts to topology
- Strategies declare compatible dimensions
- Validation at load time

---

## Technology Stack

### Core Dependencies
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.0", features = ["derive"] }  # CLI parsing
anyhow = "1.0"  # Error handling
thiserror = "1.0"  # Custom errors
log = "0.4"  # Logging facade
env_logger = "0.11"  # Logger implementation
rayon = "1.8"  # Parallel iteration (for speculative execution)

[dev-dependencies]
criterion = "0.5"  # Benchmarking
proptest = "1.0"  # Property-based testing
```

---

## Directory Structure

```
sudoku-solver-v2/
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library root
│   │
│   ├── board/
│   │   ├── mod.rs
│   │   ├── state.rs            # Board representation
│   │   ├── candidates.rs       # Candidate set operations
│   │   ├── constraints.rs      # Constraint graph & views
│   │   └── dimensions.rs       # Board dimension handling
│   │
│   ├── strategy/
│   │   ├── mod.rs
│   │   ├── bank.rs             # Strategy registry
│   │   ├── matcher.rs          # Pattern matching
│   │   ├── selector.rs         # Strategy selection
│   │   └── types.rs            # Strategy types
│   │
│   ├── solver/
│   │   ├── mod.rs
│   │   ├── engine.rs           # Main solver
│   │   ├── propagator.rs       # Constraint propagation
│   │   └── speculative.rs      # Speculative execution
│   │
│   ├── logger/
│   │   ├── mod.rs
│   │   ├── events.rs           # Event definitions
│   │   └── profiler.rs         # Performance profiling
│   │
│   ├── io/
│   │   ├── mod.rs
│   │   ├── loader.rs           # Puzzle loading
│   │   ├── formatter.rs        # Output formatting
│   │   └── stream.rs           # Event streaming
│   │
│   └── cli/
│       ├── mod.rs
│       └── config.rs           # CLI configuration
│
├── strategies/                 # Strategy JSON files
│   ├── basic/
│   │   ├── naked_single.json
│   │   └── hidden_single.json
│   ├── intermediate/
│   │   ├── naked_pair.json
│   │   ├── hidden_pair.json
│   │   └── pointing_pair.json
│   └── advanced/
│       ├── x_wing.json
│       ├── swordfish.json
│       └── xy_wing.json
│
├── puzzles/                    # Example puzzles
│   ├── easy/
│   ├── medium/
│   └── hard/
│
├── tests/
│   ├── integration/
│   └── unit/
│
├── benches/                    # Performance benchmarks
│   └── solver_bench.rs
│
├── Cargo.toml
├── ARCHITECTURE.md             # This file
└── README.md
```

---

## Performance Optimizations

### 1. **Bitwise Operations**
- Use bitsets for candidate tracking
- Fast union/intersection operations
- Efficient solved cell detection

### 2. **View Abstractions & Constraint Graph Pre-computation**
- **Zero-cost view abstractions** - Use Rust's lifetime system to create views that hold references to cells
- **Pre-computed views** - Build row, column, and box views once during initialization
- **Direct cell access** - No need to iterate entire board to find relevant cells
- **Cross-referencing** - Given a cell, instantly access all its constraint views (row, column, box)
- **O(1) lookup** for affected cells through pre-computed peer relationships
- **Pattern matching optimization** - Check constraints across views without walking every cell
- **Cache-friendly** - Views group related cells together for better memory locality

### 3. **Strategy Caching**
- Cache compiled pattern matchers
- Reuse strategy instances
- Avoid repeated JSON parsing

### 4. **Lazy Evaluation**
- Only compute what's needed
- Defer expensive operations
- Early exit on contradictions

### 5. **Parallel Speculative Execution**
- Use Rayon for parallel branches
- Explore multiple assumptions simultaneously
- CPU-bound workload benefits from parallelism

### 6. **Memory Layout**
- Contiguous cell storage (Vec)
- Cache-friendly data structures
- Minimize pointer chasing

---

## Testing Strategy

### Unit Tests
- Board operations
- Candidate set manipulation
- Strategy pattern matching
- Constraint propagation
- View abstraction correctness

### Integration Tests
- End-to-end solving
- Strategy application
- Multiple board sizes
- Edge cases and contradictions

### Property-Based Tests
- Board invariants
- Strategy correctness
- Dimension handling
- View consistency

### Benchmarks
- Solving speed by difficulty
- Strategy performance
- Memory usage
- Parallel vs sequential
- View abstraction overhead (should be zero)

---

## Future Enhancements

1. **Machine Learning Integration**
   - Train strategy selector on historical data
   - Learn optimal strategy sequences
   - Predict solving difficulty

2. **Puzzle Generation**
   - Generate puzzles of specific difficulty
   - Ensure unique solutions
   - Control strategy requirements

3. **Web Interface**
   - WebAssembly compilation
   - Interactive visualization
   - Real-time solving display

4. **Distributed Solving**
   - Distribute speculative branches
   - Network-based parallelism
   - Handle very large boards

5. **Strategy Synthesis**
   - Automatically discover new strategies
   - Genetic programming approach
   - Optimize strategy combinations

---

## Conclusion

This architecture provides a solid foundation for a high-performance, extensible Sudoku solver. The modular design allows for easy testing, maintenance, and future enhancements. The use of Rust ensures memory safety and performance, while the strategy bank system provides flexibility for adding new solving techniques without code changes.

The key innovations are:
- **View abstractions for zero-cost constraint access** - Leverage Rust's lifetime system to avoid board iteration
- **Pluggable strategy system** via JSON configuration
- **Speculative execution** instead of traditional backtracking
- **Dimension-agnostic design** supporting various board sizes
- **Comprehensive logging** for debugging and ML training
- **Decoupled visualization** through event streaming

This design balances performance, flexibility, and maintainability, making it suitable for both research and practical applications.
