use std::fmt::Display;

use crossterm::event::{KeyCode, KeyModifiers};

use super::KeyBindingPart;

struct DisplayableKeyCode(KeyCode);

impl Display for DisplayableKeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            KeyCode::F(n) => write!(f, "F{}", n),
            KeyCode::Char(c) => write!(f, "{}", c),
            KeyCode::Media(m) => write!(f, "Media{:?}", m),
            KeyCode::Modifier(m) => write!(f, "Modifier{:?}", m),
            k => write!(f, "{:?}", k),
        }
    }
}

impl Display for KeyBindingPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let KeyCode::Char(c) = self.code {
            if self.modifiers == KeyModifiers::NONE || self.modifiers == KeyModifiers::SHIFT {
                // it is assumed that the char is already a capital if shift was pressed
                return write!(f, "{}", c);
            }
        }
        let mod_mapping = [
            (KeyModifiers::ALT, 'A'),
            (KeyModifiers::CONTROL, 'C'),
            (KeyModifiers::HYPER, 'H'),
            (KeyModifiers::META, 'M'),
            (KeyModifiers::SHIFT, 'S'),
            (KeyModifiers::SUPER, 'W'),
        ];

        let mut contained_mods = String::new();

        for (km, letter) in mod_mapping {
            if self.modifiers.bits() & km.bits() > 0 {
                contained_mods.push(letter);
            }
        }

        if let KeyCode::Char(c) = self.code {
            let index = contained_mods.chars().position(|c| c == 'S');
            if let Some(index) = index {
                contained_mods.remove(index);
                assert!(c.is_uppercase());
            }
        }
        if contained_mods.is_empty() {
            write!(f, "<{}>", DisplayableKeyCode(self.code))
        } else {
            write!(f, "<{}-{}>", contained_mods, DisplayableKeyCode(self.code))
        }
    }
}
