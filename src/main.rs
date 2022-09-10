use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::{
    io,
    path::{Path, PathBuf},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

struct State {
    cwd: PathBuf,
    files: Vec<PathBuf>,
    list_state: ListState,
    file_view_content: String,
}

impl State {
    fn new(cwd: PathBuf) -> Self {
        assert!(cwd.is_dir());
        State {
            cwd,
            files: Vec::new(),
            list_state: ListState::default(),
            file_view_content: String::new(),
        }
    }

    fn update_files(&mut self) -> io::Result<()> {
        self.files = std::fs::read_dir(&self.cwd)?
            .filter_map(|dir_entry| Some(dir_entry.ok()?.path()))
            .collect();
        if self.files.is_empty() {
            self.update_selection(None)?;
        }
        Ok(())
    }

    fn update_file_view_content(&mut self) -> io::Result<()> {
        match self.selected_file() {
            None => self.file_view_content = String::new(),
            Some(path) => self.file_view_content = std::fs::read_to_string(path)?,
        }
        Ok(())
    }

    fn selected_file(&self) -> Option<&Path> {
        self.list_state.selected().map(|index| {
            assert!(index < self.files.len());
            self.files[index].as_path()
        })
    }

    fn file_names(&self) -> Vec<&str> {
        self.files
            .iter()
            .filter_map(|p| p.file_name()?.to_str())
            .collect()
    }

    fn update_selection(&mut self, index: Option<usize>) -> io::Result<()> {
        assert!(!self.files.is_empty() || index == None);
        if let Some(i) = index {
            assert!(i < self.files.len());
        }
        self.list_state.select(index);
        self.update_file_view_content()?;
        Ok(())
    }

    fn selection_down(&mut self) -> io::Result<()> {
        if self.files.is_empty() {
            return self.update_selection(None);
        }
        match self.list_state.selected() {
            None => self.update_selection(Some(0)),
            Some(i) if i == self.files.len() - 1 => Ok(()),
            Some(i) => self.update_selection(Some(i + 1)),
        }
    }

    fn selection_up(&mut self) -> io::Result<()> {
        if self.files.is_empty() {
            return self.update_selection(None);
        }
        match self.list_state.selected() {
            None => self.update_selection(Some(self.files.len() - 1)),
            Some(0) => Ok(()),
            Some(i) => self.update_selection(Some(i - 1)),
        }
    }
}

fn main() -> io::Result<()> {
    init_logging()?;

    let folder = std::env::args().nth(1).expect("no folder given");
    let folder_path = std::path::PathBuf::from(folder);
    if !folder_path.is_dir() {
        panic!(
            "path {} is not a directory",
            folder_path.into_os_string().into_string().unwrap()
        );
    }
    let mut state = State::new(folder_path);
    state.update_files()?;

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run(&mut terminal, state)?;

    execute!(io::stdout(), crossterm::terminal::Clear(ClearType::All))?;
    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    Ok(())
}

fn init_logging() -> std::io::Result<()> {
    if let Ok(log_file) = std::env::var("LOG_FILE") {
        let path = PathBuf::from(log_file.clone());
        if path.exists() {
            assert!(path.is_file(), "log file is not a file");
            std::fs::remove_file(path)?;
        }
        let config = simple_log::LogConfigBuilder::builder()
            .path(log_file)
            .time_format("")
            .output_file()
            .build();
        simple_log::new(config).expect("couldn't set up log file");
    }
    Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>, mut state: State) -> io::Result<()> {
    terminal.draw(|f| ui(f, &mut state))?;
    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('j') => {
                    state.selection_down()?;
                }
                KeyCode::Char('k') => {
                    state.selection_up()?;
                }
                KeyCode::Char('l') => {
                    if let Some(path) = state.selected_file() {
                        disable_raw_mode()?;
                        terminal.draw(|f| f.render_widget(Clear, f.size()))?;
                        std::process::Command::new("nvim")
                            .arg(path.as_os_str())
                            .status()?;
                        state.update_file_view_content()?;
                        enable_raw_mode()?;
                        execute!(io::stdout(), crossterm::terminal::Clear(ClearType::All))?;
                        terminal.draw(|f| ui(f, &mut state))?;
                    }
                }
                _ => {}
            }
        }
        terminal.draw(|f| ui(f, &mut state))?;
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &mut State) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(f.size());
    let file_list_block = Block::default().title("files").borders(Borders::ALL);
    let list_items: Vec<ListItem> = state.file_names().into_iter().map(ListItem::new).collect();
    let list = List::new(list_items)
        .block(file_list_block)
        .highlight_style(
            Style::default()
                .bg(Color::Gray)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    // TODO: clone?!
    f.render_stateful_widget(list, chunks[0], &mut state.list_state.clone());

    let file_view_block = Block::default().borders(Borders::ALL);
    let file_view_text = Paragraph::new(state.file_view_content.as_str()).block(file_view_block);
    f.render_widget(file_view_text, chunks[1]);
}
