//! Advent of Code 2019: Intcode

use crate::input::Input;
use futures_util::stream::TryStreamExt;
use std::{fmt, io};

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

/// Intcode parameter
///
/// Instructions in Intcode use a certain number of parameters in certain parameter modes. The
/// mode of a parameter determines how the parameter is used to fetch or store the actual value.
#[derive(Debug)]
enum Param {
    /// Position mode: parameter points to an address containing the value
    Position(Address),
    /// Immediate mode: parameter is used as the value
    Immediate(Value),
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Param::Position(addr) => write!(f, "[{}]", addr),
            Param::Immediate(value) => write!(f, "{}", value),
        }
    }
}

impl Param {
    /// Parse parameter with the given number from memory
    fn parse(mem: &[Value], n: usize) -> Self {
        debug_assert!(n < 3, "Parameter {} out of range", n);
        let div = (10 as Value).pow(n as u32) * 100;
        match mem[0] / div % 10 {
            0 => Param::Position(mem[1 + n] as Address),
            1 => Param::Immediate(mem[1 + n]),
            mode => panic!(
                "Unknown parameter mode {} for parameter {} in instruction {}",
                mode, n, mem[0],
            ),
        }
    }

    /// Fetch value for this parameter
    fn fetch(&self, memory: &Memory) -> Value {
        match self {
            Param::Position(address) => memory.get(*address),
            Param::Immediate(value) => *value,
        }
    }

    /// Store value into this parameter
    fn store(&self, memory: &mut Memory, value: Value) {
        match self {
            Param::Position(address) => memory.set(*address, value),
            Param::Immediate(_value) => panic!("Can't store to immediate mode parameter"),
        }
    }
}

/// Intcode instruction
///
/// Instructions in Intcode consist of the opcode that determines the operation and zero or more
/// parameters depending on which opcode is used.
#[derive(Debug)]
enum Instruction {
    /// Addition. Adds p1 and p2 and stores the sum in p3
    Add(Param, Param, Param),
    /// Addition. Multiplies p1 and p2 and stores the product in p3
    Multiply(Param, Param, Param),
    /// Program done
    Done,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Add(p1, p2, p3) => write!(f, "add {}, {}, {}", p1, p2, p3),
            Instruction::Multiply(p1, p2, p3) => write!(f, "mul {}, {}, {}", p1, p2, p3),
            Instruction::Done => write!(f, "done"),
        }
    }
}

impl Instruction {
    /// Parse instruction from memory
    fn parse(mem: &[Value]) -> Self {
        match mem[0] % 100 {
            1 => Instruction::Add(
                Param::parse(mem, 0),
                Param::parse(mem, 1),
                Param::parse(mem, 2),
            ),
            2 => Instruction::Multiply(
                Param::parse(mem, 0),
                Param::parse(mem, 1),
                Param::parse(mem, 2),
            ),
            99 => Instruction::Done,
            opcode => panic!("Unknown opcode {}", opcode),
        }
    }

    /// Size of instruction
    fn size(&self) -> usize {
        match self {
            Instruction::Add(..) => 4,
            Instruction::Multiply(..) => 4,
            Instruction::Done => 0,
        }
    }

    /// Execute instruction
    fn execute(&self, vm: &mut Vm<'_>) {
        match self {
            Instruction::Add(p1, p2, p3) => {
                let result = p1.fetch(&vm.memory) + p2.fetch(&vm.memory);
                p3.store(&mut vm.memory, result);
            }
            Instruction::Multiply(p1, p2, p3) => {
                let result = p1.fetch(&vm.memory) * p2.fetch(&vm.memory);
                p3.store(&mut vm.memory, result);
            }
            Instruction::Done => vm.done = true,
        }
        vm.ip += self.size();
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
        let instruction = Instruction::parse(self.memory.get_slice(self.ip, 4));
        instruction.execute(self);
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
