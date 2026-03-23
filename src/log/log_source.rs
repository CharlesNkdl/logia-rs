use std::{fs, path::PathBuf};

/// represent a log source
#[derive(Debug, Clone)]
pub struct LogSource {
    pub name: String,
    pub path: PathBuf,
}

pub fn discover_sources() -> Vec<LogSource> {
    let mut sources = Vec::new();
    scan_dir("/var/log", &mut sources);
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    sources
}

fn scan_dir(path: &str, sources: &mut Vec<LogSource>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(p) = path.to_str() {
                scan_dir(p, sources);
            }
            continue;
        }
        // we ignore gz and stuff , because it's  just a quick monitoring tool, not an extensive use
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy();
            if matches!(ext.as_ref(), "gz" | "bz2" | "xz" | "zst" | "zip") {
                continue;
            }
        }

        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        sources.push(LogSource { name, path });
    }
}
