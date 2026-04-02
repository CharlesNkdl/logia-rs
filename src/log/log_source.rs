use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
use crate::config::AppConfig;

#[derive(Debug, Clone, PartialEq)]
pub enum LogLocation {
    Local,
    Remote(String), // Host identifier/address
}

/// represent a log source
#[derive(Debug, Clone)]
pub struct LogSource {
    pub name: String,
    pub path: PathBuf,
    pub location: LogLocation,
}

/// A filesystem entry shown in the FileExplorer when navigating directories.
#[derive(Debug, Clone)]
pub enum FsEntry {
    Directory(PathBuf),
    LogFile(LogSource),
}

/// Lists the immediate children of `path`: directories first (sorted), then `.log` files (sorted).
/// Hidden entries and ignored directories are excluded.
pub fn list_dir(path: &Path) -> Vec<FsEntry> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut logs: Vec<LogSource> = Vec::new();

    let read = match std::fs::read_dir(path) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    for entry in read.filter_map(|e| e.ok()) {
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        // Skip hidden entries
        if name_str.starts_with('.') {
            continue;
        }
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let ep = entry.path();
        if ft.is_dir() {
            // Skip ignored directory names
            if matches!(
                name_str.as_ref(),
                "node_modules"
                    | "vendor"
                    | "target"
                    | "Library"
                    | "Applications"
                    | "System"
                    | "bin"
                    | "sbin"
                    | "dev"
                    | "proc"
                    | "sys"
            ) {
                continue;
            }
            dirs.push(ep);
        } else if ft.is_file() {
            if ep.extension().map(|e| e == "log").unwrap_or(false) {
                let name = ep
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                logs.push(LogSource {
                    name,
                    path: ep,
                    location: LogLocation::Local,
                });
            }
        }
    }

    dirs.sort();
    logs.sort_by(|a, b| a.name.cmp(&b.name));

    let mut entries: Vec<FsEntry> = dirs.into_iter().map(FsEntry::Directory).collect();
    entries.extend(logs.into_iter().map(FsEntry::LogFile));
    entries
}

/// Walks `path` up to 3 levels deep and returns all `.log` files found,
/// reusing the same ignore rules as `discover_sources`.
pub fn shallow_scan(path: &Path) -> Vec<LogSource> {
    let mut sources = Vec::new();
    let walker = WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| !is_ignored(e));

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if entry.path().extension().map(|e| e == "log").unwrap_or(false) {
                let ep = entry.path().to_path_buf();
                let name = ep
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                sources.push(LogSource {
                    name,
                    path: ep,
                    location: LogLocation::Local,
                });
            }
        }
    }

    sources.sort_by(|a, b| a.name.cmp(&b.name));
    sources.dedup_by(|a, b| a.path == b.path);
    sources
}

fn is_ignored(entry: &DirEntry) -> bool {
    let file_name = entry.file_name().to_string_lossy();
    // Ignore hidden, generic bins, system volumes and huge dependencies folders
    let is_hidden = file_name.starts_with('.') && file_name != ".";
    let is_bad_dir = matches!(
        file_name.as_ref(),
        "node_modules"
            | "vendor"
            | "target"
            | "Library"
            | "Applications"
            | "System"
            | "bin"
            | "sbin"
            | "dev"
            | "proc"
            | "sys"
    );
    is_hidden || is_bad_dir
}

pub fn discover_sources(config: &AppConfig) -> Vec<LogSource> {
    let mut sources = Vec::new();
    let paths_to_scan = if let Some(ref custom_paths) = config.local_paths {
        custom_paths.clone()
    } else {
        let mut defaults = vec![
            "/var/log".to_string(),
            "/var/www".to_string(),
            "/opt".to_string(),
        ];
        if let Some(base_dirs) = directories::BaseDirs::new() {
            if let Some(home_str) = base_dirs.home_dir().to_str() {
                defaults.push(home_str.to_string());
            }
        }
        defaults
    };
    for base_path in &paths_to_scan {
        if !PathBuf::from(base_path).exists() {
            continue;
        }
        let walker = WalkDir::new(base_path)
            .into_iter()
            .filter_entry(|e| !is_ignored(e));
        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    let ext_str = ext.to_string_lossy();
                    if ext_str == "log" {
                        let name = entry
                            .path()
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let mut display_name = name.clone();
                        if let Some(parent) = entry.path().parent() {
                            if let Some(grandparent) = parent.parent() {
                                if let Some(gp_name) = grandparent.file_name() {
                                    display_name =
                                        format!("{}/{}", gp_name.to_string_lossy(), name);
                                }
                            }
                        }
                        sources.push(LogSource {
                            name: display_name,
                            path: entry.path().to_path_buf(),
                            location: LogLocation::Local,
                        });
                    }
                }
            }
        }
    }
    sources.sort_by(|a, b| a.name.cmp(&b.name));
    sources.dedup_by(|a, b| a.path == b.path);
    sources
}
pub fn scan_remote(client: &crate::ssh::SshClient, host_name: String) -> Vec<LogSource> {
    let mut sources = Vec::new();
    let paths = vec!["/var/log", "/var/www", "/opt", "/home"];
    for path in paths {
        let cmd = format!("find {} -type f -name '*.log' 2>/dev/null", path);
        if let Ok(output) = client.execute(&cmd) {
            for line in output.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let pb = PathBuf::from(trimmed);
                    let name = pb.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let mut display_name = name.clone();
                    if let Some(parent) = pb.parent() {
                        if let Some(gp_name) = parent.parent().and_then(|p| p.file_name()) {
                            display_name = format!("{}/{}", gp_name.to_string_lossy(), name);
                        }
                    }
                    sources.push(LogSource {
                        name: display_name,
                        path: pb,
                        location: LogLocation::Remote(host_name.clone()),
                    });
                }
            }
        }
    }
    sources.sort_by(|a, b| a.name.cmp(&b.name));
    sources.dedup_by(|a, b| a.path == b.path);
    sources
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_log(dir: &std::path::Path, rel: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, "log content").unwrap();
    }

    #[test]
    fn list_dir_returns_dirs_then_logs() {
        let tmp = tempfile::Builder::new().prefix("logia_test_").tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("subdir")).unwrap();
        make_log(root, "a.log");
        make_log(root, "b.log");
        let entries = list_dir(root);
        assert!(matches!(&entries[0], FsEntry::Directory(p) if p.ends_with("subdir")));
        let log_names: Vec<_> = entries[1..]
            .iter()
            .filter_map(|e| if let FsEntry::LogFile(s) = e { Some(s.name.as_str()) } else { None })
            .collect();
        assert!(log_names.contains(&"a.log"));
        assert!(log_names.contains(&"b.log"));
    }

    #[test]
    fn list_dir_excludes_non_log_files() {
        let tmp = tempfile::Builder::new().prefix("logia_test_").tempdir().unwrap();
        let root = tmp.path();
        fs::write(root.join("readme.txt"), "text").unwrap();
        make_log(root, "app.log");
        let entries = list_dir(root);
        assert_eq!(entries.len(), 1);
        assert!(matches!(&entries[0], FsEntry::LogFile(s) if s.name == "app.log"));
    }

    #[test]
    fn shallow_scan_finds_logs_up_to_depth_3() {
        let tmp = tempfile::Builder::new()
            .prefix("logia_test_")
            .tempdir()
            .unwrap();
        let root = tmp.path();

        // depth 1
        make_log(root, "d1.log");
        // depth 2
        make_log(root, "level1/d2.log");
        // depth 3
        make_log(root, "level1/level2/d3.log");
        // depth 4 — should NOT be found
        make_log(root, "level1/level2/level3/d4.log");

        let sources = shallow_scan(root);
        let names: Vec<&str> = sources.iter().map(|s| s.name.as_str()).collect();

        assert!(names.contains(&"d1.log"), "depth 1 missing: found {:?} in {:?}", names, root);
        assert!(names.contains(&"d2.log"), "depth 2 missing");
        assert!(names.contains(&"d3.log"), "depth 3 missing");
        assert!(!names.contains(&"d4.log"), "depth 4 should be excluded");
    }

    #[test]
    fn shallow_scan_empty_dir_returns_empty() {
        let tmp = tempfile::Builder::new().prefix("logia_test_").tempdir().unwrap();
        let sources = shallow_scan(tmp.path());
        assert!(sources.is_empty());
    }
}
