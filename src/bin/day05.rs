use advent_of_code_2019::intcode::{Memory, Vm};
use async_std::io;

#[async_std::main]
async fn main() -> io::Result<()> {
    let program = Memory::from_day(5).await?;

    let mut vm = Vm::new(program.clone());
    vm.input(&[1]).run();
    println!("TEST diagnostic output: {:?}", vm.output());

    let mut vm = Vm::new(program);
    vm.input(&[5]).run();
    println!("TEST diagnostic code for system ID 5: {}", vm.output()[0]);

    Ok(())
}
