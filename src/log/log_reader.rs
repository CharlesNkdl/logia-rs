use crate::log::log_source::LogSource;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ActiveLog {
    pub source: LogSource,
    pub lines: Vec<String>,
    position: u64,
}

impl ActiveLog {
    pub fn open(source: LogSource, tail: usize) -> Self {
        let lines = tail_file(&source.path, tail);
        let position = fs::metadata(&source.path).map(|m| m.len()).unwrap_or(0);
        Self {
            source,
            lines,
            position,
        }
    }
    pub fn poll(&mut self) {
        let Ok(mut file) = File::open(&self.source.path) else {
            return;
        };
        let current_size = fs::metadata(&self.source.path)
            .map(|m| m.len())
            .unwrap_or(0);
        if current_size < self.position {
            self.position = 0;
            self.lines.clear();
        }
        if current_size == self.position {
            return;
        }
        let _ = file.seek(SeekFrom::Start(self.position));
        let reader = BufReader::new(&file);
        for line in reader.lines().map_while(Result::ok) {
            self.lines.push(line);
        }
        self.position = current_size;
    }
}

fn tail_file(path: &PathBuf, n: usize) -> Vec<String> {
    let Ok(file) = File::open(path) else {
        return vec![];
    };
    let reader = BufReader::new(file);
    let all: Vec<String> = reader.lines().map_while(Result::ok).collect();
    let start = all.len().saturating_sub(n);
    all[start..].to_vec()
}
