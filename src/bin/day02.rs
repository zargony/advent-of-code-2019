use advent_of_code_2019::Input;
use async_std::io;
use futures_util::stream::TryStreamExt;
use std::collections::HashMap;

/// IntCode virtual machine
#[derive(Debug)]
struct IntCodeVm<'p> {
    program: &'p [u32],
    memory: HashMap<usize, u32>,
    ip: usize,
    done: bool,
}

impl<'p> IntCodeVm<'p> {
    /// Create new IntCode virtual machine with the given program
    fn new(program: &'p [u32]) -> Self {
        Self {
            program: program.as_ref(),
            memory: HashMap::new(),
            ip: 0,
            done: false,
        }
    }

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

    /// Set noun (value at memory address 1)
    fn noun(&mut self, noun: u32) -> &mut Self {
        assert!(noun <= 99);
        self.set(1, noun);
        self
    }

    /// Set verb (value at memory address 2)
    fn verb(&mut self, verb: u32) -> &mut Self {
        assert!(verb <= 99);
        self.set(2, verb);
        self
    }

    /// Run one program step
    fn step(&mut self) {
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
    fn run(&mut self) -> &mut Self {
        while !self.done {
            self.step();
        }
        self
    }

    /// Return result (value at memory address 0)
    fn result(&self) -> u32 {
        self.get(0)
    }

    /// Consume the vm and return a dump of its memory
    #[cfg(test)]
    fn dump(self) -> Vec<u32> {
        let mut memory = self.program.to_vec();
        for (addr, value) in self.memory {
            memory[addr] = value;
        }
        memory
    }
}

#[async_std::main]
async fn main() -> io::Result<()> {
    let program = Input::day(2)
        .await?
        .parsed_csv_lines::<u32>()
        .try_concat()
        .await?;

    let mut vm = IntCodeVm::new(&program);
    vm.noun(12).verb(2).run();
    println!("Result: {}", vm.result());

    'out: for noun in 0..=99 {
        for verb in 0..=99 {
            let mut vm = IntCodeVm::new(&program);
            vm.noun(noun).verb(verb).run();
            if vm.result() == 19690720 {
                println!(
                    "Noun {} verb {} produces result {}",
                    noun,
                    verb,
                    vm.result()
                );
                break 'out;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_1() {
        let mut vm = IntCodeVm::new(&[1, 0, 0, 0, 99]);
        vm.run();
        assert_eq!(vm.dump(), &[2, 0, 0, 0, 99]);

        let mut vm = IntCodeVm::new(&[2, 3, 0, 3, 99]);
        vm.run();
        assert_eq!(vm.dump(), &[2, 3, 0, 6, 99]);

        let mut vm = IntCodeVm::new(&[2, 4, 4, 5, 99, 0]);
        vm.run();
        assert_eq!(vm.dump(), &[2, 4, 4, 5, 99, 9801]);

        let mut vm = IntCodeVm::new(&[1, 1, 1, 4, 99, 5, 6, 0, 99]);
        vm.run();
        assert_eq!(vm.dump(), &[30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }
}
