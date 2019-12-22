use advent_of_code_2019::Input;
use async_std::io;

fn fuel_for_mass(mass: u32) -> u32 {
    match mass / 3 {
        0..=2 => 0,
        x => x - 2,
    }
}

fn fuel_for_mass_including_fuel(mass: u32) -> u32 {
    match fuel_for_mass(mass) {
        0 => 0,
        fuel => fuel + fuel_for_mass_including_fuel(fuel),
    }
}

#[async_std::main]
async fn main() -> io::Result<()> {
    let mut input = Input::day(1).await?;
    let mut total_fuel = 0;
    while let Some(mass) = input.parse_next_line::<u32>().await? {
        total_fuel += fuel_for_mass(mass);
    }
    println!("Sum of fuel requirements: {}", total_fuel);

    input.reset().await?;
    total_fuel = 0;
    while let Some(mass) = input.parse_next_line::<u32>().await? {
        total_fuel += fuel_for_mass_including_fuel(mass);
    }
    println!("Sum of fuel requirements including fuel: {}", total_fuel);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_1() {
        assert_eq!(fuel_for_mass(12), 2);
        assert_eq!(fuel_for_mass(14), 2);
        assert_eq!(fuel_for_mass(1969), 654);
        assert_eq!(fuel_for_mass(100756), 33583);
    }

    #[test]
    fn part_2() {
        assert_eq!(fuel_for_mass_including_fuel(14), 2);
        assert_eq!(fuel_for_mass_including_fuel(1969), 966);
        assert_eq!(fuel_for_mass_including_fuel(100756), 50346);
    }
}
