use std::fs::{read_dir, remove_file, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::imgops;

pub fn delete_template(path: &Path, name: String) -> Result<(), String> {
    let mut file_path = PathBuf::from(path);
    file_path.push(name);

    match remove_file(file_path) {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to delete template {}", e)),
    }
}

pub fn list_templates(path: &Path) -> Result<Vec<String>, String> {
    match read_dir(path) {
        Ok(entries) => {
            let mut templates = Vec::new();

            for entry in entries {
                if let Ok(entry) = entry {
                    let epath = entry.path();
                    if !epath.is_file() {
                        continue;
                    }

                    match epath.file_name() {
                        Some(file_name) => {
                            let file_name: String = file_name.to_string_lossy().into();

                            if imgops::is_image(&epath) {
                                templates.push(file_name);
                            }
                        },
                        None => (),
                    }
                }
            }

            Ok(templates)
        },
        Err(e) => Err(format!("Failed to read templates directory {}: {}", path.display(), e)),
    }
}

pub fn read_template(path: &Path, name: String) -> Result<Vec<u8>, String> {
    let mut file_path = PathBuf::from(path);
    file_path.push(name);

    let mut fh = match File::open(file_path) {
        Ok(fh) => fh,
        Err(e) => return Err(format!("Failed to open template file: {}", e)),
    };

    let mut buf = Vec::new();
    match fh.read_to_end(&mut buf) {
        Ok(_) => Ok(buf),
        Err(e) => Err(format!("Failed to read template file: {}", e)),
    }
}

pub fn write_template(path: &Path, name: String, data: &[u8]) -> Result<(), String> {
    let mut file_path = PathBuf::from(path);
    file_path.push(name);

    let mut fh = match File::create(file_path) {
        Ok(fh) => fh,
        Err(e) => return Err(format!("Failed to open template file: {}", e)),
    };

    match fh.write_all(data) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write template file: {}", e)),
    }
}
