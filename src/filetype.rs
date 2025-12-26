use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

use crate::{
    Configuration,
    dist::{markdown::resolve_tokens_markdown, resolve_tokens_html},
};

static FILE_TYPES: [FileType; 3] = [
    FileType::FileHTML,
    FileType::FileMarkdown,
    FileType::FileText,
];

/**
 * All the filetype related stuff goes here. I felt, that the
 * filetype might grow overtime, so i didn't want to have it in the bin.rs
 */
#[derive(Clone)]
pub enum FileType {
    Directory,
    FileHTML,
    FileMarkdown,
    FileText,
}

impl FileType {
    pub fn extension(&self) -> &str {
        match self {
            FileType::Directory => panic!("You should never call this function for a directory!"),
            FileType::FileHTML => "html",
            FileType::FileMarkdown => "md",
            FileType::FileText => "txt",
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            FileType::Directory => false,
            _ => true,
        }
    }

    pub fn has_valid_extension(path: &PathBuf) -> bool {
        match path.extension().and_then(OsStr::to_str) {
            Some(file_ext) => {
                for filetype in &FILE_TYPES {
                    if filetype.extension() == file_ext {
                        return true;
                    }
                }
                false
            }
            None => false,
        }
    }

    pub fn get_valid_filetypes() -> &'static [FileType] {
        &FILE_TYPES
    }

    pub fn convert_content(
        &self,
        path_string: String,
        content: &str,
        config: &Configuration,
        depth: u8,
        context: &HashMap<String, String>,
    ) -> String {
        match self {
            FileType::Directory => {
                panic!("Converting the content from a folder format does not make any sense!")
            }
            FileType::FileHTML => resolve_tokens_html(path_string, config, content, depth, context),
            FileType::FileMarkdown => resolve_tokens_markdown(
                path_string,
                config,
                content,
                depth,
                context,
                ("<p>", "</p>"),
                false,
            ),
            FileType::FileText => content.to_string(),
        }
    }
}
