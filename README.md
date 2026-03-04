# 🦀 Rusty-Do


**Rusty-Do** is a minimal, Vim-inspired terminal planner designed for hierarchical task organization. 

Heavily inspired by the card-and-list mechanics of [Trello](https://trello.com) and the [kanban-tui](https://github.com/fulsomenko/kanban) workflow, it organizes tasks into spatial, side-by-side vertical columns. This layout provides high-level project visibility while maintaining rapid, low-latency interaction and local-first persistence.

## ✨ New in v2.0.0
- **Professional Markdown**: Opt-in markdown rendering for detailed task descriptions.
- **Global Theme Engine**: Live theme switching (`Alt+T`) and full JSON customization.
- **Spatial Reordering**: Move tasks and subtasks on the fly using `Shift + H/J/K/L`.
- **Compact Layout**: Optimized Kanban columns for maximum visibility on smaller terminals.
- **Automated Releases**: Native binaries for Linux, macOS, and Windows available on GitHub.


## 🚀 Quick Start

### Installation
Install via [crates.io](https://crates.io/crates/rusty-do):
```bash
cargo install rusty-do
```

### Run
Launch the application by typing the command in your terminal:
```bash
rsdo
```
### Note: Once inside the app, press `?` at any time to see the full list of keybindings. These are window-dependent and will show relevant keys for your current view.


## 📝 Markdown Support
Rusty-Do now supports professional markdown rendering in task descriptions.
- **How to use**: Start any description with the tag **`!markdown`**.
- **Transparent Editing**: The tag is automatically managed; you only see your clean, formatted notes in View mode.
- **Supported Syntax**: Headers (`#`), Bold (`**`), Italics (`*`), and Lists are all rendered via the integrated parser.


## 🎨 Theming
The app is fully customizable through a global theme engine.
- **Live Switching**: Press **`Alt + T`** to cycle between built-in presets (Tailwind, Nord).
- **Custom Themes**: Create a `theme.json` in your data directory (`~/.local/share/rusty-do/`) to define your own colors using Hex codes.


## ⌨️ Categorized Controls

### Navigation & Global
- `h` / `l` : Move Left/Right between Notebooks or Tasks.
- `j` / `k` : Move Up/Down within lists or subtasks.
- `Alt + T` : **Cycle Theme** live.
- `?` : Show the **Help Menu**.
- `q` : Quit application.

### Movement (Board)
- `Shift + H / L` : **Swap Task Column** Left or Right.
- `Shift + J / K` : **Move Subtask** Down or Up.

### Modification
- `A` / `I` : Add Task after/before selection.
- `a` / `i` : Add Subtask after/before selection.
- `E` / `Enter` : Open the **Full Inspector** (View/Edit mode).
- `r` : Rename selected Notebook or Task.
- `e` : Rename selected Subtask.
- `D` : **Delete** the selected Task.
- `d` : **Delete** the selected Subtask.
- `x` / `X` : Toggle completion status.

### Inspector (Add/Edit Mode)
- `Tab` / `Shift + Tab` : Cycle between Title, Description, and List fields.
- `Arrows` / `Home` / `End` : Precise cursor navigation within fields.
- `Ctrl + S` / `Alt + Enter` : **Submit** and save changes.
- `Esc` : Cancel and discard changes.


## 🛠️ Built With
![Rust](https://img.shields.io/badge/rust-%23E32F26.svg?style=for-the-badge&logo=rust&logoColor=white)
![Ratatui](https://img.shields.io/badge/Ratatui-black?style=for-the-badge&logo=ratatui&logoColor=white)

## 📜 License
Licensed under the **MIT License**. See [LICENSE](LICENSE) for details.
