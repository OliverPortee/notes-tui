use std::vec;

use crossterm::{event::{KeyCode, KeyEvent, KeyModifiers}, terminal::{disable_raw_mode, enable_raw_mode}, execute};
use tui::widgets::Clear;

use crate::{ui, CrossTerminal, state::State};

#[derive(PartialEq, Eq, Clone, Copy)]
struct KeyBindingPart {
    code: KeyCode,
    modifiers: KeyModifiers,
}

#[derive(Clone)]
pub struct KeyBinding {
    keys: Vec<KeyBindingPart>,
    repeatable: bool,
    pub action: fn(state: &mut State, terminal: &mut CrossTerminal, count: usize) -> std::io::Result<()>,
}

pub struct KeyStateMachine {
    key_bindings: Vec<KeyBinding>,
    current_count: usize,
    current_keys: Vec<KeyBindingPart>,
    current_bindings: Vec<usize>,
    is_done: bool,
}

impl KeyBindingPart {
    fn new_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyBindingPart {
        KeyBindingPart { code, modifiers }
    }

    fn new(code: KeyCode) -> KeyBindingPart {
        Self::new_with_mods(code, KeyModifiers::NONE)
    }
}

impl KeyBinding {
    fn new_multi(
        keys: Vec<KeyBindingPart>,
        repeatable: bool,
        action: fn(state: &mut State, terminal: &mut CrossTerminal, count: usize) -> std::io::Result<()>,
    ) -> Self {
        assert!(!keys.is_empty());
        if let KeyCode::Char(c) = keys[0].code {
            assert!(!c.is_digit(10));
        }
        Self {
            keys,
            repeatable,
            action,
        }
    }

    fn new_single(
        key: KeyBindingPart,
        repeatable: bool,
        action: fn(state: &mut State, terminal: &mut CrossTerminal, count: usize) -> std::io::Result<()>,
    ) -> Self {
        Self::new_multi(vec![key], repeatable, action)
    }
}

impl KeyStateMachine {
    pub fn new(key_bindings: Vec<KeyBinding>) -> KeyStateMachine {
        let len = key_bindings.len();
        KeyStateMachine {
            key_bindings,
            current_count: 0,
            current_keys: Vec::new(),
            current_bindings: (0..len).collect(),
            is_done: false,
        }
    }

    fn count_digit(&mut self, d: u32) {
        if self.current_count > 429000000 {
            return;
        }
        if self.current_count == 0 && d == 0 {
            return;
        }
        self.current_count = self.current_count * 10 + d as usize;
    }

    pub fn register_event(&mut self, e: KeyEvent) -> Option<KeyBinding> {
        if self.is_done {
            self.reset();
        }

        if self.current_keys.is_empty() && e.modifiers == KeyModifiers::NONE {
            if let KeyCode::Char(char) = e.code {
                if let Some(digit) = char.to_digit(10) {
                    self.count_digit(digit);
                    return None;
                }
            }
        }
        let key_binding_part = KeyBindingPart {
            code: e.code,
            modifiers: e.modifiers,
        };

        let index = self.current_keys.len();

        if index == 0 && self.current_count > 1 {
            self.current_bindings.retain(|binding_index| {
                let binding = &self.key_bindings[*binding_index];
                binding.repeatable
            });
        }

        self.current_bindings.retain(|binding_index| {
            let binding = &self.key_bindings[*binding_index];
            binding.keys[index] == key_binding_part
        });

        if self.current_bindings.is_empty() {
            self.reset();
            return None;
        }

        self.current_keys.push(key_binding_part);

        for binding_index in self.current_bindings.iter() {
            let binding = self.key_bindings[*binding_index].clone();
            if index + 1 == binding.keys.len() {
                self.is_done = true;
                return Some(binding);
            }
        }
        None
    }

    pub fn reset(&mut self) {
        self.current_count = 0;
        self.current_keys = Vec::new();
        self.current_bindings = (0..self.key_bindings.len()).collect();
        self.is_done = false;
    }

    pub fn count(&self) -> usize {
        return if self.current_count == 0 {
            1
        } else {
            self.current_count
        };
    }
}

pub fn make_key_sm() -> KeyStateMachine {
    let kbs: Vec<KeyBinding> = vec![
        KeyBinding::new_single(
            KeyBindingPart::new(KeyCode::Char('j')),
            true,
            |state, _, _| state.selection_down(),
        ),
        KeyBinding::new_single(
            KeyBindingPart::new(KeyCode::Char('k')),
            true,
            |state, _, _| state.selection_up(),
        ),
        KeyBinding::new_multi(
            vec![KeyBindingPart::new(KeyCode::Char('g')), KeyBindingPart::new(KeyCode::Char('g'))],
            false,
            |state, _, _| state.selection_top(),
        ),
        KeyBinding::new_single(
            KeyBindingPart::new(KeyCode::Char('l')),
            false,
            |state, terminal, _| {
                if let Some(path) = state.selected_file() {
                    disable_raw_mode()?;
                    terminal.draw(|f| f.render_widget(Clear, f.size()))?;
                    std::process::Command::new(&state.editor)
                        .arg(path.as_os_str())
                        .status()?;
                    state.update_file_view_content()?;
                    enable_raw_mode()?;
                    execute!(std::io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
                    terminal.draw(|f| ui(f, state))?;
                }
                Ok(())
            },
        ),
    ];
    KeyStateMachine::new(kbs)
}
