//! Logging module for the Sudoku Solver
//!
//! This module provides logging utilities with timestamps and performance measurement.

use std::time::Instant;
use std::fmt;

/// Performance timer for measuring operation duration
pub struct Timer {
    start: Instant,
    label: String,
}

impl Timer {
    /// Creates a new timer with a label
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        log::trace!("Timer started: {}", label);
        Self {
            start: Instant::now(),
            label,
        }
    }
    
    /// Gets the elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
    
    /// Gets the elapsed time in microseconds
    pub fn elapsed_us(&self) -> u128 {
        self.start.elapsed().as_micros()
    }
    
    /// Logs the elapsed time at INFO level
    pub fn log_elapsed(&self) {
        let elapsed = self.elapsed_ms();
        if elapsed > 1000 {
            log::info!("{} completed in {:.2}s", self.label, elapsed as f64 / 1000.0);
        } else if elapsed > 0 {
            log::info!("{} completed in {}ms", self.label, elapsed);
        } else {
            log::info!("{} completed in {}μs", self.label, self.elapsed_us());
        }
    }
    
    /// Logs the elapsed time at DEBUG level
    pub fn log_elapsed_debug(&self) {
        let elapsed = self.elapsed_ms();
        if elapsed > 1000 {
            log::debug!("{} took {:.2}s", self.label, elapsed as f64 / 1000.0);
        } else if elapsed > 0 {
            log::debug!("{} took {}ms", self.label, elapsed);
        } else {
            log::debug!("{} took {}μs", self.label, self.elapsed_us());
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.log_elapsed_debug();
    }
}

/// Statistics tracker for solver operations
#[derive(Debug, Clone, Default)]
pub struct SolverStats {
    pub iterations: usize,
    pub cells_solved: usize,
    pub strategies_applied: usize,
    pub backtracks: usize,
    pub constraint_propagations: usize,
}

impl SolverStats {
    /// Creates a new statistics tracker
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Logs the current statistics
    pub fn log_stats(&self) {
        log::info!("Solver Statistics:");
        log::info!("  Iterations: {}", self.iterations);
        log::info!("  Cells solved: {}", self.cells_solved);
        log::info!("  Strategies applied: {}", self.strategies_applied);
        log::info!("  Backtracks: {}", self.backtracks);
        log::info!("  Constraint propagations: {}", self.constraint_propagations);
    }
}

impl fmt::Display for SolverStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "iterations={}, cells_solved={}, strategies={}, backtracks={}, propagations={}",
            self.iterations,
            self.cells_solved,
            self.strategies_applied,
            self.backtracks,
            self.constraint_propagations
        )
    }
}

/// Initializes the logging system with custom format
pub fn init_logger() {
    env_logger::Builder::from_default_env()
        .format_timestamp_millis()
        .init();
}

/// Initializes the logging system with a specific log level
pub fn init_logger_with_level(level: log::LevelFilter) {
    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp_millis()
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timer_creation() {
        let timer = Timer::new("test");
        assert!(timer.elapsed_us() >= 0);
    }
    
    #[test]
    fn test_stats_creation() {
        let stats = SolverStats::new();
        assert_eq!(stats.iterations, 0);
        assert_eq!(stats.cells_solved, 0);
    }
}
