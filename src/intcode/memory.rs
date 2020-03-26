//! Advent of Code 2019: Intcode memory

/// Intcode memory address
pub type Address = usize;

/// Intcode memory value
pub type Value = i32;

/// Intcode memory
///
/// Memory of an Intcode machine is a continuous range of signed integers addressed by their
/// position (zero based index). Memory can be loaded from (ASCII) text files with content encoded
/// as comma separated values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Memory(Vec<Value>);

impl From<Vec<Value>> for Memory {
    fn from(data: Vec<Value>) -> Self {
        Memory(data)
    }
}

impl<T: AsRef<[Value]>> PartialEq<T> for Memory {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl Memory {
    /// Returns the size of memory
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Get value at given memory address
    pub fn get(&self, addr: Address) -> Value {
        assert!(
            addr < self.size(),
            "Reading from memory out of bounds ({} >= {})",
            addr,
            self.size()
        );
        self.0[addr]
    }

    /// Get slice of values at given memory address window
    pub fn get_slice(&self, addr: Address, len: usize) -> &[Value] {
        let addr_end = Address::min(addr + len, self.size());
        assert!(
            addr < self.size(),
            "Reading from memory out of bounds ({}..{} >= {})",
            addr,
            addr_end,
            self.size()
        );
        &self.0[addr..addr_end]
    }

    /// Set value at given memory address
    pub fn set(&mut self, addr: Address, value: Value) {
        assert!(
            addr < self.size(),
            "Writing to memory out of bounds ({} >= {})",
            addr,
            self.size()
        );
        self.0[addr] = value;
    }
}
