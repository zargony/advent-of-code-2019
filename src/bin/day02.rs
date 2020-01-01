use advent_of_code_2019::intcode::{Memory, Vm};
use async_std::io;

#[async_std::main]
async fn main() -> io::Result<()> {
    let mut memory = Memory::from_day(2).await?;
    let mut vm = Vm::new(&mut memory);
    vm.noun(12).verb(2).run();
    println!("Result: {}", vm.result());

    let memory = Memory::from_day(2).await?;
    'out: for noun in 0..=99 {
        for verb in 0..=99 {
            let mut memory = memory.clone();
            let mut vm = Vm::new(&mut memory);
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
