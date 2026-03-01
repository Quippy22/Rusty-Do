# 🦀 Rusty-Do

---

**Rusty-Do** is a minimal, Vim-inspired terminal planner designed for hierarchical task organization. 

Heavily inspired by the card-and-list mechanics of [Trello](https://trello.com) and the [kanban-tui](https://github.com/fulsomenko/kanban) workflow, it organizes tasks into spatial, side-by-side vertical columns. This layout provides high-level project visibility while maintaining rapid, low-latency interaction and local-first persistence.

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

## 🏗️ The Hierarchy
Rusty-Do organizes tasks into a three-tier hierarchy:
1.  **Notebooks**: High-level projects or areas of focus (e.g., "Work", "Home", "Rust-Project").
2.  **Tasks**: Major milestones or categories within a notebook, displayed as vertical Kanban-style columns.
3.  **Subtasks**: Granular, actionable items within a task.

## ⌨️ Vim-Inspired Controls

### Navigation (Global)
- `h` / `l` : Move Left/Right between Notebooks or Tasks.
- `j` / `k` : Move Up/Down within lists or subtasks.
- `?` : Show the **Help Menu**.
- `Enter` : Access a selected Notebook or Inspect a Task.
- `Esc` : Return to the previous screen.
- `q` : Quit application.

### Modification
- `a` : **Append** a new item (Notebook/Task/Subtask) after the selection.
- `i` : **Insert** a new item before the selection.
- `e` : **Edit** the name of the selected item.
- `E` : Open the **Full Inspector** to edit title, description, and subtasks in split-screen.
- `D` : **Delete** the selected Task.
- `d` : **Delete** the selected Subtask.
- `r` : **Rename** the selected Notebook or Task.
- `x` : Toggle completion of a **Subtask**.
- `X` : Toggle completion of an **Entire Task** (requires confirmation).

### Inspector (Add/Edit Mode)
- `Tab` / `Shift+Tab` : Cycle between Title, Description, and List fields.
- `Enter` : Add a new entry to the list field (while focused on List).
- `Alt + Enter` : **Submit** all changes and save.
- `Esc` : Cancel and discard changes.

## ✨ Features
- **Ghost Placeholders**: See your new notebook or task appear in the background in real-time as you type its name.
- **Smart Sorting**: Your most recently accessed notebooks automatically jump to the top of the list.
- **XDG Persistence**: Data is stored safely in your standard local data directory (`~/.local/share/rusty-do/` on Linux) as clean, readable JSON.
- **Zero-Latency UI**: Optimized for high-speed interaction with immediate visual feedback.

## 🛠️ Built With
![Rust](https://img.shields.io/badge/rust-%23E32F26.svg?style=for-the-badge&logo=rust&logoColor=white)
![Ratatui](https://img.shields.io/badge/Ratatui-black?style=for-the-badge&logo=ratatui&logoColor=white)

## 📜 License
Licensed under the **MIT License**. See [LICENSE](LICENSE) for details.
