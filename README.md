# 🦀 Rusty-Do

A terminal-based to-do list built in Rust.

The design is heavily inspired by the card-and-list mechanics of [Trello](https://trello.com). Instead of a traditional flat-text checklist, this application organizes tasks into side-by-side vertical columns. This spatial layout allows for categorizing tasks into different stages or groups while maintaining a clear overview of all active items within a single terminal window.

## ⚙️ The Mechanics

### Board Layout
The interface is centered around a horizontal board. The screen is divided into multiple vertical lists, each acting as a container for individual task "cards." Navigation is designed to be 2D: users move horizontally between columns to switch categories and vertically within a column to select specific tasks.

### Master-Detail Interface
Rusty-Do utilizes a split-pane layout to handle task depth:
- **The Master (Left/Center):** This area displays the board and its various lists. It provides the high-level context of where tasks sit in the workflow.
- **The Detail (Right):** Upon selecting a card, a dedicated detail pane opens on the right side of the screen. This section is used to display long-form notes, metadata, or extended descriptions that would otherwise clutter the main board view.

### Interaction
The application uses Vim-inspired keybinds for navigation and editing.

## 🏗️ Technical Architecture

The project explores several core Rust concepts and external crates:

- **Data Modeling:** Using recursive or nested structures (Vectors of Lists, which contain Vectors of Tasks) while navigating Rust’s ownership and borrowing requirements.
- **TUI Rendering (Ratatui):** Implementing complex grid-based layouts and managing terminal "rects" to ensure the side-by-side columns and detail panes resize correctly.
- **Input Handling (Crossterm):** Managing a raw mode terminal event loop to capture and process discrete keypresses for modal navigation.
- **Persistence (Serde):** Mapping the entire board state to a local `data.json` file to ensure tasks are saved and loaded across sessions.

## ⚠️ Disclaimer
This project is strictly a **learning experience**. The primary motivation is to explore the Rust language, its memory safety model, and its ecosystem for building CLI tools. It is not intended to be a production-ready project management suite or a replacement for professional software. The focus is on the process of learning how to structure a stateful, low-level application.
