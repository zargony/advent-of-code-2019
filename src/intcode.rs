//! Advent of Code 2019: Intcode

use crate::input::Input;
use futures_util::stream::TryStreamExt;
use std::io;

/// Intcode memory address
type Address = usize;

/// Intcode memory value
type Value = i32;

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
    /// Load memory from puzzle input (csv) of the given day
    pub async fn from_day(day: usize) -> io::Result<Self> {
        let input = Input::day(day).await?;
        Self::from_input(input).await
    }

    /// Load memory from puzzle input (csv) with the given name
    pub async fn from_file(name: &str) -> io::Result<Self> {
        let input = Input::open(name).await?;
        Self::from_input(input).await
    }

    /// Load memory from given puzzle input (csv)
    pub async fn from_input(input: Input) -> io::Result<Self> {
        let data = input.parsed_csv_lines::<Value>().try_concat().await?;
        Ok(Self::from(data))
    }

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

/// Intcode virtual machine
#[derive(Debug)]
pub struct Vm<'m> {
    /// Memory of the virtual machine
    memory: &'m mut Memory,
    /// Instruction pointer (address of next instruction)
    ip: Address,
    /// Flag to signal that the program is done
    done: bool,
}

impl<'m> Vm<'m> {
    /// Create new virtual machine with the given memory
    pub fn new(memory: &'m mut Memory) -> Self {
        Self {
            memory,
            ip: Address::default(),
            done: false,
        }
    }

    /// Set noun (value at memory address 1)
    pub fn noun(&mut self, noun: Value) -> &mut Self {
        assert!(noun <= 99);
        self.memory.set(1, noun);
        self
    }

    /// Set verb (value at memory address 2)
    pub fn verb(&mut self, verb: Value) -> &mut Self {
        assert!(verb <= 99);
        self.memory.set(2, verb);
        self
    }

    /// Run one program step
    pub fn step(&mut self) {
        match self.memory.get(self.ip) {
            // Instruction with opcode 1: addition
            1 => {
                let param1 = self.memory.get(self.ip + 1);
                let param2 = self.memory.get(self.ip + 2);
                let param3 = self.memory.get(self.ip + 3);
                let result = self.memory.get(param1 as usize) + self.memory.get(param2 as usize);
                self.memory.set(param3 as usize, result);
            }
            // Instruction with opcode 2: multiplication
            2 => {
                let param1 = self.memory.get(self.ip + 1);
                let param2 = self.memory.get(self.ip + 2);
                let param3 = self.memory.get(self.ip + 3);
                let result = self.memory.get(param1 as usize) * self.memory.get(param2 as usize);
                self.memory.set(param3 as usize, result);
            }
            // Instruction with opcode 99: done
            99 => {
                self.done = true;
                return;
            }
            // Instruction with unknown opcode: crash
            opcode => panic!("Unknown opcode {} at address {}", opcode, self.ip),
        }
        self.ip += 4;
    }

    /// Run program (run steps until done)
    pub fn run(&mut self) -> &mut Self {
        while !self.done {
            self.step();
        }
        self
    }

    /// Return a reference to the memory
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// Return result (value at memory address 0)
    pub fn result(&self) -> Value {
        self.memory.get(0)
    }
}
