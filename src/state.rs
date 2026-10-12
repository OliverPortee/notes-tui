use std::ffi::OsString;
use std::io::Result;

use std::path::PathBuf;

use chrono::NaiveDate;
use tui::widgets::ListState;

use crate::{
    keybindings::{example, KeyStateMachine},
    sorting::{sort_files, Sorting},
    util, CrossTerminal,
};

const DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Eq, Clone)]
pub struct DiaryEntry {
    pub path: PathBuf,
    pub date: NaiveDate,
    pub size: u64,
}

impl PartialEq for DiaryEntry {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

pub struct State {
    pub cwd: PathBuf,
    entries: Vec<DiaryEntry>,
    pub list_state: ListState,
    pub file_view_content: String,
    pub key_state_machine: KeyStateMachine,
    editor: OsString,
    sorting: Sorting,
    reverse_sort: bool,
}

impl State {
    pub fn new(cwd: PathBuf, editor: OsString, sorting: Sorting, reverse_sort: bool) -> Self {
        assert!(cwd.is_dir());
        State {
            cwd,
            entries: Vec::new(),
            list_state: ListState::default(),
            file_view_content: String::new(),
            key_state_machine: KeyStateMachine::new(example::keybindings()),
            editor,
            sorting,
            reverse_sort,
        }
    }

    pub fn update_files(&mut self) -> Result<()> {
        self.entries = std::fs::read_dir(&self.cwd)?
            .filter_map(|dir_entry| dir_entry.ok())
            .filter(|dir_entry| match dir_entry.file_type() {
                Ok(ft) => ft.is_file(),
                _ => false,
            })
            .filter_map(|dir_entry| {
                let path = dir_entry.path();
                let file_name = path.file_name()?.to_string_lossy();
                Some(DiaryEntry {
                    path: dir_entry.path(),
                    date: NaiveDate::parse_from_str(&file_name, DATE_FORMAT).ok()?,
                    size: dir_entry.metadata().ok()?.len(),
                })
            })
            .collect();
        if self.entries.is_empty() {
            self.update_selection(None);
        } else if let Some(index) = self.list_state.selected() {
            if index >= self.entries.len() {
                self.update_selection(Some(self.entries.len() - 1));
            }
        }
        self.update_sort();
        Ok(())
    }

    pub fn update_sort(&mut self) {
        let path = self.selected_file().map(|f| f.path.clone());
        sort_files(&mut self.entries, &self.sorting);
        if self.reverse_sort {
            self.entries.reverse();
        }
        if let Some(path) = path {
            let new_selection = self
                .entries
                .iter()
                .position(|other| other.path == path)
                .unwrap();
            self.update_selection(Some(new_selection));
        }
    }

    pub fn update_file_view_content(&mut self) -> Result<()> {
        match self.selected_file() {
            None => self.file_view_content = String::new(),
            Some(file) => self.file_view_content = std::fs::read_to_string(&file.path)?,
        }
        Ok(())
    }

    pub fn selected_file(&self) -> Option<&DiaryEntry> {
        self.list_state.selected().map(|index| {
            assert!(index < self.entries.len());
            &self.entries[index]
        })
    }

    pub fn entry_names(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|e| e.date.format(DATE_FORMAT).to_string())
            .collect()
    }

    pub fn update_selection(&mut self, index: Option<usize>) {
        assert!(!self.entries.is_empty() || index == None);
        if let Some(i) = index {
            assert!(i < self.entries.len());
        }
        self.list_state.select(index);
    }
}

pub mod updates {

    use super::*;

    pub fn selection_down(state: &mut State, _: &mut CrossTerminal, count: usize) -> Result<()> {
        if state.entries.is_empty() {
            return Ok(());
        }
        let last_index = state.entries.len() - 1;
        match state.list_state.selected() {
            None => state.update_selection(Some(0)),
            Some(i) if i == last_index => {}
            Some(i) => {
                let count = if count == 0 { 1 } else { count };
                let new = i + count;
                let new = if new > last_index { last_index } else { new };
                state.update_selection(Some(new))
            }
        }
        state.update_file_view_content()
    }

    pub fn selection_up(state: &mut State, _: &mut CrossTerminal, count: usize) -> Result<()> {
        if state.entries.is_empty() {
            return Ok(());
        }
        match state.list_state.selected() {
            None => state.update_selection(Some(state.entries.len() - 1)),
            Some(0) => {}
            Some(i) => {
                let count = if count == 0 { 1 } else { count };
                if count > i {
                    state.update_selection(Some(0))
                } else {
                    state.update_selection(Some(i - count))
                }
            }
        }
        state.update_file_view_content()
    }

    pub fn selection_top(state: &mut State, _: &mut CrossTerminal, count: usize) -> Result<()> {
        if state.entries.is_empty() {
            return Ok(());
        }
        let last_index = state.entries.len() - 1;
        let new = if count == 0 { 0 } else { count - 1 };
        let new = if new > last_index { last_index } else { new };
        state.update_selection(Some(new));
        state.update_file_view_content()
    }

    pub fn selection_bottom(state: &mut State, _: &mut CrossTerminal, count: usize) -> Result<()> {
        if state.entries.is_empty() {
            return Ok(());
        }
        let last_index = state.entries.len() - 1;
        let count = if count == 0 { 0 } else { count - 1 };
        let new = if count > last_index {
            0
        } else {
            last_index - count
        };
        state.update_selection(Some(new));
        state.update_file_view_content()
    }

    fn open_relative_date(
        state: &mut State,
        terminal: &mut CrossTerminal,
        offset: i64,
    ) -> Result<()> {
        let filename = util::format_date(offset);
        let mut path = state.cwd.clone();
        path.push(filename);
        path.set_extension("md");
        util::open_editor(&state.editor, vec![&path], terminal, &state.cwd)?;
        state.update_files()?;
        let index = state.entries.iter().position(|f| f.path == path);
        if let Some(index) = index {
            state.update_selection(Some(index));
        }
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
        if let Some(file) = state.selected_file() {
            let path = file.path.clone();
            util::open_editor(&state.editor, vec![&file.path], terminal, &state.cwd)?;
            state.update_files()?;
            let index = state.entries.iter().position(|f| f.path == path);
            if let Some(index) = index {
                state.update_selection(Some(index));
            }
            state.update_file_view_content()?;
        }
        Ok(())
    }

    pub fn sort_by_date(state: &mut State, _: &mut CrossTerminal, _: usize) -> Result<()> {
        state.sorting = Sorting::Date;
        state.update_sort();
        Ok(())
    }

    pub fn sort_by_size(state: &mut State, _: &mut CrossTerminal, _: usize) -> Result<()> {
        state.sorting = Sorting::Size;
        state.update_sort();
        Ok(())
    }

    pub fn reverse_sort(state: &mut State, _: &mut CrossTerminal, _: usize) -> Result<()> {
        let f = state.selected_file().map(DiaryEntry::clone);
        state.reverse_sort = !state.reverse_sort;
        state.entries.reverse();
        if let Some(f) = f {
            let new_selection = state.entries.iter().position(|other| *other == f).unwrap();
            state.update_selection(Some(new_selection));
        }
        Ok(())
    }

    pub fn delete_file(state: &mut State, _: &mut CrossTerminal, count: usize) -> Result<()> {
        let count = if count == 0 { 1 } else { count };
        for _ in 0..count {
            if let Some(index) = state.list_state.selected() {
                let file = &state.entries[index];
                std::fs::remove_file(&file.path)?;
                state.entries.remove(index);
                if state.entries.is_empty() {
                    state.update_selection(None);
                    break;
                }
                if index >= state.entries.len() {
                    state.update_selection(Some(state.entries.len() - 1));
                }
            }
        }
        state.update_file_view_content()?;

        Ok(())
    }
}
