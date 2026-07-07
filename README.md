# Sudoku Solver v2 - v1.0

A Sudoku solver written in Rust, using constraint propagation, a JSON-driven strategy system, and breadth-first speculative search for puzzles that need guessing.

**NB! I don't really know Rust as a language — this project is a way to test how well AI/LLMs can implement my own ideas.** It was largely vibe-coded, then debugged and redesigned with help from Claude after early versions silently produced invalid solutions on hard puzzles.
I started off with the idea of how the solver should work, but lacking the fluency in Rust to implement it.
First drafting a crude design document, having AI iterate on it, then start writing code based on the design doc.
First I used blackbox ai, but when that got stuck and burned through credits, then helped itself to to more by charging my card without asking me, I put a stop to that. Claude has helped me with the rest.

## Features

- **View Abstractions**: O(1) access to rows/columns/boxes via pre-computed indices
- **Constraint Propagation**: Candidate elimination via peer views
- **Bitset Candidates**: Fast set operations using bitwise ops (u16 per cell)
- **JSON Strategy System**: Strategies defined in JSON, loaded at runtime, no code changes needed
- **Strategies**: Naked singles, hidden singles, naked pairs, pointing pairs, X-Wing, Swordfish, XY-Wing
- **Strategy Selection Policies**: Priority, Difficulty, FirstMatch
- **Speculative Solving**: When deterministic strategies stall, breadth-first search tries candidate branches in parallel (via Rayon), dropping contradictions and requiring no depth limit — bounded only by a total-node safety cap
- **Cascade Heuristic**: Speculation prefers cells with 2 candidates, then ranks by how many other cells a guess would cascade-solve
- **Correctness Checks**: `is_valid()` (no duplicate values among filled cells) and `is_complete()` (fully filled *and* valid) — the latter is what the solver and CLI actually use to report success
- **Tunable Logging**: Standard `log`/`env_logger`; set `RUST_LOG=info|debug|trace` for internal solving detail
- **CLI Verbosity**: `-v` / `-vv` flags control how much is printed, independent of `RUST_LOG`

## Known Limitations

- Only supports 9x9 boards (hardcoded)
- Test suite currently checks "made progress" / "board stays valid," not "matches a known correct solution" — a regression test comparing against a precomputed solved board is still on the to-do list
- No test yet locks in the deep-speculation case (30+ level guessing) as a permanent regression check

## Installation

```bash
git clone <repo-url>
cd sudoku-solver-v2
cargo build --release
```

## Usage

### Solve from a string or file
```bash
cargo run --release -- solve "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
cargo run --release -- solve puzzles/easy1.txt
```

### Puzzle format
81 characters, `0` or `.` for empty cells, `1`-`9` for clues. Whitespace ignored.

### Options
| Flag | Effect |
|---|---|
| `-v` | Show initial board, elapsed solve time |
| `-vv` | Also show strategy-loading detail and speculation statistics |
| `-s, --speculation-mode <mode>` | `sequential`, `parallel`, or `hybrid` (default: `hybrid`) |
| `-d, --speculation-depth <n>` | Depth cap for sequential mode (default: 100; the default `parallel`/`hybrid` search is depth-unlimited, bounded only by a total node cap) |
| `--no-speculation` | Disable speculation, use plain backtracking instead |
| `--no-stats` | Disable statistics tracking |

### Fine-grained internal logging
```bash
RUST_LOG=debug cargo run --release -- solve puzzle.txt
RUST_LOG=trace cargo run --release -- solve puzzle.txt   # per-branch detail
```

### Example (`-vv`, a puzzle requiring speculation)
```
Sudoku Solver v2 - Advanced Strategy System

✓ Board is valid (no contradictions)

Statistics:
  Solved cells: 27/81
  Unsolved cells: 54
  Completion: 33.3%


 Initial Board:
. . . | . . . | 5 8 . 
. 8 3 | . . 7 | . . . 
2 . 9 | . . . | . 7 1 
------+-------+------
5 1 4 | 9 . . | . . . 
. . . | . 1 . | . . . 
. . . | . . 8 | 4 1 2 
------+-------+------
6 3 . | . . . | 8 . 5 
. . . | 1 . . | 6 2 . 
. 4 2 | . . . | . . . 

Solving with advanced strategies...

✓ Loaded strategy system
✓ Speculation enabled (mode: Hybrid, depth: 100)

Speculation Statistics:
  Branches explored: 36
  Branches pruned: 3
  Max depth reached: 33
  Contradictions found: 0

✓ Puzzle solved successfully!

Time elapsed: 6.809571ms

Final Board:
4 6 7 | 3 2 1 | 5 8 9 
1 8 3 | 5 9 7 | 2 6 4 
2 5 9 | 8 4 6 | 3 7 1 
------+-------+------
5 1 4 | 9 6 2 | 7 3 8 
7 2 8 | 4 1 3 | 9 5 6 
3 9 6 | 7 5 8 | 4 1 2 
------+-------+------
6 3 1 | 2 7 9 | 8 4 5 
9 7 5 | 1 8 4 | 6 2 3 
8 4 2 | 6 3 5 | 1 9 7 

Statistics:
  Solved cells: 81/81
  Unsolved cells: 0
  Completion: 100.0%
```

## Testing

```bash
cargo test               # all tests
cargo test -- --nocapture
cargo test test_solve_easy_puzzle_1
```

## Project Structure

```
sudoku-solver-v2/
├── src/
│   ├── main.rs              # CLI application
│   ├── lib.rs                # Library root
│   ├── board/
│   │   ├── mod.rs            # Board, is_valid()/is_complete(), row/col/box validation
│   │   ├── candidates.rs     # Bitset candidate operations
│   │   └── views.rs          # Row/column/box view abstractions
│   ├── solver/
│   │   ├── mod.rs            # Constraint propagation, strategy iteration, backtracking fallback
│   │   └── speculative.rs    # Breadth-first speculation, cascade heuristic, statistics
│   ├── strategy/
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── bank.rs            # Loads strategy JSON files
│   │   ├── matcher.rs          # Pattern matchers (naked/hidden singles, pairs, X-Wing, Swordfish, XY-Wing)
│   │   └── selector.rs         # Strategy selection + application
│   └── io/
│       └── mod.rs
├── strategies/                 # JSON strategy definitions (basic/, intermediate/)
├── tests/
│   ├── integration_test.rs
│   ├── edge_cases_test.rs
│   ├── strategy_system_test.rs
│   ├── advanced_strategy_test.rs
│   └── speculation_test.rs
├── puzzles/
└── Cargo.toml
```

## How Speculation Works

1. Solve deterministically (naked/hidden singles, pairs, X-Wing, etc.) until stuck.
2. Pick a cell: prefer any with exactly 2 candidates; among candidates, rank by how many other cells a guess would cascade-solve (via a cheap simulate-and-propagate check).
3. Try each candidate value as a branch, in parallel, one full layer at a time (breadth-first, not per-branch recursion).
4. A branch that contradicts is dropped immediately, not carried forward.
5. Repeat from step 1 within each surviving branch until one reaches a complete, valid board, or all branches are exhausted.
6. No depth limit — the search naturally terminates because dead branches are pruned and the state space only shrinks. A large total-node cap exists purely as a runaway-bug safety valve, not a normal stopping condition.

## License

MIT