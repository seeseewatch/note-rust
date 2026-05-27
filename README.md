# NoteRust

A minimal notepad-like text editor built with Rust and [egui](https://github.com/emilk/egui).

## Features

- **File Operations** — New, Open, Save, Save As via native OS file dialogs
- **Clipboard Support** — Cut, Copy, Paste, Delete
- **Undo / Redo** — Full undo history via TextEdit
- **Line Numbers** — Toggleable gutter with gray monospace numerals
- **Word Wrap** — Toggle between wrapping and horizontal scrolling
- **Font Size** — Adjustable via `Ctrl++` / `Ctrl+-` (range: 8—32 px)
- **Dirty Indicator** — `*` prefix in the title bar when unsaved changes exist
- **Dynamic Title** — Window title updates to reflect the current file name

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New file |
| `Ctrl+O` | Open file |
| `Ctrl+S` | Save file |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+X` | Cut |
| `Ctrl+C` | Copy |
| `Ctrl+V` | Paste |
| `Del` | Delete |
| `Ctrl+A` | Select All |
| `Ctrl++` | Increase font size |
| `Ctrl+-` | Decrease font size |

## Dependencies

- **[egui / eframe](https://crates.io/crates/eframe)** — Immediate-mode GUI and native windowing
- **[rfd](https://crates.io/crates/rfd)** — Native OS file dialogs (async, non-blocking)

## Build & Run

```bash
cargo run
```

Requires Rust **1.85+** (edition 2024).
