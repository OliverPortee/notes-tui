
# small stuff
* [✓] logging
* [✓] expand ~
* [✓] state -> separate file
* [✓] use $VISUAL/$EDITOR
* [✓] don't leave any rendered content when exiting
* [ ] README.md
* [ ] cli
* [ ] scrolling
* [ ] create folder if not existing
* [ ] immutable state in ui() (only after scrolling)

# design decisions
* [ ] more general as a file browser or only as notes/diary

# features

## keybindings
* [✓] multiple key keybings (gg)
* [✓] keybinding combinations (C-s)
* [ ] keybinding configuration via toml

## folders

## search

## sort

## syntax highlight

# keybindings

<num>j: selection down
<num>k: selection up
l: open
h: exit folder (only when folders implemented)
gg: to the top
G: to the bottom
<C-u>: half page up
<C-d>: half page down
dd: delete
a: rename after filename
A: rename after extension
/: search
Esc: escape command
