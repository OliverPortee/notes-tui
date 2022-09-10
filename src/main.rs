use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use home_dir::HomeDirExt;
use state::State;
use std::{
    io::{self, Stdout},
    path::PathBuf,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListItem, Paragraph},
    Frame, Terminal,
};

mod keybindings;
mod state;

type CrossTerminal = Terminal<CrosstermBackend<Stdout>>;

fn fail<T, S: AsRef<str>>(msg: S) -> T {
    eprintln!("{}", msg.as_ref());
    std::process::exit(1);
}

fn main() -> io::Result<()> {
    init_logging()?;

    let folder = std::env::args()
        .nth(1)
        .unwrap_or_else(|| fail("no folder given"));
    let folder_path = std::path::PathBuf::from(folder)
        .expand_home()
        .unwrap_or_else(|_| fail("could not find out HOME directory"));
    if !folder_path.is_dir() {
        fail::<(), String>(format!(
            "path {} is not a directory",
            folder_path.clone().into_os_string().into_string().unwrap()
        ));
    }

    // setup terminal
    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;

    let editor = std::env::var_os("VISUAL")
        .or_else(|| std::env::var_os("EDITOR"))
        .unwrap_or_else(|| fail("could not find $VISUAL or $EDITOR"));

    let mut state = State::new(folder_path, editor);
    state.update_files()?;

    run(state, terminal)?;

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

fn init_logging() -> std::io::Result<()> {
    if let Ok(log_file) = std::env::var("LOG_FILE") {
        let path = PathBuf::from(log_file.clone());
        if path.exists() {
            if !path.is_file() {
                fail::<(), &str>("log file is not a file");
            }
            std::fs::remove_file(path)?;
        }
        let config = simple_log::LogConfigBuilder::builder()
            .path(log_file)
            .time_format("")
            .output_file()
            .build();
        simple_log::new(config).unwrap_or_else(|_| fail("couldn't set up log file"));
    }
    Ok(())
}

fn run(mut state: State, mut terminal: CrossTerminal) -> io::Result<()> {
    let state = &mut state;
    let terminal = &mut terminal;
    terminal.draw(|f| ui(f, state))?;
    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc => {
                    state.key_state_machine.reset();
                }
                _ => {
                    let result = state.key_state_machine.register_event(key);
                    if let Some(kb) = result {
                        let count = state.key_state_machine.count();
                        (kb.action)(state, terminal, count)?;
                    }
                }
            }
        }
        terminal.draw(|f| ui(f, state))?;
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &mut State) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Length(1),
                Constraint::Percentage(60),
            ]
            .as_ref(),
        )
        .split(f.size());
    let file_list_block = Block::default();
    let list_items: Vec<ListItem> = state.file_names().into_iter().map(ListItem::new).collect();
    let list = List::new(list_items)
        .block(file_list_block)
        .highlight_style(
            Style::default()
                .bg(Color::Gray)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
    // TODO: clone?!
    f.render_stateful_widget(list, chunks[0], &mut state.list_state.clone());

    let file_view_block = Block::default();
    let file_view_text = Paragraph::new(state.file_view_content.as_str()).block(file_view_block);
    f.render_widget(file_view_text, chunks[2]);
}
