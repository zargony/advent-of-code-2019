use advent_of_code_2019::Input;
use async_std::io;
use futures_util::stream::TryStreamExt;

/// IntCode virtual machine
#[derive(Debug)]
struct IntVm {
    memory: Vec<u32>,
    ip: usize,
    done: bool,
}

impl IntVm {
    /// Create new IntCode virtual machine with the given program
    fn new(program: Vec<u32>) -> Self {
        Self {
            memory: program,
            ip: 0,
            done: false,
        }
    }

    /// Create new IntCode virtual machine and read program from the given input
    async fn load(input: Input) -> io::Result<Self> {
        let program = input.parsed_csv_lines::<u32>().try_concat().await?;
        Ok(Self::new(program))
    }

    /// Set noun (value at memory address 1)
    fn noun(&mut self, noun: u32) -> &mut Self {
        assert!(noun <= 99);
        self.memory[1] = noun;
        self
    }

    /// Set verb (value at memory address 2)
    fn verb(&mut self, verb: u32) -> &mut Self {
        assert!(verb <= 99);
        self.memory[2] = verb;
        self
    }

    /// Run one program step
    fn step(&mut self) {
        match self.memory[self.ip] {
            // Instruction with opcode 1: addition
            1 => {
                let param1 = self.memory[self.ip + 1];
                let param2 = self.memory[self.ip + 2];
                let param3 = self.memory[self.ip + 3];
                self.memory[param3 as usize] =
                    self.memory[param1 as usize] + self.memory[param2 as usize];
            }
            // Instruction with opcode 2: multiplication
            2 => {
                let param1 = self.memory[self.ip + 1];
                let param2 = self.memory[self.ip + 2];
                let param3 = self.memory[self.ip + 3];
                self.memory[param3 as usize] =
                    self.memory[param1 as usize] * self.memory[param2 as usize];
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
        self.memory[0]
    }

    /// Consume the vm and return a dump of its memory
    #[cfg(test)]
    fn dump(self) -> Vec<u32> {
        self.memory
    }
}

#[async_std::main]
async fn main() -> io::Result<()> {
    let input = Input::day(2).await?;
    let mut vm = IntVm::load(input).await?;
    vm.noun(12).verb(2).run();
    println!("Result: {}", vm.result());

    'out: for noun in 0..=99 {
        for verb in 0..=99 {
            let input = Input::day(2).await?;
            let mut vm = IntVm::load(input).await?;
            vm.noun(noun).verb(verb).run();
            if vm.result() == 19690720 {
                println!("Noun {} verb {} produce result {}", noun, verb, vm.result());
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
        let mut vm = IntVm::new(vec![1, 0, 0, 0, 99]);
        vm.run();
        assert_eq!(vm.dump(), &[2, 0, 0, 0, 99]);

        let mut vm = IntVm::new(vec![2, 3, 0, 3, 99]);
        vm.run();
        assert_eq!(vm.dump(), &[2, 3, 0, 6, 99]);

        let mut vm = IntVm::new(vec![2, 4, 4, 5, 99, 0]);
        vm.run();
        assert_eq!(vm.dump(), &[2, 4, 4, 5, 99, 9801]);

        let mut vm = IntVm::new(vec![1, 1, 1, 4, 99, 5, 6, 0, 99]);
        vm.run();
        assert_eq!(vm.dump(), &[30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }
}
