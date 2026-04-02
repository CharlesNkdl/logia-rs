use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Writes the currently visible (optionally filtered) log lines to a timestamped snapshot file
/// in the current working directory.
///
/// If `query` is non-empty, only lines containing the query (case-insensitive) are written.
/// Returns the path of the created file on success.
pub fn snapshot(lines: &[String], query: &str) -> Result<PathBuf, std::io::Error> {
    let filtered: Vec<&String> = if query.is_empty() {
        lines.iter().collect()
    } else {
        let q = query.to_lowercase();
        lines
            .iter()
            .filter(|l| l.to_lowercase().contains(&q))
            .collect()
    };

    let filename = format!("logia-snapshot-{}.log", timestamp_str());
    let path = std::env::current_dir()?.join(&filename);

    let mut file = std::fs::File::create(&path)?;
    for line in filtered {
        writeln!(file, "{}", line)?;
    }

    Ok(path)
}

/// Returns a timestamp string in `YYYY-MM-DD_HH-MM-SS` format using system time.
fn timestamp_str() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let (year, month, day, hour, min, sec) = secs_to_datetime(secs);
    format!(
        "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
        year, month, day, hour, min, sec
    )
}

/// Converts Unix seconds to (year, month, day, hour, minute, second) without external crates.
fn secs_to_datetime(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let sec = (secs % 60) as u32;
    let mins = secs / 60;
    let min = (mins % 60) as u32;
    let hours = mins / 60;
    let hour = (hours % 24) as u32;
    let days = (hours / 24) as u32;

    // Compute year from days since epoch (1970-01-01)
    let mut year = 1970u32;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let month_days: [u32; 12] = [
        31,
        if is_leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u32;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        month += 1;
    }
    let day = remaining + 1;

    (year, month, day, hour, min, sec)
}

fn is_leap(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn snapshot_writes_filtered_lines() {
        let lines = vec![
            "INFO: server started".to_string(),
            "ERROR: disk full".to_string(),
            "INFO: request received".to_string(),
            "DEBUG: cache miss".to_string(),
        ];

        let path = snapshot(&lines, "info").expect("snapshot should succeed");
        assert!(path.exists());

        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        assert!(filename.starts_with("logia-snapshot-"));
        assert!(filename.ends_with(".log"));

        let content = fs::read_to_string(&path).expect("should read file");
        let written: Vec<&str> = content.lines().collect();
        assert_eq!(written.len(), 2);
        assert!(written[0].contains("INFO: server started"));
        assert!(written[1].contains("INFO: request received"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn snapshot_writes_all_lines_when_no_query() {
        let lines = vec!["line one".to_string(), "line two".to_string()];
        let path = snapshot(&lines, "").expect("snapshot should succeed");
        let content = fs::read_to_string(&path).expect("should read file");
        assert_eq!(content.lines().count(), 2);
        let _ = std::fs::remove_file(&path);
    }
}
