use std::path::PathBuf;

/**
 * All the filetype related stuff goes here. I felt, that the
 * filetype might grow overtime, so i didn't want to have it in the bin.rs
 */

pub enum FileType {
    Directory,
    FileHTML,
    FileMarkdown,
}

impl FileType {
    pub fn file_suffix(&self) -> &str {
        match self {
            FileType::Directory => panic!("You should never call this function for a directory!"),
            FileType::FileHTML => ".html",
            FileType::FileMarkdown => ".md",
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            FileType::Directory => false,
            FileType::FileHTML | FileType::FileMarkdown => true,
        }
    }
}

impl From<&PathBuf> for FileType {
    fn from(value: &PathBuf) -> Self {
        todo!()
    }
}
