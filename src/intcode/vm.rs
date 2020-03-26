//! Advent of Code 2019: Intcode VM

use super::memory::{Address, Memory, Value};
use async_std::prelude::*;
use async_std::sync::{self, Sender};
use std::fmt;

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
    async fn execute(&self, vm: &mut Vm) {
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
                let rx = vm.input.as_mut().expect("No input channel set");
                let value = rx
                    .next()
                    .await
                    .expect("No input values left (input channel closed)");
                p1.store(&mut vm.memory, value);
                vm.ip += 2;
            }
            Instruction::Output(p1) => {
                let tx = vm.output.as_mut().expect("No output channel set");
                tx.send(p1.fetch(&vm.memory)).await;
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
            Instruction::Done => {
                vm.input = None;
                vm.output = None;
                vm.done = true;
            }
        }
    }
}

/// Intcode virtual machine
pub struct Vm {
    /// Memory of the virtual machine
    memory: Memory,
    /// Instruction pointer (address of next instruction)
    ip: Address,
    /// Input channel for receiving input values
    input: Option<Box<dyn Stream<Item = Value> + Unpin>>,
    /// Output channel for sending output values
    output: Option<Sender<Value>>,
    /// Flag to signal that the program is done
    done: bool,
}

impl fmt::Debug for Vm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vm")
            .field("memory", &self.memory)
            .field("ip", &self.ip)
            .field("input-present", &self.input.is_some())
            .field("output-present", &self.output.is_some())
            .field("done", &self.done)
            .finish()
    }
}

impl From<Memory> for Vm {
    fn from(memory: Memory) -> Self {
        Self {
            memory,
            ip: Address::default(),
            input: None,
            output: None,
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

    /// Set stream that yields input values for the vm
    pub fn input(&mut self, input: impl Stream<Item = Value> + Unpin + 'static) -> &mut Self {
        self.input = Some(Box::new(input));
        self
    }

    /// Run one program step
    pub async fn step(&mut self) {
        let instruction = Instruction::parse(self.memory.get_slice(self.ip, 4));
        instruction.execute(self).await;
    }

    /// Run program (run steps until done)
    pub async fn run(&mut self) {
        while !self.done {
            self.step().await;
        }
    }

    /// Run program and collect output into a vector
    pub async fn run_and_collect(&mut self) -> Vec<Value> {
        let rx = self.output();
        self.run().join(rx.collect()).await.1
    }

    /// Return a stream that yields output values of the vm
    pub fn output(&mut self) -> impl Stream<Item = Value> + Unpin + 'static {
        assert!(self.output.is_none(), "Output stream already set");
        let (tx, rx) = sync::channel(1);
        self.output = Some(tx);
        rx
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
