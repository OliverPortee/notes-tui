use std::ffi::OsString;
use std::io::Result;

use std::path::PathBuf;

use tui::widgets::ListState;

use crate::{
    keybindings::{make_key_sm, KeyStateMachine},
    util, CrossTerminal,
};

pub struct State {
    pub cwd: PathBuf,
    files: Vec<PathBuf>,
    pub list_state: ListState,
    pub file_view_content: String,
    pub key_state_machine: KeyStateMachine,
    pub editor: OsString,
}

impl State {
    pub fn new(cwd: PathBuf, editor: OsString) -> Self {
        assert!(cwd.is_dir());
        State {
            cwd,
            files: Vec::new(),
            list_state: ListState::default(),
            file_view_content: String::new(),
            key_state_machine: make_key_sm(),
            editor,
        }
    }

    pub fn update_files(&mut self) -> Result<()> {
        self.files = std::fs::read_dir(&self.cwd)?
            .filter_map(|dir_entry| Some(dir_entry.ok()?.path()))
            .collect();
        if self.files.is_empty() {
            self.update_selection(None)?;
        }
        Ok(())
    }

    pub fn update_file_view_content(&mut self) -> Result<()> {
        match self.selected_file() {
            None => self.file_view_content = String::new(),
            Some(path) => self.file_view_content = std::fs::read_to_string(path)?,
        }
        Ok(())
    }

    pub fn selected_file(&self) -> Option<PathBuf> {
        self.list_state.selected().map(|index| {
            assert!(index < self.files.len());
            self.files[index].clone()
        })
    }

    pub fn file_names(&self) -> Vec<&str> {
        self.files
            .iter()
            .filter_map(|p| p.file_name()?.to_str())
            .collect()
    }

    pub fn update_selection(&mut self, index: Option<usize>) -> Result<()> {
        assert!(!self.files.is_empty() || index == None);
        if let Some(i) = index {
            assert!(i < self.files.len());
        }
        self.list_state.select(index);
        self.update_file_view_content()?;
        Ok(())
    }
}

pub fn selection_down(state: &mut State, _: &mut CrossTerminal, count: usize) -> Result<()> {
    if state.files.is_empty() {
        return Ok(());
    }
    let last_index = state.files.len() - 1;
    match state.list_state.selected() {
        None => state.update_selection(Some(0)),
        Some(i) if i == last_index => Ok(()),
        Some(i) => {
            let count = if count == 0 { 1 } else { count };
            let new = i + count;
            let new = if new > last_index { last_index } else { new };
            state.update_selection(Some(new))
        }
    }
}

pub fn selection_up(state: &mut State, _: &mut CrossTerminal, count: usize) -> Result<()> {
    if state.files.is_empty() {
        return Ok(());
    }
    match state.list_state.selected() {
        None => state.update_selection(Some(state.files.len() - 1)),
        Some(0) => Ok(()),
        Some(i) => {
            let count = if count == 0 { 1 } else { count };
            if count > i {
                state.update_selection(Some(0))
            } else {
                state.update_selection(Some(i - count))
            }
        }
    }
}

pub fn selection_top(state: &mut State, _: &mut CrossTerminal, count: usize) -> Result<()> {
    if state.files.is_empty() {
        return Ok(());
    }
    let last_index = state.files.len() - 1;
    let new = if count == 0 { 0 } else { count - 1 };
    let new = if new > last_index { last_index } else { new };
    state.update_selection(Some(new))
}

pub fn selection_bottom(state: &mut State, _: &mut CrossTerminal, count: usize) -> Result<()> {
    if state.files.is_empty() {
        return Ok(());
    }
    let last_index = state.files.len() - 1;
    let count = if count == 0 { 0 } else { count - 1 };
    let new = if count > last_index {
        0
    } else {
        last_index - count
    };
    state.update_selection(Some(new))
}

fn open_relative_date(state: &mut State, terminal: &mut CrossTerminal, offset: i64) -> Result<()> {
    let filename = util::format_date(offset);
    let mut path = state.cwd.clone();
    path.push(filename);
    path.set_extension("md");
    util::open_editor(&state.editor, vec![path], terminal)?;
    state.update_files()?;
    state.update_file_view_content()?;
    Ok(())
}

pub fn open_rel_date_fwd(
    state: &mut State,
    terminal: &mut CrossTerminal,
    offset: usize,
) -> Result<()> {
    open_relative_date(state, terminal, offset as i64)
}

pub fn open_rel_date_bwd(
    state: &mut State,
    terminal: &mut CrossTerminal,
    offset: usize,
) -> Result<()> {
    open_relative_date(state, terminal, -(offset as i64))
}

pub fn open_selected(state: &mut State, terminal: &mut CrossTerminal, _: usize) -> Result<()> {
    if let Some(path) = state.selected_file() {
        util::open_editor(&state.editor, vec![path], terminal)?;
        state.update_files()?;
        state.update_file_view_content()?;
    }
    Ok(())
}
