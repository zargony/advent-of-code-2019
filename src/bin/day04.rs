use advent_of_code_2019::Input;
use futures_util::stream::TryStreamExt;
use std::{error, fmt};

/// A six digit number password
#[derive(Debug)]
struct Password([u8; 6]);

impl From<u32> for Password {
    fn from(n: u32) -> Self {
        Password([
            (n / 100_000 % 10) as u8,
            (n / 10_000 % 10) as u8,
            (n / 1_000 % 10) as u8,
            (n / 100 % 10) as u8,
            (n / 10 % 10) as u8,
            (n % 10) as u8,
        ])
    }
}

impl From<Password> for u32 {
    fn from(pwd: Password) -> Self {
        pwd.0[0] as u32 * 100_000
            + pwd.0[1] as u32 * 10_000
            + pwd.0[2] as u32 * 1_000
            + pwd.0[3] as u32 * 100
            + pwd.0[4] as u32 * 10
            + pwd.0[5] as u32
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}{}{}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
        )
    }
}

impl Password {
    /// Check if password has double digits
    fn has_double(&self) -> bool {
        self.0[0] == self.0[1]
            || self.0[1] == self.0[2]
            || self.0[2] == self.0[3]
            || self.0[3] == self.0[4]
            || self.0[4] == self.0[5]
    }

    /// Check if password has double digits (strict)
    fn has_double_strict(&self) -> bool {
        (self.0[0] == self.0[1] && self.0[1] != self.0[2])
            || (self.0[1] == self.0[2] && self.0[0] != self.0[1] && self.0[2] != self.0[3])
            || (self.0[2] == self.0[3] && self.0[1] != self.0[2] && self.0[3] != self.0[4])
            || (self.0[3] == self.0[4] && self.0[2] != self.0[3] && self.0[4] != self.0[5])
            || (self.0[4] == self.0[5] && self.0[3] != self.0[4])
    }

    /// Check if password has increasing digits
    fn is_increasing(&self) -> bool {
        self.0[0] <= self.0[1]
            && self.0[1] <= self.0[2]
            && self.0[2] <= self.0[3]
            && self.0[3] <= self.0[4]
            && self.0[4] <= self.0[5]
    }

    /// Check if password is valid (has double digits and increasing digits)
    fn is_valid(&self) -> bool {
        self.has_double() && self.is_increasing()
    }

    /// Check if password is valid (has strict double digits and increasing digits)
    fn is_valid_strict(&self) -> bool {
        self.has_double_strict() && self.is_increasing()
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let mut lines = Input::day(4).await?.lines();
    let line = lines.try_next().await?.unwrap();
    let mut numbers = line.split('-');
    let start = numbers.next().unwrap().parse::<u32>()?;
    let end = numbers.next().unwrap().parse::<u32>()?;

    // Brute force valid passwords in the given range
    let count = (start..=end)
        .map(Password::from)
        .filter(Password::is_valid)
        .count();
    println!("Number of different passwords: {}", count);

    // Brute force strict valid passwords in the given range
    let count = (start..=end)
        .map(Password::from)
        .filter(Password::is_valid_strict)
        .count();
    println!("Number of strictly different passwords: {}", count);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_1() {
        let pwd = Password::from(111_111);
        assert!(pwd.has_double());
        assert!(pwd.is_increasing());
        assert!(pwd.is_valid());

        let pwd = Password::from(223_450);
        assert!(pwd.has_double());
        assert!(!pwd.is_increasing());
        assert!(!pwd.is_valid());

        let pwd = Password::from(123_789);
        assert!(!pwd.has_double());
        assert!(pwd.is_increasing());
        assert!(!pwd.is_valid());
    }

    #[test]
    fn part_2() {
        let pwd = Password::from(112_233);
        assert!(pwd.has_double_strict());
        assert!(pwd.is_increasing());
        assert!(pwd.is_valid_strict());

        let pwd = Password::from(123_444);
        assert!(!pwd.has_double_strict());
        assert!(pwd.is_increasing());
        assert!(!pwd.is_valid_strict());

        let pwd = Password::from(111_122);
        assert!(pwd.has_double_strict());
        assert!(pwd.is_increasing());
        assert!(pwd.is_valid_strict());
    }
}
