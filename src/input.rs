//! Advent of Code 2019: puzzle input reading

use async_std::fs::File;
use async_std::io::{self, BufReader};
use async_std::path::PathBuf;
use async_std::prelude::*;
use futures_util::future::ready;
use futures_util::stream::TryStreamExt;
use std::error;
use std::str::FromStr;

/// Path to puzzle input files
const INPUT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/input");

/// Puzzle input
#[derive(Debug)]
pub struct Input {
    reader: BufReader<File>,
}

impl Input {
    /// Open puzzle input for the given day
    pub async fn day(day: usize) -> io::Result<Self> {
        Self::open(&format!("day{:02}", day)).await
    }

    /// Open puzzle input with the given name
    pub async fn open(name: &str) -> io::Result<Self> {
        let mut filename: PathBuf = INPUT_PATH.into();
        filename.push(name);
        filename.set_extension("txt");
        let reader = BufReader::new(File::open(filename).await?);
        Ok(Input { reader })
    }

    /// Stream of lines
    pub fn lines(self) -> impl Stream<Item = io::Result<String>> {
        self.reader.lines()
    }

    /// Stream of parsed lines
    pub fn parsed_lines<T>(self) -> impl Stream<Item = io::Result<T>>
    where
        T: FromStr,
        T::Err: error::Error + Send + Sync + 'static,
    {
        self.lines().and_then(|line| {
            ready(
                line.parse()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            )
        })
    }

    /// Stream of comma separated values
    pub fn csv_lines(self) -> impl Stream<Item = io::Result<Vec<String>>> {
        self.lines().map_ok(|line| {
            line.split(',')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
    }

    /// Stream of parsed comma separated values
    pub fn parsed_csv_lines<T>(self) -> impl Stream<Item = io::Result<Vec<T>>>
    where
        T: FromStr,
        T::Err: error::Error + Send + Sync + 'static,
    {
        self.csv_lines().and_then(|values| {
            ready(
                values
                    .iter()
                    .map(|value| {
                        value
                            .parse()
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                    })
                    .collect(),
            )
        })
    }
}
