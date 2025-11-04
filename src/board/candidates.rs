//! Candidate set implementation using bitsets for efficient storage and operations.
//!
//! Each cell in a Sudoku puzzle can have multiple candidate values (1-9 for standard 9x9).
//! We use a bitset representation where each bit represents whether a value is a candidate.

use std::fmt;

/// A set of candidate values for a Sudoku cell, represented as a bitset.
/// For a 9x9 Sudoku, we use bits 1-9 (bit 0 is unused).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateSet {
    /// Bitset where bit i represents whether value i is a candidate
    /// Bit 0 is unused, bits 1-9 represent values 1-9
    bits: u16,
}

impl CandidateSet {
    /// Creates a new CandidateSet with all values 1-9 as candidates
    pub fn full() -> Self {
        // Set bits 1-9 (0b0000_0011_1111_1110 = 0x03FE)
        Self { bits: 0x03FE }
    }

    /// Creates an empty CandidateSet with no candidates
    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Creates a CandidateSet with a single value
    pub fn single(value: u8) -> Self {
        assert!((1..=9).contains(&value), "Value must be between 1 and 9");
        Self { bits: 1 << value }
    }

    /// Checks if a value is a candidate
    pub fn contains(&self, value: u8) -> bool {
        assert!((1..=9).contains(&value), "Value must be between 1 and 9");
        (self.bits & (1 << value)) != 0
    }

    /// Adds a value to the candidate set
    pub fn insert(&mut self, value: u8) {
        assert!((1..=9).contains(&value), "Value must be between 1 and 9");
        self.bits |= 1 << value;
    }

    /// Removes a value from the candidate set
    pub fn remove(&mut self, value: u8) {
        assert!((1..=9).contains(&value), "Value must be between 1 and 9");
        self.bits &= !(1 << value);
    }

    /// Returns the number of candidates in the set
    pub fn count(&self) -> u32 {
        self.bits.count_ones()
    }

    /// Checks if the set is empty
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// Checks if the set contains exactly one candidate (cell is solved)
    pub fn is_single(&self) -> bool {
        self.count() == 1
    }

    /// Returns the single value if the set contains exactly one candidate
    pub fn get_single(&self) -> Option<u8> {
        if self.is_single() {
            // Find the position of the single set bit
            Some(self.bits.trailing_zeros() as u8)
        } else {
            None
        }
    }

    /// Returns an iterator over all candidate values
    pub fn iter(&self) -> CandidateIter {
        CandidateIter {
            bits: self.bits,
            current: 1,
        }
    }

    /// Returns a vector of all candidate values
    pub fn to_vec(&self) -> Vec<u8> {
        self.iter().collect()
    }

    /// Union: returns candidates that are in either set
    pub fn union(&self, other: &CandidateSet) -> CandidateSet {
        CandidateSet {
            bits: self.bits | other.bits,
        }
    }

    /// Intersection: returns candidates that are in both sets
    pub fn intersection(&self, other: &CandidateSet) -> CandidateSet {
        CandidateSet {
            bits: self.bits & other.bits,
        }
    }

    /// Difference: returns candidates that are in self but not in other
    pub fn difference(&self, other: &CandidateSet) -> CandidateSet {
        CandidateSet {
            bits: self.bits & !other.bits,
        }
    }

    /// Removes all candidates from other from this set
    pub fn remove_all(&mut self, other: &CandidateSet) {
        self.bits &= !other.bits;
    }
}

impl fmt::Display for CandidateSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{{}}")
        } else {
            let values: Vec<String> = self.iter().map(|v| v.to_string()).collect();
            write!(f, "{{{}}}", values.join(","))
        }
    }
}

/// Iterator over candidate values in a CandidateSet
pub struct CandidateIter {
    bits: u16,
    current: u8,
}

impl Iterator for CandidateIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current <= 9 {
            let value = self.current;
            self.current += 1;
            if (self.bits & (1 << value)) != 0 {
                return Some(value);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_set() {
        let set = CandidateSet::full();
        assert_eq!(set.count(), 9);
        for i in 1..=9 {
            assert!(set.contains(i));
        }
    }

    #[test]
    fn test_empty_set() {
        let set = CandidateSet::empty();
        assert_eq!(set.count(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_single_value() {
        let set = CandidateSet::single(5);
        assert_eq!(set.count(), 1);
        assert!(set.is_single());
        assert_eq!(set.get_single(), Some(5));
    }

    #[test]
    fn test_insert_remove() {
        let mut set = CandidateSet::empty();
        set.insert(3);
        set.insert(7);
        assert_eq!(set.count(), 2);
        assert!(set.contains(3));
        assert!(set.contains(7));

        set.remove(3);
        assert_eq!(set.count(), 1);
        assert!(!set.contains(3));
        assert!(set.contains(7));
    }

    #[test]
    fn test_set_operations() {
        let set1 = {
            let mut s = CandidateSet::empty();
            s.insert(1);
            s.insert(2);
            s.insert(3);
            s
        };

        let set2 = {
            let mut s = CandidateSet::empty();
            s.insert(2);
            s.insert(3);
            s.insert(4);
            s
        };

        let union = set1.union(&set2);
        assert_eq!(union.count(), 4);
        assert!(union.contains(1) && union.contains(2) && union.contains(3) && union.contains(4));

        let intersection = set1.intersection(&set2);
        assert_eq!(intersection.count(), 2);
        assert!(intersection.contains(2) && intersection.contains(3));

        let difference = set1.difference(&set2);
        assert_eq!(difference.count(), 1);
        assert!(difference.contains(1));
    }

    #[test]
    fn test_iterator() {
        let mut set = CandidateSet::empty();
        set.insert(2);
        set.insert(5);
        set.insert(8);

        let values: Vec<u8> = set.iter().collect();
        assert_eq!(values, vec![2, 5, 8]);
    }
}
