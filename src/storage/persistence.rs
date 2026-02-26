use chrono::{DateTime, Local};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::models::notebook::Notebook;
use crate::storage::paths::FileSystem;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotebookMetadata {
    pub id: String,
    pub name: String,
    pub last_opened: DateTime<Local>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct StorageIndex {
    pub notebooks: Vec<NotebookMetadata>,
}

#[derive(Clone)]
pub struct Persistence {
    pub fs: FileSystem,
}

impl Persistence {
    pub fn new(fs: FileSystem) -> Self {
        Self { fs }
    }

    pub fn load_index(&self) -> Result<StorageIndex> {
        if !self.fs.index_path.exists() {
            return Ok(StorageIndex::default());
        }

        let content = fs::read_to_string(&self.fs.index_path)?;
        let index: StorageIndex = serde_json::from_str(&content)?;
        Ok(index)
    }

    pub fn save_index(&self, index: &StorageIndex) -> Result<()> {
        let content = serde_json::to_string_pretty(index)?;
        fs::write(&self.fs.index_path, content)?;
        Ok(())
    }

    pub fn load_notebook(&self, id: &str) -> Result<Notebook> {
        let path = self.fs.get_notebook_path(id);
        let content = fs::read_to_string(path)?;
        let notebook: Notebook = serde_json::from_str(&content)?;
        Ok(notebook)
    }

    pub fn save_notebook(&self, notebook: &Notebook) -> Result<()> {
        let path = self.fs.get_notebook_path(&notebook.id);
        let content = serde_json::to_string_pretty(notebook)?;
        fs::write(path, content)?;
        Ok(())
    }
}

impl Persistence {
    /// Scans the notebooks sub-directory, verifies notebook files, and syncs index.json
    pub fn validate_and_sync_index(&self) -> Result<StorageIndex> {
        let mut index = self.load_index().unwrap_or_default();
        let entries = fs::read_dir(&self.fs.notebooks_dir)?;

        let mut actual_notebooks = Vec::new();

        for entry in entries {
            let path = entry?.path();

            // Only look at .json files in the notebooks folder
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                // Verify it's actually a valid Notebook
                if let Ok(notebook) = self.verify_notebook_file(&path) {
                    actual_notebooks.push(notebook);
                }
            }
        }

        // 1. Remove entries from index that no longer exist on disk (based on ID)
        index
            .notebooks
            .retain(|meta| actual_notebooks.iter().any(|nb| nb.id == meta.id));

        // 2. Add new files to index or update existing names
        for nb in actual_notebooks {
            if let Some(meta) = index.notebooks.iter_mut().find(|m| m.id == nb.id) {
                // Update display name in case it was changed inside the file
                meta.name = nb.name;
            } else {
                // New notebook file discovered
                index.notebooks.push(NotebookMetadata {
                    id: nb.id,
                    name: nb.name,
                    last_opened: Local::now(),
                });
            }
        }

        // 3. Sort by last_opened (Descending: newest first)
        index
            .notebooks
            .sort_by(|a, b| b.last_opened.cmp(&a.last_opened));

        self.save_index(&index)?;
        Ok(index)
    }

    pub fn update_last_opened(&self, id: &str) -> Result<()> {
        let mut index = self.load_index()?;
        if let Some(meta) = index.notebooks.iter_mut().find(|m| m.id == id) {
            meta.last_opened = Local::now();
            index
                .notebooks
                .sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
            self.save_index(&index)?;
        }
        Ok(())
    }

    fn verify_notebook_file(&self, path: &PathBuf) -> Result<Notebook> {
        let content = fs::read_to_string(path)?;
        let notebook: Notebook = serde_json::from_str(&content)?;
        Ok(notebook)
    }
}
