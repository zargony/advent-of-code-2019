//! Advent of Code 2019 helper library

#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use async_std::fs::File;
use async_std::io::{self, BufReader, SeekFrom};
use async_std::path::PathBuf;
use async_std::prelude::*;
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

    /// Reset puzzle input to start over at the beginning
    pub async fn reset(&mut self) -> io::Result<()> {
        self.reader.get_mut().seek(SeekFrom::Start(0)).await?;
        Ok(())
    }

    /// Read next line from puzzle input
    pub async fn next_line(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        match self.reader.read_line(&mut line).await? {
            0 => Ok(None),
            _ => {
                line.pop();
                Ok(Some(line))
            },
        }
    }

    /// Read and parse next line from puzzle input
    pub async fn parse_next_line<T>(&mut self) -> io::Result<Option<T>>
    where
        T: FromStr,
        T::Err: error::Error + Send + Sync + 'static,
    {
        match self.next_line().await? {
            Some(line) => match line.parse() {
                Ok(value) => Ok(Some(value)),
                Err(err) => Err(io::Error::new(io::ErrorKind::InvalidData, err)),
            },
            None => Ok(None),
        }
    }
}
