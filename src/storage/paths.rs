use std::fs;
use std::path::PathBuf;

use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use directories::ProjectDirs;

#[derive(Clone)]
pub struct FileSystem {
    pub data_dir: PathBuf,
    pub notebooks_dir: PathBuf,
    pub index_path: PathBuf,
}

impl FileSystem {
    pub fn new() -> Result<Self> {
        // 1. Get standard project directories
        let proj_dirs = ProjectDirs::from("", "", "rusty-do")
            .ok_or_eyre("Could not determine project directories")?;

        // 2. Define the paths
        let data_dir = proj_dirs.data_dir().to_path_buf();
        let notebooks_dir = data_dir.join("notebooks");
        let index_path = data_dir.join("index.json");

        // 3. Ensure the folders exist
        fs::create_dir_all(&notebooks_dir)?;

        Ok(Self {
            data_dir,
            notebooks_dir,
            index_path,
        })
    }

    /// Returns the full path to a notebook's JSON file: ~/.local/share/rusty-do/notebooks/ID.json
    pub fn get_notebook_path(&self, id: &str) -> PathBuf {
        self.notebooks_dir.join(format!("{}.json", id))
    }
}
