use std::path::PathBuf;

pub struct DesktopEntry {
    pub name: String,
    pub icon: String,
    pub program: String,
    pub args: Vec<String>,
}

pub struct BrowserCandidate {
    pub name: String,
    pub icon: String,
    pub file_name: String,
    pub source_path: PathBuf,
}
