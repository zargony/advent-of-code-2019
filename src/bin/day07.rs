use advent_of_code_2019::intcode::{Memory, Value, Vm};
use async_std::io;

struct AmplificationCircuit {
    program: Memory,
}

impl AmplificationCircuit {
    /// Create new amplification circuit with the given amplifier controller software
    fn new(program: Memory) -> Self {
        Self { program }
    }

    /// Run a single amplifier with the given phase setting and input value
    fn run_amp(&self, phase: Value, input: Value) -> Value {
        let mut vm = Vm::new(self.program.clone());
        vm.input(&[phase, input]).run();
        vm.output()[0]
    }

    /// Run a chain of amplifiers with the given phase settings
    fn run_amp_chain(&self, phases: &[Value]) -> Value {
        phases
            .iter()
            .fold(0, |value, phase| self.run_amp(*phase, value))
    }

    /// Run chain of 5 amplifiers trying all permuations of phase settings to find the max output
    fn find_max_thrust_k5(&self) -> (Vec<Value>, Value) {
        permutator::KPermutationIterator::new(&[0, 1, 2, 3, 4], 5)
            .map(|phases| phases.into_iter().cloned().collect::<Vec<_>>())
            .map(|phases| {
                let output = self.run_amp_chain(&phases);
                (phases, output)
            })
            .max_by_key(|(_phases, output)| *output)
            .unwrap()
    }
}

#[async_std::main]
async fn main() -> io::Result<()> {
    let program = Memory::from_day(7).await?;
    let amplifiers = AmplificationCircuit::new(program);

    println!("Max thruster signal {}", amplifiers.find_max_thrust_k5().1);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_1_example_1() {
        let program = Memory::from(vec![
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ]);
        let amplifiers = AmplificationCircuit::new(program);
        assert_eq!(amplifiers.find_max_thrust_k5(), (vec![4, 3, 2, 1, 0], 43210));
    }

    #[test]
    fn part_1_example_2() {
        let program = Memory::from(vec![
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ]);
        let amplifiers = AmplificationCircuit::new(program);
        assert_eq!(amplifiers.find_max_thrust_k5(), (vec![0, 1, 2, 3, 4], 54321));
    }

    #[test]
    fn part_1_example_3() {
        let program = Memory::from(vec![
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ]);
        let amplifiers = AmplificationCircuit::new(program);
        assert_eq!(amplifiers.find_max_thrust_k5(), (vec![1, 0, 4, 3, 2], 65210));
    }
}
