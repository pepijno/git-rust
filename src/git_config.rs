use std::fs::File;
use std::io::{BufReader, BufRead, Read, stdin};
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ConfigError {
    InvalidSectionLine(usize, String),
    InvalidValue(usize, String),
}

impl Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::InvalidSectionLine(index, line) => write!(f, "Line {}: invalid section {}", index, line),
            ConfigError::InvalidValue(index, line) => write!(f, "Line {}: invalid value {}", index, line),
        }
    }
}

#[derive(Debug)]
pub struct ConfigKey {
    section: String,
    key: String,
    value: String,
}

impl fmt::Display for ConfigKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}.{} = {}", self.section, self.key, self.value)
    }
}

pub fn read_config_file(file: File) -> Result<Vec<ConfigKey>, ConfigError> {
    let reader = BufReader::new(file);
    read_config(reader)
}

pub fn read_config_stdin() -> Result<Vec<ConfigKey>, ConfigError> {
    let stdin = stdin();
    let handler = stdin.lock();
    let reader = BufReader::new(handler);
    read_config(reader)
}

fn read_config<R: Read>(reader: BufReader<R>) -> Result<Vec<ConfigKey>, ConfigError> {
    let mut keys: Vec<ConfigKey> = Vec::new();

    let mut section: String = String::new();
    for (index, line) in reader.lines().enumerate().filter_map(filter_comments) {
        if line.starts_with('[') {
            if !line.ends_with(']') {
                return Err(ConfigError::InvalidSectionLine(index, line));
            }
            section = line[1..(line.len() - 1)].to_string();
        } else {
            let split: Vec<&str> = line.splitn(2, '=').collect();
            if split.len() != 2 {
                return Err(ConfigError::InvalidValue(index, line));
            }
            let trimmed: Vec<String> = split.into_iter().map(|x| x.trim().to_string()).collect();
            let key = trimmed.first().unwrap().to_string();
            let value = trimmed.last().unwrap().to_string();
            keys.push(ConfigKey { section: section.clone(), key, value });
        }
    }
    Ok(keys)
}

fn filter_comments(input: (usize, Result<String, std::io::Error>)) -> Option<(usize, String)> {
    let (index, line) = input;
    let line = line.unwrap();
    let comments = line.split('#').next().unwrap_or("");
    Some(comments)
        .filter(|string| !string.is_empty())
        .map(|string| (index, string.parse().unwrap()))
}