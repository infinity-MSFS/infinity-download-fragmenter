use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HashOutput {
    pub file_name: String,
    pub hash: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FileStructure {
    pub version: String,
    pub files: Vec<HashOutput>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchMapStructure {
    pub version: (String, String),
    pub changed_files: Vec<String>,
    pub removed_files: Vec<String>,
    pub added_files: Vec<String>,
}

impl PatchMapStructure {
    pub fn new(old_version: String, new_version: String) -> Self {
        Self {
            version: (old_version, new_version),
            changed_files: Vec::new(),
            removed_files: Vec::new(),
            added_files: Vec::new(),
        }
    }
}
