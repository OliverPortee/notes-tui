use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use sorting::Sorting;
use state::State;
use std::{
    io::{self, Stdout},
    path::PathBuf,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use util::fail;

mod keybindings;
mod sorting;
mod state;
mod util;

type CrossTerminal = Terminal<CrosstermBackend<Stdout>>;

fn main() -> io::Result<()> {
    util::init_logging()?;

    let folder_path: PathBuf = std::env::args()
        .nth(1)
        .unwrap_or_else(|| fail("no folder given"))
        .into();
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

    let mut state = State::new(folder_path, editor, Sorting::Natural, false);
    state.update_files()?;

    run(state, terminal)?;

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;

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
            terminal.draw(|f| ui(f, state))?;
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &State) {
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(f.size());
    let paragraph = Paragraph::new(state.cwd.as_os_str().to_string_lossy())
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(paragraph, v_chunks[0]);
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(40),
            Constraint::Length(1),
            Constraint::Percentage(60),
        ])
        .split(v_chunks[1]);
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
    let mut list_state = state.list_state.clone();
    f.render_stateful_widget(list, h_chunks[0], &mut list_state);

    let file_view_block = Block::default();
    let file_view_text = Paragraph::new(state.file_view_content.as_str()).block(file_view_block);
    f.render_widget(file_view_text, h_chunks[2]);
}
