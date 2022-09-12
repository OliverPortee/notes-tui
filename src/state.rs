use std::io::{ErrorKind, Result};
use std::{ffi::OsString, time::SystemTime};

use std::path::PathBuf;

use tui::widgets::ListState;

use crate::{
    keybindings::{example, KeyStateMachine},
    sorting::{sort_files, Sorting},
    util, CrossTerminal,
};

#[derive(Eq, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: OsString,
    pub is_folder: bool,
    pub ctime: SystemTime,
    pub mtime: SystemTime,
    pub size: u64,
}

impl PartialEq for FileInfo {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

pub struct State {
    pub cwd: PathBuf,
    files: Vec<FileInfo>,
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
            files: Vec::new(),
            list_state: ListState::default(),
            file_view_content: String::new(),
            key_state_machine: KeyStateMachine::new(example::keybindings()),
            editor,
            sorting,
            reverse_sort,
        }
    }

    pub fn update_files(&mut self) -> Result<()> {
        self.files = std::fs::read_dir(&self.cwd)?
            .filter_map(|dir_entry| Some(dir_entry.ok()?))
            .map(|dir_entry| -> Result<FileInfo> {
                let metadata = dir_entry.metadata()?;
                Ok(FileInfo {
                    path: dir_entry.path(),
                    name: dir_entry
                        .path()
                        .file_name()
                        .ok_or(std::io::Error::new(
                            ErrorKind::NotFound,
                            "could not read file name",
                        ))?
                        .into(),
                    is_folder: dir_entry.file_type()?.is_dir(),
                    ctime: metadata.created()?,
                    mtime: metadata.modified()?,
                    size: metadata.len(),
                })
            })
            .filter_map(|r| r.ok())
            .collect();
        if self.files.is_empty() {
            self.update_selection(None);
            self.update_file_view_content()?;
        }
        self.update_sort();
        Ok(())
    }

    pub fn update_sort(&mut self) {
        let f = self.selected_file().map(FileInfo::clone);
        sort_files(&mut self.files, &self.sorting);
        if self.reverse_sort {
            self.files.reverse();
        }
        if let Some(f) = f {
            let new_selection = self.files.iter().position(|other| *other == f).unwrap();
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

    pub fn selected_file(&self) -> Option<&FileInfo> {
        self.list_state.selected().map(|index| {
            assert!(index < self.files.len());
            &self.files[index]
        })
    }

    pub fn file_names(&self) -> Vec<&str> {
        self.files
            .iter()
            .filter_map(|f| f.path.file_name()?.to_str())
            .collect()
    }

    pub fn update_selection(&mut self, index: Option<usize>) {
        assert!(!self.files.is_empty() || index == None);
        if let Some(i) = index {
            assert!(i < self.files.len());
        }
        self.list_state.select(index);
    }
}

pub mod updates {

    use super::*;

    pub fn selection_down(state: &mut State, _: &mut CrossTerminal, count: usize) -> Result<()> {
        if state.files.is_empty() {
            return Ok(());
        }
        let last_index = state.files.len() - 1;
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
        if state.files.is_empty() {
            return Ok(());
        }
        match state.list_state.selected() {
            None => state.update_selection(Some(state.files.len() - 1)),
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
        if state.files.is_empty() {
            return Ok(());
        }
        let last_index = state.files.len() - 1;
        let new = if count == 0 { 0 } else { count - 1 };
        let new = if new > last_index { last_index } else { new };
        state.update_selection(Some(new));
        state.update_file_view_content()
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
        if let Some(file) = state.selected_file() {
            util::open_editor(&state.editor, vec![&file.path], terminal)?;
            state.update_files()?;
            state.update_file_view_content()?;
        }
        Ok(())
    }

    pub fn sort_by_name(state: &mut State, _: &mut CrossTerminal, _: usize) -> Result<()> {
        state.sorting = Sorting::Name;
        state.update_sort();
        Ok(())
    }

    pub fn sort_by_ctime(state: &mut State, _: &mut CrossTerminal, _: usize) -> Result<()> {
        state.sorting = Sorting::Ctime;
        state.update_sort();
        Ok(())
    }

    pub fn sort_by_mtime(state: &mut State, _: &mut CrossTerminal, _: usize) -> Result<()> {
        state.sorting = Sorting::Mtime;
        state.update_sort();
        Ok(())
    }

    pub fn sort_by_size(state: &mut State, _: &mut CrossTerminal, _: usize) -> Result<()> {
        state.sorting = Sorting::Size;
        state.update_sort();
        Ok(())
    }

    pub fn sort_by_natural(state: &mut State, _: &mut CrossTerminal, _: usize) -> Result<()> {
        state.sorting = Sorting::Natural;
        state.update_sort();
        Ok(())
    }

    pub fn reverse_sort(state: &mut State, _: &mut CrossTerminal, _: usize) -> Result<()> {
        let f = state.selected_file().map(FileInfo::clone);
        state.reverse_sort = !state.reverse_sort;
        state.files.reverse();
        if let Some(f) = f {
            let new_selection = state.files.iter().position(|other| *other == f).unwrap();
            state.update_selection(Some(new_selection));
        }
        Ok(())
    }
}
