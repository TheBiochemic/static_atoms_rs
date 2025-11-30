use std::{env, fs};

use crate::{Configuration, filetype::FileType};

#[cfg(test)]
mod tests_markdown;

#[cfg(test)]
mod tests_html;

pub fn get_config() -> Configuration {
    Configuration {
        root: env::temp_dir().join("static_atoms_rs_tests"),
        out: None,
        input_files: Vec::default(),
        clean: false,
        write: false,
        verbose: true,
        max_depth: u8::MAX,
        hide_extension: false,
    }
}

fn create_test_page(
    filetype: FileType,
    config: &Configuration,
    subfolder: Option<&str>,
    page_name: &str,
    content: &str,
) {
    if filetype.is_file() {
        let mut path = config.root.join("sections");
        if let Some(subfolder) = subfolder {
            path = path.join(subfolder)
        }

        _ = fs::create_dir_all(path.clone());
        path = path.join(format!("{page_name}{}", filetype.file_suffix()));
        _ = fs::write(path, content);
    }
}
