use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemSettings {
    pub op: FileSystemOp,
    pub path: String,
    pub dest_path: String,
    pub content: String,
    pub encoding: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for FileSystemSettings {
    fn default() -> Self {
        Self {
            op: FileSystemOp::FileRead,
            path: String::new(),
            dest_path: String::new(),
            content: String::new(),
            encoding: "utf8".into(),
            output_var: "RESULT".into(),
            capture: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileSystemOp {
    CreatePath,
    FileAppend,
    FileAppendLines,
    FileCopy,
    FileMove,
    FileDelete,
    FileExists,
    FileRead,
    FileReadBytes,
    FileReadLines,
    FileWrite,
    FileWriteBytes,
    FileWriteLines,
    FolderDelete,
    FolderExists,
    GetFilesInFolder,
}
