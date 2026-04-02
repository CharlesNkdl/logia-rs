use crate::log::log_source::{LogLocation, LogSource};
use crate::ssh::SshClient;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ActiveLog {
    pub source: LogSource,
    pub lines: Vec<String>,
    pub scroll: u16,
}

impl ActiveLog {
    pub fn open(source: LogSource, tail: usize, ssh: Option<&SshClient>) -> Self {
        let lines = Self::load(&source, tail, ssh);
        Self {
            source,
            lines,
            scroll: 0,
        }
    }

    /// Reload the file from disk (manual refresh via `r`).
    pub fn refresh(&mut self, tail: usize, ssh: Option<&SshClient>) {
        self.lines = Self::load(&self.source, tail, ssh);
        self.scroll = 0;
    }

    fn load(source: &LogSource, tail: usize, ssh: Option<&SshClient>) -> Vec<String> {
        match &source.location {
            LogLocation::Remote(_) => {
                if let Some(client) = ssh {
                    let cmd = format!("tail -n {} {}", tail, source.path.to_string_lossy());
                    client
                        .execute(&cmd)
                        .map(|o| o.lines().map(|s| s.to_string()).collect())
                        .unwrap_or_default()
                } else {
                    vec!["Error: No SSH connection active for remote log".to_string()]
                }
            }
            LogLocation::Local => tail_file(&source.path, tail),
        }
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
