//! Advent of Code 2019: Intcode

mod memory;
pub use self::memory::{Address, Memory, Value};

mod vm;
pub use self::vm::Vm;

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::stream;

    #[async_std::test]
    async fn day02_example_1() {
        let program = Memory::from(vec![1, 0, 0, 0, 99]);
        let mut vm = Vm::new(program);
        vm.run().await;
        assert_eq!(vm.memory(), &[2, 0, 0, 0, 99]);
    }

    #[async_std::test]
    async fn day02_example_2() {
        let program = Memory::from(vec![2, 3, 0, 3, 99]);
        let mut vm = Vm::new(program);
        vm.run().await;
        assert_eq!(vm.memory(), &[2, 3, 0, 6, 99]);
    }

    #[async_std::test]
    async fn day02_example_3() {
        let program = Memory::from(vec![2, 4, 4, 5, 99, 0]);
        let mut vm = Vm::new(program);
        vm.run().await;
        assert_eq!(vm.memory(), &[2, 4, 4, 5, 99, 9801]);
    }

    #[async_std::test]
    async fn day02_example_4() {
        let program = Memory::from(vec![1, 1, 1, 4, 99, 5, 6, 0, 99]);
        let mut vm = Vm::new(program);
        vm.run().await;
        assert_eq!(vm.memory(), &[30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }

    #[async_std::test]
    async fn day05_position_mode_equals() {
        let program = Memory::from(vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8]);

        let mut vm = Vm::new(program.clone());
        vm.input(stream::from_iter(vec![5]));
        assert_eq!(vm.run_and_collect().await, &[0]);

        let mut vm = Vm::new(program);
        vm.input(stream::from_iter(vec![8]));
        assert_eq!(vm.run_and_collect().await, &[1]);
    }

    #[async_std::test]
    async fn day05_position_mode_less_than() {
        let program = Memory::from(vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8]);

        let mut vm = Vm::new(program.clone());
        vm.input(stream::from_iter(vec![5]));
        assert_eq!(vm.run_and_collect().await, &[1]);

        let mut vm = Vm::new(program);
        vm.input(stream::from_iter(vec![8]));
        assert_eq!(vm.run_and_collect().await, &[0]);
    }

    #[async_std::test]
    async fn day05_immediate_mode_equals() {
        let program = Memory::from(vec![3, 3, 1108, -1, 8, 3, 4, 3, 99]);

        let mut vm = Vm::new(program.clone());
        vm.input(stream::from_iter(vec![5]));
        assert_eq!(vm.run_and_collect().await, &[0]);

        let mut vm = Vm::new(program);
        vm.input(stream::from_iter(vec![8]));
        assert_eq!(vm.run_and_collect().await, &[1]);
    }

    #[async_std::test]
    async fn day05_immediate_mode_less_than() {
        let program = Memory::from(vec![3, 3, 1107, -1, 8, 3, 4, 3, 99]);

        let mut vm = Vm::new(program.clone());
        vm.input(stream::from_iter(vec![5]));
        assert_eq!(vm.run_and_collect().await, &[1]);

        let mut vm = Vm::new(program);
        vm.input(stream::from_iter(vec![8]));
        assert_eq!(vm.run_and_collect().await, &[0]);
    }

    #[async_std::test]
    async fn day05_position_mode_jump() {
        let program = Memory::from(vec![
            3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9,
        ]);

        let mut vm = Vm::new(program.clone());
        vm.input(stream::from_iter(vec![0]));
        assert_eq!(vm.run_and_collect().await, &[0]);

        let mut vm = Vm::new(program);
        vm.input(stream::from_iter(vec![1]));
        assert_eq!(vm.run_and_collect().await, &[1]);
    }

    #[async_std::test]
    async fn day05_immediate_mode_jump() {
        let program = Memory::from(vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1]);

        let mut vm = Vm::new(program.clone());
        vm.input(stream::from_iter(vec![0]));
        assert_eq!(vm.run_and_collect().await, &[0]);

        let mut vm = Vm::new(program);
        vm.input(stream::from_iter(vec![1]));
        assert_eq!(vm.run_and_collect().await, &[1]);
    }

    #[async_std::test]
    async fn day05_large_example() {
        let program = Memory::from(vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);

        let mut vm = Vm::new(program.clone());
        vm.input(stream::from_iter(vec![5]));
        assert_eq!(vm.run_and_collect().await, &[999]);

        let mut vm = Vm::new(program.clone());
        vm.input(stream::from_iter(vec![8]));
        assert_eq!(vm.run_and_collect().await, &[1000]);

        let mut vm = Vm::new(program);
        vm.input(stream::from_iter(vec![11]));
        assert_eq!(vm.run_and_collect().await, &[1001]);
    }
}
