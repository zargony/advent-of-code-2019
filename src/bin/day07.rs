use advent_of_code_2019::intcode::{Memory, Value, Vm};
use advent_of_code_2019::Input;
use async_std::prelude::*;
use async_std::{io, stream};
use futures_util::future;
use futures_util::stream::StreamExt;

#[derive(Debug)]
struct AmplifierChain {
    amplifiers: Vec<Vm>,
}

impl AmplifierChain {
    /// Create a new amplifier chain with the given program and phases
    fn new(program: Memory, phases: &[Value]) -> Self {
        let mut amplifiers: Vec<Vm> = Vec::with_capacity(phases.len());
        for &phase in phases {
            let mut amp = Vm::new(program.clone());
            if let Some(prev_amp) = amplifiers.last_mut() {
                amp.input(stream::once(phase).chain(prev_amp.output()));
            }
            amplifiers.push(amp);
        }
        amplifiers[0].input(stream::once(phases[0]).chain(stream::once(0)));
        Self { amplifiers }
    }

    /// Return a stream that yields output values of the amplifier chain
    fn output(&mut self) -> impl Stream<Item = Value> + Unpin + 'static {
        self.amplifiers.last_mut().unwrap().output()
    }

    /// Run the amplifier chain
    async fn run(&mut self) {
        future::join_all(self.amplifiers.iter_mut().map(|amp| amp.run())).await;
    }

    /// Run the amplifier chain and collect output into a vector
    async fn run_and_collect(&mut self) -> Vec<Value> {
        let rx = self.output();
        self.run().join(rx.collect()).await.1
    }

    /// Run the amplifier chain and collect a single result
    async fn run_single_result(&mut self) -> Value {
        let results = self.run_and_collect().await;
        assert!(
            results.len() == 1,
            "Amplifier yielded {} results unexpectedly",
            results.len()
        );
        results[0]
    }

    /// Stream of amplifier chain outputs for all k-permutations of the given phase values
    fn permutate<'a>(
        program: Memory,
        phases: &'a [Value],
    ) -> impl Stream<Item = (Vec<Value>, Value)> + 'a {
        stream::from_iter(permutator::KPermutationIterator::new(phases, phases.len()))
            .map(|phases| phases.into_iter().cloned().collect::<Vec<_>>())
            .map(move |phases| (Self::new(program.clone(), &phases), phases))
            .then(|(mut amp, phases)| async move { (phases, amp.run_single_result().await) })
    }

    /// Return max output over all k-permutations of the given phase values
    async fn permutate_max(program: Memory, phases: &[Value]) -> Option<(Vec<Value>, Value)> {
        Self::permutate(program, phases)
            .fold(None, |res, (phases, thrust)| {
                future::ready(match res {
                    Some((ref _ph, ref th)) if thrust < *th => res,
                    _ => Some((phases, thrust)),
                })
            })
            .await
    }
}

#[async_std::main]
async fn main() -> io::Result<()> {
    let program = Input::day(7).await?.memory().await?;

    let (phases, thrust) = AmplifierChain::permutate_max(program, &[0, 1, 2, 3, 4])
        .await
        .unwrap();
    println!(
        "Phase configuration {:?} yields max thruster signal of {}",
        phases, thrust
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn part_1_example_1() {
        let program = Memory::from(vec![
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ]);
        assert_eq!(
            AmplifierChain::permutate_max(program, &[0, 1, 2, 3, 4]).await,
            Some((vec![4, 3, 2, 1, 0], 43210))
        );
    }

    #[async_std::test]
    async fn part_1_example_2() {
        let program = Memory::from(vec![
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ]);
        assert_eq!(
            AmplifierChain::permutate_max(program, &[0, 1, 2, 3, 4]).await,
            Some((vec![0, 1, 2, 3, 4], 54321))
        );
    }

    #[async_std::test]
    async fn part_1_example_3() {
        let program = Memory::from(vec![
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ]);
        assert_eq!(
            AmplifierChain::permutate_max(program, &[0, 1, 2, 3, 4]).await,
            Some((vec![1, 0, 4, 3, 2], 65210))
        );
    }
}
