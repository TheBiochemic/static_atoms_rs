use std::{collections::HashMap, path::PathBuf};

use crate::{
    Configuration,
    dist::{markdown::resolve_tokens_markdown, resolve_tokens_html},
};

pub static FILE_TYPES: [FileType; 2] = [FileType::FileHTML, FileType::FileMarkdown];

/**
 * All the filetype related stuff goes here. I felt, that the
 * filetype might grow overtime, so i didn't want to have it in the bin.rs
 */
#[derive(Clone)]
pub enum FileType {
    Directory,
    FileHTML,
    FileMarkdown,
}

impl FileType {
    pub fn extension(&self) -> &str {
        match self {
            FileType::Directory => panic!("You should never call this function for a directory!"),
            FileType::FileHTML => "html",
            FileType::FileMarkdown => "md",
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            FileType::Directory => false,
            FileType::FileHTML | FileType::FileMarkdown => true,
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
        }
    }
}
