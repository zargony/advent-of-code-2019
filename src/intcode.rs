//! Advent of Code 2019: Intcode

use std::collections::HashMap;

/// Intcode virtual machine
#[derive(Debug)]
pub struct Vm<'p> {
    program: &'p [u32],
    memory: HashMap<usize, u32>,
    ip: usize,
    done: bool,
}

impl<'p> Vm<'p> {
    /// Get value at given memory address
    fn get(&self, addr: usize) -> u32 {
        assert!(addr < self.program.len());
        match self.memory.get(&addr) {
            Some(value) => *value,
            None => self.program[addr],
        }
    }

    /// Set value at given memory address
    fn set(&mut self, addr: usize, value: u32) {
        assert!(addr < self.program.len());
        self.memory.insert(addr, value);
    }
}

impl<'p> Vm<'p> {
    /// Create new Intcode virtual machine with the given program
    pub fn new(program: &'p [u32]) -> Self {
        Self {
            program: program.as_ref(),
            memory: HashMap::new(),
            ip: 0,
            done: false,
        }
    }

    /// Set noun (value at memory address 1)
    pub fn noun(&mut self, noun: u32) -> &mut Self {
        assert!(noun <= 99);
        self.set(1, noun);
        self
    }

    /// Set verb (value at memory address 2)
    pub fn verb(&mut self, verb: u32) -> &mut Self {
        assert!(verb <= 99);
        self.set(2, verb);
        self
    }

    /// Run one program step
    pub fn step(&mut self) {
        match self.get(self.ip) {
            // Instruction with opcode 1: addition
            1 => {
                let param1 = self.get(self.ip + 1);
                let param2 = self.get(self.ip + 2);
                let param3 = self.get(self.ip + 3);
                let result = self.get(param1 as usize) + self.get(param2 as usize);
                self.set(param3 as usize, result);
            }
            // Instruction with opcode 2: multiplication
            2 => {
                let param1 = self.get(self.ip + 1);
                let param2 = self.get(self.ip + 2);
                let param3 = self.get(self.ip + 3);
                let result = self.get(param1 as usize) * self.get(param2 as usize);
                self.set(param3 as usize, result);
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

    /// Return result (value at memory address 0)
    pub fn result(&self) -> u32 {
        self.get(0)
    }

    /// Consume the vm and return a dump of its memory
    pub fn dump(self) -> Vec<u32> {
        let mut memory = self.program.to_vec();
        for (addr, value) in self.memory {
            memory[addr] = value;
        }
        memory
    }
}
