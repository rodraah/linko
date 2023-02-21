use std::{
    fs::{create_dir_all},
    path::PathBuf,};

pub fn split_string(line: &str, cut: &str, position: usize) -> String {
    let cut = line.split_once(cut).unwrap_or_default();
    // Isn't necessary to handle the case where the string isn't cutted
    match position {
        0 => cut.0.to_string(),
        1 => cut.1.to_string(),
        _ => panic!(),
    }
}

pub fn get_app_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| { 
        println!("There's no home directory!");
        quit::with_code(0);
    });
    let apps_path = PathBuf::from(home).join(".config/linko");
    if ! apps_path.exists() {
        create_dir_all(&apps_path).unwrap_or_else(|e| {
            println!("Failed to create config directory: {}", e);
        });
    }
    apps_path
}
