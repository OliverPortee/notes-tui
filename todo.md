
# small stuff
* [✓] logging
* [✓] expand ~
* [✓] state -> separate file
* [✓] use $VISUAL/$EDITOR
* [✓] don't leave any rendered content when exiting
* [✓] header with folder name
* [✓] new files
* [ ] footer for commands/debug text
* [ ] README.md
* [ ] help text
* [ ] scrolling
* [ ] create folder if not existing
* [ ] immutable state in ui() (only after scrolling)
* [ ] change nvim's cwd
* [ ] directly open file from cli (today, the last one, tomorrow, specific date)
* [ ] show current key chain in footer
* [ ] remove tilde expansion? (done automatically by shell)
* [ ] select the right file when creating a new one
* [ ] keybinding actions as non-member functions
* [ ] go to specific year or month or day

# design decisions
* [ ] more general as a file browser or only as notes/diary

# features

## keybindings
* [✓] multiple key keybings (gg)
* [✓] keybinding combinations (C-s)
* [ ] make repetition work
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

