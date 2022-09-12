use crossterm::event::{KeyCode, KeyModifiers};

use super::{KeyBinding, KeyBindingPart};
use crate::state::updates::*;

pub fn keybindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new_from_chars("l", false, open_selected),
        KeyBinding::new_from_chars("j", true, selection_down),
        KeyBinding::new_from_chars("k", true, selection_up),
        KeyBinding::new_from_chars("o", true, open_rel_date_fwd),
        KeyBinding::new_from_chars("b", true, open_rel_date_bwd),
        KeyBinding::new_from_chars("gg", true, selection_top),
        KeyBinding::new_from_chars("sn", false, sort_by_natural),
        KeyBinding::new_from_chars("ss", false, sort_by_size),
        KeyBinding::new_from_chars("sc", false, sort_by_ctime),
        KeyBinding::new_from_chars("sm", false, sort_by_mtime),
        KeyBinding::new_from_chars("sa", false, sort_by_name),
        KeyBinding::new_from_chars("sr", false, reverse_sort),
        KeyBinding::new_from_chars("dd", true, delete_file),
        KeyBinding::new(
            vec![KeyBindingPart::new(KeyCode::Char('G'), KeyModifiers::SHIFT)],
            true,
            selection_bottom,
        ),
        KeyBinding::new(vec![
            KeyBindingPart::new(KeyCode::Enter, KeyModifiers::NONE),
            KeyBindingPart::new(KeyCode::Backspace, KeyModifiers::NONE),
            KeyBindingPart::new(KeyCode::Char('t'), KeyModifiers::CONTROL | KeyModifiers::ALT),
            KeyBindingPart::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
            KeyBindingPart::new(KeyCode::Tab, KeyModifiers::NONE),
        ], false, |_,_,_| Ok(())),
    ]
}
