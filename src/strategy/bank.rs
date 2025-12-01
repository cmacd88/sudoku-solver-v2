//! Strategy bank for loading and managing strategies from JSON files.
//!
//! The StrategyBank loads strategy definitions from a directory of JSON files,
//! validates them, and provides access to strategies for the solver.

use super::types::Strategy;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when loading strategies
#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("Failed to read strategy directory: {0}")]
    DirectoryReadError(String),
    
    #[error("Failed to read strategy file {path}: {error}")]
    FileReadError { path: String, error: String },
    
    #[error("Failed to parse strategy JSON in {path}: {error}")]
    JsonParseError { path: String, error: String },
    
    #[error("Invalid strategy: {0}")]
    ValidationError(String),
    
    #[error("Strategy not found: {0}")]
    StrategyNotFound(String),
}

/// A bank of strategies loaded from JSON files
#[derive(Debug)]
pub struct StrategyBank {
    /// All loaded strategies
    strategies: Vec<Strategy>,
    
    /// Index by strategy name for quick lookup
    strategy_index: HashMap<String, usize>,
    
    /// Strategies grouped by difficulty
    by_difficulty: HashMap<u32, Vec<usize>>,
}

impl StrategyBank {
    /// Creates a new empty strategy bank
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
            strategy_index: HashMap::new(),
            by_difficulty: HashMap::new(),
        }
    }
    
    /// Loads strategies from a directory
    /// 
    /// Recursively scans the directory for .json files and loads them as strategies.
    /// Invalid files are logged but don't stop the loading process.
    pub fn load_from_directory<P: AsRef<Path>>(path: P) -> Result<Self, StrategyError> {
        let mut bank = Self::new();
        
        let path = path.as_ref();
        if !path.exists() {
            return Err(StrategyError::DirectoryReadError(
                format!("Directory does not exist: {}", path.display())
            ));
        }
        
        if !path.is_dir() {
            return Err(StrategyError::DirectoryReadError(
                format!("Path is not a directory: {}", path.display())
            ));
        }
        
        bank.load_directory_recursive(path)?;
        
        Ok(bank)
    }
    
    /// Recursively loads strategies from a directory
    fn load_directory_recursive(&mut self, path: &Path) -> Result<(), StrategyError> {
        let entries = fs::read_dir(path)
            .map_err(|e| StrategyError::DirectoryReadError(e.to_string()))?;
        
        for entry in entries {
            let entry = entry
                .map_err(|e| StrategyError::DirectoryReadError(e.to_string()))?;
            
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively load subdirectories
                self.load_directory_recursive(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Load JSON files
                match self.load_from_file(&path) {
                    Ok(_) => {
                        // Successfully loaded
                    }
                    Err(e) => {
                        // Log error but continue loading other files
                        eprintln!("Warning: Failed to load strategy from {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Loads a single strategy from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), StrategyError> {
        let path = path.as_ref();
        let path_str = path.display().to_string();
        
        // Read file contents
        let contents = fs::read_to_string(path)
            .map_err(|e| StrategyError::FileReadError {
                path: path_str.clone(),
                error: e.to_string(),
            })?;
        
        // Parse JSON
        let strategy: Strategy = serde_json::from_str(&contents)
            .map_err(|e| StrategyError::JsonParseError {
                path: path_str.clone(),
                error: e.to_string(),
            })?;
        
        // Validate strategy
        self.validate_strategy(&strategy)?;
        
        // Add to bank
        self.add_strategy(strategy);
        
        Ok(())
    }
    
    /// Validates a strategy definition
    fn validate_strategy(&self, strategy: &Strategy) -> Result<(), StrategyError> {
        // Check that name is not empty
        if strategy.metadata.name.is_empty() {
            return Err(StrategyError::ValidationError(
                "Strategy name cannot be empty".to_string()
            ));
        }
        
        // Check that name is unique
        if self.strategy_index.contains_key(&strategy.metadata.name) {
            return Err(StrategyError::ValidationError(
                format!("Duplicate strategy name: {}", strategy.metadata.name)
            ));
        }
        
        // Check that difficulty is reasonable
        if strategy.metadata.difficulty == 0 || strategy.metadata.difficulty > 100 {
            return Err(StrategyError::ValidationError(
                format!("Strategy difficulty must be between 1 and 100, got {}", strategy.metadata.difficulty)
            ));
        }
        
        Ok(())
    }
    
    /// Adds a strategy to the bank
    fn add_strategy(&mut self, strategy: Strategy) {
        let index = self.strategies.len();
        let name = strategy.metadata.name.clone();
        let difficulty = strategy.metadata.difficulty;
        
        self.strategies.push(strategy);
        self.strategy_index.insert(name, index);
        
        self.by_difficulty
            .entry(difficulty)
            .or_insert_with(Vec::new)
            .push(index);
    }
    
    /// Gets a strategy by name
    pub fn get_strategy(&self, name: &str) -> Option<&Strategy> {
        self.strategy_index
            .get(name)
            .and_then(|&idx| self.strategies.get(idx))
    }
    
    /// Gets all strategies
    pub fn get_all_strategies(&self) -> &[Strategy] {
        &self.strategies
    }
    
    /// Gets strategies by difficulty level
    pub fn get_strategies_by_difficulty(&self, difficulty: u32) -> Vec<&Strategy> {
        self.by_difficulty
            .get(&difficulty)
            .map(|indices| {
                indices.iter()
                    .filter_map(|&idx| self.strategies.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Gets strategies up to a certain difficulty level
    pub fn get_strategies_up_to_difficulty(&self, max_difficulty: u32) -> Vec<&Strategy> {
        self.strategies
            .iter()
            .filter(|s| s.metadata.difficulty <= max_difficulty)
            .collect()
    }
    
    /// Gets strategies sorted by priority (highest first)
    pub fn get_strategies_by_priority(&self) -> Vec<&Strategy> {
        let mut strategies: Vec<&Strategy> = self.strategies.iter().collect();
        strategies.sort_by(|a, b| b.priority.cmp(&a.priority));
        strategies
    }
    
    /// Filters strategies by board dimensions
    pub fn get_strategies_for_dimensions(&self, dimensions: &str) -> Vec<&Strategy> {
        self.strategies
            .iter()
            .filter(|s| s.metadata.applicable_dimensions.contains(&dimensions.to_string()))
            .collect()
    }
    
    /// Returns the number of loaded strategies
    pub fn len(&self) -> usize {
        self.strategies.len()
    }
    
    /// Checks if the bank is empty
    pub fn is_empty(&self) -> bool {
        self.strategies.is_empty()
    }
}

impl Default for StrategyBank {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_strategy_json() -> String {
        r#"{
            "name": "test_strategy",
            "difficulty": 1,
            "description": "A test strategy",
            "applicable_dimensions": ["9x9"],
            "pattern": {
                "type": "single_cell",
                "conditions": [
                    {"type": "single_candidate"}
                ]
            },
            "action": {
                "type": "set_cell_value",
                "target": "matched_cells",
                "value": "single_candidate"
            },
            "priority": 50
        }"#.to_string()
    }

    #[test]
    fn test_empty_bank() {
        let bank = StrategyBank::new();
        assert_eq!(bank.len(), 0);
        assert!(bank.is_empty());
    }

    #[test]
    fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(create_test_strategy_json().as_bytes()).unwrap();
        
        let mut bank = StrategyBank::new();
        let result = bank.load_from_file(&file_path);
        
        assert!(result.is_ok());
        assert_eq!(bank.len(), 1);
        assert!(bank.get_strategy("test_strategy").is_some());
    }

    #[test]
    fn test_load_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a strategy file
        let file_path = temp_dir.path().join("test.json");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(create_test_strategy_json().as_bytes()).unwrap();
        
        let bank = StrategyBank::load_from_directory(temp_dir.path());
        
        assert!(bank.is_ok());
        let bank = bank.unwrap();
        assert_eq!(bank.len(), 1);
    }

    #[test]
    fn test_get_strategies_by_difficulty() {
        let mut bank = StrategyBank::new();
        
        // We can't easily test this without creating actual strategy objects
        // This is a placeholder for when we have more strategies
        assert_eq!(bank.get_strategies_by_difficulty(1).len(), 0);
    }

    #[test]
    fn test_validation_empty_name() {
        let bank = StrategyBank::new();
        let mut strategy: Strategy = serde_json::from_str(&create_test_strategy_json()).unwrap();
        strategy.metadata.name = String::new();
        
        let result = bank.validate_strategy(&strategy);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_invalid_difficulty() {
        let bank = StrategyBank::new();
        let mut strategy: Strategy = serde_json::from_str(&create_test_strategy_json()).unwrap();
        strategy.metadata.difficulty = 0;
        
        let result = bank.validate_strategy(&strategy);
        assert!(result.is_err());
    }
}
