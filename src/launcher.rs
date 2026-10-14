use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::desktop_entry::DesktopEntry;

pub fn target_url_from_args<I>(args: I) -> Option<String>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let joined = args
        .into_iter()
        .map(|s| s.as_ref().to_owned())
        .collect::<Vec<_>>()
        .join(" ");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

pub fn get_target_url() -> Option<String> {
    target_url_from_args(std::env::args().skip(1))
}

pub fn launch(entry: &DesktopEntry, url: &str) -> std::io::Result<std::process::Child> {
    std::process::Command::new(&entry.program)
        .args(&entry.args)
        .arg(url)
        .spawn()
}

pub fn get_app_path() -> PathBuf {
    let apps_path = match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".config/linko"),
        Err(_) => std::env::temp_dir().join("linko"),
    };
    if !apps_path.exists() {
        if let Err(e) = create_dir_all(&apps_path) {
            eprintln!("Error: Failed to create config directory: {}", e);
        }
    }
    apps_path
}
