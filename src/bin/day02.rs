use advent_of_code_2019::intcode::{Memory, Vm};
use async_std::io;

#[async_std::main]
async fn main() -> io::Result<()> {
    let program = Memory::from_day(2).await?;

    let mut vm = Vm::new(program.clone());
    vm.noun(12).verb(2).run();
    println!("Result: {}", vm.result());

    'out: for noun in 0..=99 {
        for verb in 0..=99 {
            let mut vm = Vm::new(program.clone());
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
