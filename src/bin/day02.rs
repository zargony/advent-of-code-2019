use advent_of_code_2019::intcode::Vm;
use advent_of_code_2019::Input;
use async_std::io;
use futures_util::stream::TryStreamExt;

#[async_std::main]
async fn main() -> io::Result<()> {
    let program = Input::day(2)
        .await?
        .parsed_csv_lines::<u32>()
        .try_concat()
        .await?;

    let mut vm = Vm::new(&program);
    vm.noun(12).verb(2).run();
    println!("Result: {}", vm.result());

    'out: for noun in 0..=99 {
        for verb in 0..=99 {
            let mut vm = Vm::new(&program);
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
        let mut vm = Vm::new(&[1, 0, 0, 0, 99]);
        vm.run();
        assert_eq!(vm.dump(), &[2, 0, 0, 0, 99]);

        let mut vm = Vm::new(&[2, 3, 0, 3, 99]);
        vm.run();
        assert_eq!(vm.dump(), &[2, 3, 0, 6, 99]);

        let mut vm = Vm::new(&[2, 4, 4, 5, 99, 0]);
        vm.run();
        assert_eq!(vm.dump(), &[2, 4, 4, 5, 99, 9801]);

        let mut vm = Vm::new(&[1, 1, 1, 4, 99, 5, 6, 0, 99]);
        vm.run();
        assert_eq!(vm.dump(), &[30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }
}
