use crate::{state::*, CrossTerminal};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub mod example;
mod serde;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct KeyBindingPart {
    code: KeyCode,
    modifiers: KeyModifiers,
}

#[derive(Clone)]
pub struct KeyBinding {
    keys: Vec<KeyBindingPart>,
    repeatable: bool,
    pub action:
        fn(state: &mut State, terminal: &mut CrossTerminal, count: usize) -> std::io::Result<()>,
}

pub struct KeyStateMachine {
    key_bindings: Vec<KeyBinding>,
    current_count: usize,
    pub current_keys: Vec<KeyBindingPart>,
    current_bindings: Vec<usize>,
    is_done: bool,
    key_count_after_number: usize,
}

impl KeyBindingPart {
    fn new(code: KeyCode, modifiers: KeyModifiers) -> KeyBindingPart {
        KeyBindingPart { code, modifiers }
    }

    fn new_char(c: char) -> KeyBindingPart {
        Self::new(KeyCode::Char(c), KeyModifiers::NONE)
    }
}

impl KeyBinding {
    fn new(
        keys: Vec<KeyBindingPart>,
        repeatable: bool,
        action: fn(
            state: &mut State,
            terminal: &mut CrossTerminal,
            count: usize,
        ) -> std::io::Result<()>,
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

    fn new_from_chars<S: Into<String>>(
        chars: S,
        repeatable: bool,
        action: fn(
            state: &mut State,
            terminal: &mut CrossTerminal,
            count: usize,
        ) -> std::io::Result<()>,
    ) -> Self {
        Self::new(
            chars.into().chars().map(KeyBindingPart::new_char).collect(),
            repeatable,
            action,
        )
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
            key_count_after_number: 0,
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
        
        let key_binding_part = KeyBindingPart {
            code: e.code,
            modifiers: e.modifiers,
        };

        if self.current_keys.is_empty() && e.modifiers == KeyModifiers::NONE {
            if let KeyCode::Char(char) = e.code {
                if let Some(digit) = char.to_digit(10) {
                    self.count_digit(digit);
                    self.current_keys.push(key_binding_part);
                    return None;
                }
            }
        }

        if self.key_count_after_number == 0 && self.current_count > 1 {
            self.current_bindings.retain(|binding_index| {
                let binding = &self.key_bindings[*binding_index];
                binding.repeatable
            });
        }

        self.current_bindings.retain(|binding_index| {
            let binding = &self.key_bindings[*binding_index];
            binding.keys[self.key_count_after_number] == key_binding_part
        });

        if self.current_bindings.is_empty() {
            self.reset();
            return None;
        }

        self.current_keys.push(key_binding_part);
        self.key_count_after_number += 1;

        for binding_index in self.current_bindings.iter() {
            let binding = self.key_bindings[*binding_index].clone();
            if self.key_count_after_number == binding.keys.len() {
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
        self.key_count_after_number = 0;
    }

    pub fn count(&self) -> usize {
        self.current_count
    }
}
