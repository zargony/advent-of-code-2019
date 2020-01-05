//! Advent of Code 2019: Intcode

use crate::input::Input;
use futures_util::stream::TryStreamExt;
use std::collections::VecDeque;
use std::{fmt, io};

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
    /// Parse parameter with the given number from memory slice of the instruction
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
    /// Get value from input and store it in p1
    Input(Param),
    /// Output value
    Output(Param),
    /// Jump if true / not zero: set instruction pointer to p2 if p1 is not zero
    JumpIfNotZero(Param, Param),
    /// Jump if false / zero: set instruction pointer to p2 if p1 is zero
    JumpIfZero(Param, Param),
    /// Less than: if p1 is less than p2, stores 1 to p3, 0 otherwise
    LessThan(Param, Param, Param),
    /// Equals: if p1 equals p2, stores 1 to p3, 0 otherwise
    Equals(Param, Param, Param),
    /// Program done
    Done,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Add(p1, p2, p3) => write!(f, "add {} {} {}", p1, p2, p3),
            Instruction::Multiply(p1, p2, p3) => write!(f, "mul {} {} {}", p1, p2, p3),
            Instruction::Input(p1) => write!(f, "in {}", p1),
            Instruction::Output(p1) => write!(f, "out {}", p1),
            Instruction::JumpIfNotZero(p1, p2) => write!(f, "jnz {} {}", p1, p2),
            Instruction::JumpIfZero(p1, p2) => write!(f, "jz  {} {}", p1, p2),
            Instruction::LessThan(p1, p2, p3) => write!(f, "lt  {} {} {}", p1, p2, p3),
            Instruction::Equals(p1, p2, p3) => write!(f, "eq  {} {} {}", p1, p2, p3),
            Instruction::Done => write!(f, "done"),
        }
    }
}

impl Instruction {
    /// Parse instruction from memory slice
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
            3 => Instruction::Input(Param::parse(mem, 0)),
            4 => Instruction::Output(Param::parse(mem, 0)),
            5 => Instruction::JumpIfNotZero(Param::parse(mem, 0), Param::parse(mem, 1)),
            6 => Instruction::JumpIfZero(Param::parse(mem, 0), Param::parse(mem, 1)),
            7 => Instruction::LessThan(
                Param::parse(mem, 0),
                Param::parse(mem, 1),
                Param::parse(mem, 2),
            ),
            8 => Instruction::Equals(
                Param::parse(mem, 0),
                Param::parse(mem, 1),
                Param::parse(mem, 2),
            ),
            99 => Instruction::Done,
            opcode => panic!("Unknown opcode {}", opcode),
        }
    }

    /// Execute instruction
    fn execute(&self, vm: &mut Vm) {
        match self {
            Instruction::Add(p1, p2, p3) => {
                let result = p1.fetch(&vm.memory) + p2.fetch(&vm.memory);
                p3.store(&mut vm.memory, result);
                vm.ip += 4;
            }
            Instruction::Multiply(p1, p2, p3) => {
                let result = p1.fetch(&vm.memory) * p2.fetch(&vm.memory);
                p3.store(&mut vm.memory, result);
                vm.ip += 4;
            }
            Instruction::Input(p1) => {
                assert!(vm.input.len() > 0, "No input values left");
                p1.store(&mut vm.memory, vm.input.pop_front().unwrap());
                vm.ip += 2;
            }
            Instruction::Output(p1) => {
                vm.output.push(p1.fetch(&vm.memory));
                vm.ip += 2;
            }
            Instruction::JumpIfNotZero(p1, p2) => {
                if p1.fetch(&vm.memory) != 0 {
                    vm.ip = p2.fetch(&vm.memory) as Address;
                } else {
                    vm.ip += 3;
                }
            }
            Instruction::JumpIfZero(p1, p2) => {
                if p1.fetch(&vm.memory) == 0 {
                    vm.ip = p2.fetch(&vm.memory) as Address;
                } else {
                    vm.ip += 3;
                }
            }
            Instruction::LessThan(p1, p2, p3) => {
                if p1.fetch(&vm.memory) < p2.fetch(&vm.memory) {
                    p3.store(&mut vm.memory, 1);
                } else {
                    p3.store(&mut vm.memory, 0);
                }
                vm.ip += 4;
            }
            Instruction::Equals(p1, p2, p3) => {
                if p1.fetch(&vm.memory) == p2.fetch(&vm.memory) {
                    p3.store(&mut vm.memory, 1);
                } else {
                    p3.store(&mut vm.memory, 0);
                }
                vm.ip += 4;
            }
            Instruction::Done => vm.done = true,
        }
    }
}

/// Intcode virtual machine
#[derive(Debug)]
pub struct Vm {
    /// Memory of the virtual machine
    memory: Memory,
    /// Instruction pointer (address of next instruction)
    ip: Address,
    /// Input values
    input: VecDeque<Value>,
    /// Output values
    output: Vec<Value>,
    /// Flag to signal that the program is done
    done: bool,
}

impl From<Memory> for Vm {
    fn from(memory: Memory) -> Self {
        Self {
            memory,
            ip: Address::default(),
            input: VecDeque::new(),
            output: Vec::new(),
            done: false,
        }
    }
}

impl Vm {
    /// Create new virtual machine with the given program memory
    pub fn new(program: Memory) -> Self {
        Self::from(program)
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

    /// Set input values
    pub fn input(&mut self, values: &[Value]) -> &mut Self {
        self.input.clear();
        self.input.extend(values);
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

    /// Return a reference to output values
    pub fn output(&self) -> &[Value] {
        &self.output
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day02_example_1() {
        let program = Memory::from(vec![1, 0, 0, 0, 99]);
        let mut vm = Vm::new(program);
        vm.run();
        assert_eq!(vm.memory(), &[2, 0, 0, 0, 99]);
    }

    #[test]
    fn day02_example_2() {
        let program = Memory::from(vec![2, 3, 0, 3, 99]);
        let mut vm = Vm::new(program);
        vm.run();
        assert_eq!(vm.memory(), &[2, 3, 0, 6, 99]);
    }

    #[test]
    fn day02_example_3() {
        let program = Memory::from(vec![2, 4, 4, 5, 99, 0]);
        let mut vm = Vm::new(program);
        vm.run();
        assert_eq!(vm.memory(), &[2, 4, 4, 5, 99, 9801]);
    }

    #[test]
    fn day02_example_4() {
        let program = Memory::from(vec![1, 1, 1, 4, 99, 5, 6, 0, 99]);
        let mut vm = Vm::new(program);
        vm.run();
        assert_eq!(vm.memory(), &[30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }

    #[test]
    fn day05_position_mode_equals() {
        let program = Memory::from(vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8]);

        let mut vm = Vm::new(program.clone());
        vm.input(&[5]).run();
        assert_eq!(vm.output(), &[0]);

        let mut vm = Vm::new(program);
        vm.input(&[8]).run();
        assert_eq!(vm.output(), &[1]);
    }

    #[test]
    fn day05_position_mode_less_than() {
        let program = Memory::from(vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8]);

        let mut vm = Vm::new(program.clone());
        vm.input(&[5]).run();
        assert_eq!(vm.output(), &[1]);

        let mut vm = Vm::new(program);
        vm.input(&[8]).run();
        assert_eq!(vm.output(), &[0]);
    }

    #[test]
    fn day05_immediate_mode_equals() {
        let program = Memory::from(vec![3, 3, 1108, -1, 8, 3, 4, 3, 99]);

        let mut vm = Vm::new(program.clone());
        vm.input(&[5]).run();
        assert_eq!(vm.output(), &[0]);

        let mut vm = Vm::new(program);
        vm.input(&[8]).run();
        assert_eq!(vm.output(), &[1]);
    }

    #[test]
    fn day05_immediate_mode_less_than() {
        let program = Memory::from(vec![3, 3, 1107, -1, 8, 3, 4, 3, 99]);

        let mut vm = Vm::new(program.clone());
        vm.input(&[5]).run();
        assert_eq!(vm.output(), &[1]);

        let mut vm = Vm::new(program);
        vm.input(&[8]).run();
        assert_eq!(vm.output(), &[0]);
    }

    #[test]
    fn day05_position_mode_jump() {
        let program = Memory::from(vec![
            3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9,
        ]);

        let mut vm = Vm::new(program.clone());
        vm.input(&[0]).run();
        assert_eq!(vm.output(), &[0]);

        let mut vm = Vm::new(program);
        vm.input(&[1]).run();
        assert_eq!(vm.output(), &[1]);
    }

    #[test]
    fn day05_immediate_mode_jump() {
        let program = Memory::from(vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1]);

        let mut vm = Vm::new(program.clone());
        vm.input(&[0]).run();
        assert_eq!(vm.output(), &[0]);

        let mut vm = Vm::new(program);
        vm.input(&[1]).run();
        assert_eq!(vm.output(), &[1]);
    }

    #[test]
    fn day05_large_example() {
        let program = Memory::from(vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);

        let mut vm = Vm::new(program.clone());
        vm.input(&[5]).run();
        assert_eq!(vm.output(), &[999]);

        let mut vm = Vm::new(program.clone());
        vm.input(&[8]).run();
        assert_eq!(vm.output(), &[1000]);

        let mut vm = Vm::new(program);
        vm.input(&[11]).run();
        assert_eq!(vm.output(), &[1001]);
    }
}
