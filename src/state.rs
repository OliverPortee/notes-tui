use std::io::Result;

use std::path::PathBuf;

use tui::widgets::ListState;

use crate::keybindings::{KeyStateMachine, make_key_sm};


pub struct State {
    cwd: PathBuf,
    files: Vec<PathBuf>,
    pub list_state: ListState,
    pub file_view_content: String,
    pub key_state_machine: KeyStateMachine,
}

impl State {
    pub fn new(cwd: PathBuf) -> Self {
        assert!(cwd.is_dir());
        State {
            cwd,
            files: Vec::new(),
            list_state: ListState::default(),
            file_view_content: String::new(),
            key_state_machine: make_key_sm(),
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

    pub fn selection_down(&mut self) -> Result<()> {
        if self.files.is_empty() {
            return Ok(());
        }
        match self.list_state.selected() {
            None => self.update_selection(Some(0)),
            Some(i) if i == self.files.len() - 1 => Ok(()),
            Some(i) => self.update_selection(Some(i + 1)),
        }
    }

    pub fn selection_up(&mut self) -> Result<()> {
        if self.files.is_empty() {
            return Ok(());
        }
        match self.list_state.selected() {
            None => self.update_selection(Some(self.files.len() - 1)),
            Some(0) => Ok(()),
            Some(i) => self.update_selection(Some(i - 1)),
        }
    }

    pub fn selection_top(&mut self) -> Result<()> {
        if self.files.is_empty() {
            return Ok(());
        }
        self.update_selection(Some(0))
    }
}