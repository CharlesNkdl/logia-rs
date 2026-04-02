use crate::config::AppConfig;
use crate::event::{AppEvent, Event, EventHandler};
use crate::export::snapshot;
use crate::log::log_reader::ActiveLog;
use crate::log::log_source::{FsEntry, LogSource, discover_sources, list_dir, shallow_scan};
use crate::ssh::SshClient;
use crate::ui::base::render;
use crate::ui::theme::ColorScheme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::widgets::ListState;
use std::path::PathBuf;
use tui_input::Input;

#[derive(Debug, PartialEq, Clone)]
pub enum LoadingState {
    Idle,
    Connecting,
    Scanning,
}

#[derive(Debug, PartialEq)]
pub enum AppMode {
    ServerSelect,
    FileExplorer,
    LogViewer,
    SearchPrompt,
    LogSearchPrompt,
    SshForm,
    GotoPrompt,
}

pub struct SshFormState {
    pub inputs: Vec<Input>,
    pub active_field: usize,
}

impl Default for SshFormState {
    fn default() -> Self {
        Self {
            inputs: vec![
                Input::default(), // Name
                Input::default(), // Host
                Input::default(), // Port (default 22?)
                Input::default(), // User
                Input::default(), // Key Path / Password
            ],
            active_field: 0,
        }
    }
}

pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub mode: AppMode,

    // Config and SSH
    pub config: AppConfig,
    pub ssh_client: Option<SshClient>,

    // File Explorer State
    pub sources: Vec<LogSource>,
    pub filtered_sources: Vec<LogSource>,
    pub list_state: ListState,

    // Filesystem navigation state
    pub current_dir: Option<PathBuf>,
    pub fs_entries: Vec<FsEntry>,

    // Search State
    pub search_input: Input,

    // Goto path prompt
    pub goto_input: Input,

    // Form State
    pub ssh_form: SshFormState,

    // Log Viewer State
    pub active_log: Option<ActiveLog>,

    // Theme
    pub color_scheme: ColorScheme,

    // Loading state
    pub loading: LoadingState,
    pub loading_frame: u8,
    pub status_message: Option<String>,

    // Export state
    pub export_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let sources = discover_sources(&config);
        let color_scheme = config.color_scheme.clone().unwrap_or_default();
        Self {
            running: true,
            events: EventHandler::new(),
            mode: AppMode::ServerSelect,
            color_scheme,
            config,
            ssh_client: None,
            filtered_sources: sources.clone(),
            sources,
            list_state,
            current_dir: None,
            fs_entries: Vec::new(),
            search_input: Input::default(),
            goto_input: Input::default(),
            ssh_form: SshFormState::default(),
            active_log: None,
            loading: LoadingState::Idle,
            loading_frame: 0,
            status_message: None,
            export_message: None,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| render(frame, &mut self))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => {
                if self.loading != LoadingState::Idle {
                    self.loading_frame = self.loading_frame.wrapping_add(1) % 10;
                }
            }
            Event::Crossterm(event) => {
                if let crossterm::event::Event::Key(key_event) = event {
                    if key_event.kind == crossterm::event::KeyEventKind::Press {
                        self.handle_key_event(key_event)?
                    }
                }
            }
            Event::App(AppEvent::Quit) => self.running = false,
        }
        Ok(())
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if key_event.modifiers == KeyModifiers::CONTROL
            && (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C'))
        {
            self.events.send(AppEvent::Quit);
            return Ok(());
        }
        let is_typing_mode = matches!(
            self.mode,
            AppMode::SearchPrompt
                | AppMode::LogSearchPrompt
                | AppMode::SshForm
                | AppMode::GotoPrompt
        );
        if !is_typing_mode
            && key_event.modifiers == KeyModifiers::NONE
            && key_event.code == KeyCode::Char('t')
        {
            self.color_scheme = self.color_scheme.next();
            self.config.color_scheme = Some(self.color_scheme.clone());
            let _ = self.config.save();
            return Ok(());
        }
        match self.mode {
            AppMode::ServerSelect => {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                    KeyCode::Enter => {
                        let selected = self.list_state.selected().unwrap_or(0);
                        if selected == 0 {
                            self.ssh_client = None;
                            self.loading = LoadingState::Scanning;
                            self.sources = discover_sources(&self.config);
                            self.loading = LoadingState::Idle;
                            self.current_dir = None;
                            self.fs_entries.clear();
                            self.update_search();
                            self.mode = AppMode::FileExplorer;
                        } else {
                            let server_config = &self.config.servers[selected - 1];
                            self.loading = LoadingState::Connecting;
                            match SshClient::connect(server_config) {
                                Ok(client) => {
                                    self.loading = LoadingState::Scanning;
                                    self.sources = crate::log::log_source::scan_remote(
                                        &client,
                                        server_config.name.clone(),
                                    );
                                    self.loading = LoadingState::Idle;
                                    self.status_message = None;
                                    self.ssh_client = Some(client);
                                    self.current_dir = None;
                                    self.fs_entries.clear();
                                    self.update_search();
                                    self.mode = AppMode::FileExplorer;
                                }
                                Err(e) => {
                                    self.loading = LoadingState::Idle;
                                    self.status_message = Some(format!("Connection failed: {}", e));
                                }
                            }
                        }
                        self.list_state.select(Some(0));
                    }
                    KeyCode::Char('n') | KeyCode::Char('a') => {
                        self.mode = AppMode::SshForm;
                        self.ssh_form = SshFormState::default();
                    }
                    KeyCode::Up => self.cursor_up(self.config.servers.len() + 1), // +1 for Local
                    KeyCode::Down => self.cursor_down(self.config.servers.len() + 1),
                    _ => {}
                }
            }
            AppMode::SshForm => match key_event.code {
                KeyCode::Esc => {
                    self.mode = AppMode::ServerSelect;
                }
                KeyCode::Up | KeyCode::BackTab => {
                    if self.ssh_form.active_field > 0 {
                        self.ssh_form.active_field -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Tab => {
                    if self.ssh_form.active_field < 4 {
                        self.ssh_form.active_field += 1;
                    }
                }
                KeyCode::Enter => {
                    let name = self.ssh_form.inputs[0].value().to_string();
                    let host = self.ssh_form.inputs[1].value().to_string();
                    let port = self.ssh_form.inputs[2].value().parse().unwrap_or(22);
                    let username = self.ssh_form.inputs[3].value().to_string();
                    let auth = self.ssh_form.inputs[4].value().to_string();
                    let is_pwd = !auth.contains('/') && !auth.contains('~');
                    let password = if is_pwd && !auth.is_empty() {
                        Some(auth.clone())
                    } else {
                        None
                    };
                    let key_path = if !is_pwd && !auth.is_empty() {
                        Some(auth.clone())
                    } else {
                        None
                    };
                    self.config.servers.push(crate::config::ServerConfig {
                        name,
                        host,
                        port,
                        username,
                        password,
                        key_path,
                    });
                    let _ = self.config.save();
                    self.mode = AppMode::ServerSelect;
                }
                KeyCode::Char(c) => {
                    self.ssh_form.inputs[self.ssh_form.active_field]
                        .handle(tui_input::InputRequest::InsertChar(c));
                }
                KeyCode::Backspace => {
                    self.ssh_form.inputs[self.ssh_form.active_field]
                        .handle(tui_input::InputRequest::DeletePrevChar);
                }
                KeyCode::Delete => {
                    self.ssh_form.inputs[self.ssh_form.active_field]
                        .handle(tui_input::InputRequest::DeleteNextChar);
                }
                KeyCode::Left => {
                    self.ssh_form.inputs[self.ssh_form.active_field]
                        .handle(tui_input::InputRequest::GoToPrevChar);
                }
                KeyCode::Right => {
                    self.ssh_form.inputs[self.ssh_form.active_field]
                        .handle(tui_input::InputRequest::GoToNextChar);
                }
                _ => {}
            },
            AppMode::FileExplorer => match key_event.code {
                KeyCode::Esc => {
                    self.status_message = None;
                    self.current_dir = None;
                    self.fs_entries.clear();
                    self.mode = AppMode::ServerSelect;
                }
                KeyCode::Char('/') => self.mode = AppMode::SearchPrompt,
                KeyCode::Enter => self.open_selected(),
                KeyCode::Up => {
                    let len = if self.current_dir.is_some() {
                        self.fs_entries.len()
                    } else {
                        self.filtered_sources.len()
                    };
                    self.cursor_up(len);
                }
                KeyCode::Down => {
                    let len = if self.current_dir.is_some() {
                        self.fs_entries.len()
                    } else {
                        self.filtered_sources.len()
                    };
                    self.cursor_down(len);
                }
                KeyCode::Char('-') | KeyCode::Backspace => {
                    if let Some(ref dir) = self.current_dir.clone() {
                        if let Some(parent) = dir.parent() {
                            let parent = parent.to_path_buf();
                            self.fs_entries = list_dir(&parent);
                            self.current_dir = Some(parent);
                            self.list_state.select(Some(0));
                        } else {
                            self.current_dir = None;
                            self.fs_entries.clear();
                            self.list_state.select(Some(0));
                        }
                    }
                }
                KeyCode::Char('s') => {
                    let scan_path = self.current_dir.clone().unwrap_or_else(|| {
                        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                    });
                    let found = shallow_scan(&scan_path);
                    self.sources = found.clone();
                    self.filtered_sources = found;
                    self.list_state.select(Some(0));
                    if self.current_dir.is_some() {
                        self.fs_entries = list_dir(&scan_path);
                    }
                }
                KeyCode::Char('g') => {
                    let start = self
                        .current_dir
                        .clone()
                        .or_else(|| {
                            directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf())
                        })
                        .unwrap_or_else(|| PathBuf::from("/"));
                    self.goto_input = Input::default();
                    for c in start.to_string_lossy().chars() {
                        self.goto_input
                            .handle(tui_input::InputRequest::InsertChar(c));
                    }
                    self.mode = AppMode::GotoPrompt;
                }
                _ => {}
            },
            AppMode::SearchPrompt => {
                match key_event.code {
                    KeyCode::Esc => self.mode = AppMode::FileExplorer,
                    KeyCode::Enter => self.mode = AppMode::FileExplorer, // keep search filter
                    _ => {
                        if handle_search_input(&mut self.search_input, key_event) {
                            self.update_search();
                        }
                    }
                }
            }
            AppMode::GotoPrompt => {
                match key_event.code {
                    KeyCode::Esc => {
                        self.mode = AppMode::FileExplorer;
                    }
                    KeyCode::Enter => {
                        let raw = self.goto_input.value().trim().to_string();
                        let path = PathBuf::from(&raw);
                        if path.is_dir() {
                            self.fs_entries = list_dir(&path);
                            self.current_dir = Some(path);
                            self.list_state.select(Some(0));
                            self.mode = AppMode::FileExplorer;
                        } else {
                            self.status_message = Some(format!("Not a directory: {}", raw));
                            self.mode = AppMode::FileExplorer;
                        }
                    }
                    KeyCode::Tab => {
                        // Autocomplete: find the longest common prefix among matching entries
                        let current = self.goto_input.value().to_string();
                        if let Some(completed) = tab_complete(&current) {
                            self.goto_input = Input::default();
                            for c in completed.chars() {
                                self.goto_input
                                    .handle(tui_input::InputRequest::InsertChar(c));
                            }
                        }
                    }
                    _ => {
                        handle_search_input(&mut self.goto_input, key_event);
                    }
                }
            }
            AppMode::LogViewer => match key_event.code {
                KeyCode::Esc => {
                    self.active_log = None;
                    self.export_message = None;
                    self.mode = AppMode::FileExplorer;
                }
                KeyCode::Char('/') => {
                    self.export_message = None;
                    self.search_input.reset();
                    self.mode = AppMode::LogSearchPrompt;
                }
                KeyCode::Char('r') => {
                    self.export_message = None;
                    let ssh = self.ssh_client.as_ref();
                    if let Some(log) = self.active_log.as_mut() {
                        log.refresh(500, ssh);
                    }
                }
                KeyCode::Char('e') => {
                    let lines: Vec<String> = self
                        .active_log
                        .as_ref()
                        .map(|l| l.lines.clone())
                        .unwrap_or_default();
                    let query = self.search_input.value().to_string();
                    match snapshot(&lines, &query) {
                        Ok(path) => {
                            self.export_message = Some(format!("Exported to {}", path.display()));
                        }
                        Err(e) => {
                            self.export_message = Some(format!("Export failed: {}", e));
                        }
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(log) = self.active_log.as_mut() {
                        log.scroll = log.scroll.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(log) = self.active_log.as_mut() {
                        log.scroll = log.scroll.saturating_add(1);
                    }
                }
                KeyCode::PageUp => {
                    if let Some(log) = self.active_log.as_mut() {
                        log.scroll = log.scroll.saturating_sub(20);
                    }
                }
                KeyCode::PageDown => {
                    if let Some(log) = self.active_log.as_mut() {
                        log.scroll = log.scroll.saturating_add(20);
                    }
                }
                KeyCode::Home => {
                    if let Some(log) = self.active_log.as_mut() {
                        log.scroll = 0;
                    }
                }
                KeyCode::End => {
                    if let Some(log) = self.active_log.as_mut() {
                        let total = log.lines.len() as u16;
                        log.scroll = total.saturating_sub(1);
                    }
                }
                _ => {
                    if self.export_message.is_some() {
                        self.export_message = None;
                    }
                }
            },
            AppMode::LogSearchPrompt => match key_event.code {
                KeyCode::Esc => {
                    self.mode = AppMode::LogViewer;
                    self.search_input.reset();
                }
                KeyCode::Enter => {
                    self.mode = AppMode::LogViewer;
                }
                _ => {
                    handle_search_input(&mut self.search_input, key_event);
                }
            },
        }
        Ok(())
    }

    fn update_search(&mut self) {
        let query = self.search_input.value().to_lowercase();
        if query.is_empty() {
            self.filtered_sources = self.sources.clone();
        } else {
            self.filtered_sources = self
                .sources
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&query))
                .cloned()
                .collect();
        }
        self.list_state.select(Some(0));
    }

    fn cursor_up(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn cursor_down(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i + 1 < max {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn open_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(ref dir) = self.current_dir.clone() {
                match self.fs_entries.get(i).cloned() {
                    Some(FsEntry::Directory(path)) => {
                        self.fs_entries = list_dir(&path);
                        self.current_dir = Some(path);
                        self.list_state.select(Some(0));
                    }
                    Some(FsEntry::LogFile(source)) => {
                        self.active_log =
                            Some(ActiveLog::open(source, 100, self.ssh_client.as_ref()));
                        self.mode = AppMode::LogViewer;
                    }
                    None => {}
                }
                let _ = dir;
            } else {
                // Normal flat list mode
                if let Some(source) = self.filtered_sources.get(i).cloned() {
                    self.active_log = Some(ActiveLog::open(source, 100, self.ssh_client.as_ref()));
                    self.mode = AppMode::LogViewer;
                }
            }
        }
    }
}

/// Tab-completion for filesystem paths.
/// Given a partial path string, returns the completed path if unambiguous,
/// or the longest common prefix among all matches.
pub fn tab_complete(input: &str) -> Option<String> {
    let path = PathBuf::from(input);

    // Determine the directory to list and the prefix to match
    let (dir, prefix) = if input.ends_with('/') || input.ends_with(std::path::MAIN_SEPARATOR) {
        (path.clone(), String::new())
    } else {
        let parent = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/"));
        let stem = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        (parent, stem)
    };

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return None;
    };

    let mut matches: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with(&prefix) && !name.starts_with('.')
        })
        .map(|e| {
            let p = dir.join(e.file_name());
            // Append separator for directories
            if e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let mut s = p.to_string_lossy().to_string();
                s.push('/');
                s
            } else {
                p.to_string_lossy().to_string()
            }
        })
        .collect();

    matches.sort();

    match matches.len() {
        0 => None,
        1 => Some(matches.remove(0)),
        _ => {
            let first = matches[0].clone();
            let common: String = first
                .chars()
                .enumerate()
                .take_while(|(i, c)| matches.iter().all(|m| m.chars().nth(*i) == Some(*c)))
                .map(|(_, c)| c)
                .collect();
            if common.len() > input.len() {
                Some(common)
            } else {
                None
            }
        }
    }
}
/// Returns `true` if the input value changed (i.e. the caller should re-run filtering).
pub fn handle_search_input(input: &mut Input, key: KeyEvent) -> bool {
    use tui_input::InputRequest;
    let req = match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('a') | KeyCode::Char('A')) => {
            Some(InputRequest::GoToStart)
        }
        (KeyModifiers::CONTROL, KeyCode::Char('e') | KeyCode::Char('E')) => {
            Some(InputRequest::GoToEnd)
        }
        (KeyModifiers::CONTROL, KeyCode::Char('u') | KeyCode::Char('U')) => {
            Some(InputRequest::DeleteLine)
        }
        (KeyModifiers::CONTROL, KeyCode::Char('w') | KeyCode::Char('W')) => {
            Some(InputRequest::DeletePrevWord)
        }
        (KeyModifiers::NONE, KeyCode::Home) => Some(InputRequest::GoToStart),
        (KeyModifiers::NONE, KeyCode::End) => Some(InputRequest::GoToEnd),
        (KeyModifiers::NONE, KeyCode::Char(c)) => Some(InputRequest::InsertChar(c)),
        (KeyModifiers::NONE, KeyCode::Backspace) => Some(InputRequest::DeletePrevChar),
        (KeyModifiers::NONE, KeyCode::Delete) => Some(InputRequest::DeleteNextChar),
        (KeyModifiers::NONE, KeyCode::Left) => Some(InputRequest::GoToPrevChar),
        (KeyModifiers::NONE, KeyCode::Right) => Some(InputRequest::GoToNextChar),
        _ => None,
    };
    if let Some(r) = req {
        let before = input.value().len();
        input.handle(r);
        let after = input.value().len();
        matches!(
            r,
            InputRequest::InsertChar(_)
                | InputRequest::DeletePrevChar
                | InputRequest::DeleteNextChar
                | InputRequest::DeletePrevWord
                | InputRequest::DeleteLine
        ) || before != after
    } else {
        false
    }
}
