use std::{env, fs};

use crate::{Configuration, filetype::FileType};

#[cfg(test)]
mod tests_markdown;

#[cfg(test)]
mod tests_html;

pub fn get_config() -> Configuration {
    get_config_internal("static_atoms_rs_tests", false)
}

pub fn get_config_multi() -> Configuration {
    get_config_internal("static_atoms_rs_tests_multi", true)
}

fn get_config_internal(proj_dir: &str, write: bool) -> Configuration {
    Configuration {
        root: env::temp_dir().join(proj_dir),
        out: None,
        input_files: Vec::default(),
        clean: false,
        write,
        verbose: true,
        max_depth: u8::MAX,
        hide_extension: false,
    }
}

fn create_embed_objects(
    filetype: FileType,
    config: &Configuration,
    subfolders: Vec<&str>,
    page_name: &str,
    content: &str,
    top_level_folder: Option<&str>,
) {
    if filetype.is_file() {
        let mut path = if let Some(top_level_folder) = top_level_folder {
            config.root.join(top_level_folder)
        } else {
            config.root.clone()
        };
        for subfolder in subfolders {
            path = path.join(subfolder)
        }

        _ = fs::create_dir_all(path.clone());
        path = path.join(format!("{page_name}{}", filetype.file_suffix()));
        _ = fs::write(path, content);

        // Sleep for slow filesystems
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn create_test_section(
    filetype: FileType,
    config: &Configuration,
    subfolders: Vec<&str>,
    page_name: &str,
    content: &str,
) {
    create_embed_objects(
        filetype,
        config,
        subfolders,
        page_name,
        content,
        Some("sections"),
    )
}

fn create_test_page(
    filetype: FileType,
    config: &Configuration,
    subfolders: Vec<&str>,
    page_name: &str,
    content: &str,
) {
    create_embed_objects(
        filetype,
        config,
        subfolders,
        page_name,
        content,
        Some("pages"),
    )
}

fn create_index_page(filetype: FileType, config: &Configuration, content: &str) {
    create_embed_objects(filetype, config, vec![], "index", content, None)
}
