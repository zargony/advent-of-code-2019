use advent_of_code_2019::intcode::Vm;
use advent_of_code_2019::Input;
use async_std::{io, stream};

#[async_std::main]
async fn main() -> io::Result<()> {
    let program = Input::day(5).await?.memory().await?;

    let mut vm = Vm::new(program.clone());
    vm.input(stream::from_iter(vec![1]));
    println!("TEST diagnostic output: {:?}", vm.run_and_collect().await);

    let mut vm = Vm::new(program);
    vm.input(stream::from_iter(vec![5]));
    println!(
        "TEST diagnostic code for system ID 5: {}",
        vm.run_and_collect().await[0],
    );

    Ok(())
}
