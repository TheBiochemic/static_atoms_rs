use std::{ffi::OsStr, fs, io, path::{Path, PathBuf}};

use crate::Configuration;

fn get_dist_path(config: &Configuration) -> PathBuf {
    match &config.out {
        Some(dist) => dist.clone(),
        None => config.root.clone().join("dist"),
    }
}

fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(entry.path(), to.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), to.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn run_dist(config: &Configuration) {
    let pages_path = config.root.clone().join("pages");
    let media_path = config.root.clone().join("media");
    let stylesheet_path = config.root.clone().join("style.css");
    let dist_path = get_dist_path(config);

    if config.clean {
        println!("Clearing old dist directory...");
        fs::remove_dir_all(&dist_path)
            .expect("Wasn't able to remove the pages directory. Does it exist?");
    }

    fs::create_dir_all(&dist_path)
            .expect("Wasn't able to recreate the pages directory. Do you have the permissions?");

    // Copy the stylesheet over
    println!("Copy Stylesheet...");
    if fs::copy(stylesheet_path, dist_path.join("style.css")).is_err() {
        println!("Wasnt able to copy {}", dist_path.join("style.css").to_string_lossy())
    };

    // Copy all the media over
    println!("Copy all Media...");
    if copy_dir(media_path, dist_path.join("media")).is_err() {
        println!("Something went wrong, when copying the media over to {}", dist_path.join("media").to_string_lossy());
    }

    process_page(config, config.root.clone().join("index.html"));

    // Go through the pages directory
    for page in fs::read_dir(pages_path).expect("Wasn't able to go through the pages directory. Does it exist and are you allowed to open it?") {
        match page {
            Ok(found_page) => {
                if 
                    found_page.path().is_file() && 
                    found_page.path().extension().and_then(OsStr::to_str) == Some("html") {
                    process_page(config, found_page.path());
                };
            },
            Err(_) => println!("Couldn't open pages path, ignoring it"),
        }
    }
}

pub fn read_section(config: &Configuration, path: &PathBuf) -> String {
    let mut contents = fs::read_to_string(path).expect("Wasn't able to read the file contents");
    while let Some(index) = contents.find("<##") {
        if let Some(index_end) = contents[index..].find(">") {
            contents.replace_range(
                index..(index + index_end + 1), 
                &parse_token(config, &contents[index..(index + index_end + 1)])
            );
        }
    };

    contents
}

pub fn process_page(config: &Configuration, page: PathBuf) {
    println!("Transforming {} ...", page.to_str().unwrap_or_default());
    
    let contents = read_section(config, &page);

    if config.write {
        let relative_path = page.strip_prefix(config.root.clone()).expect("Wasnt able to get relative Path");
        let dist_path = get_dist_path(config);
        let absolute_path = dist_path.join(relative_path);
        let mut ancestors = absolute_path.ancestors();
        ancestors.next();
        
        if let Some(ancestor) = ancestors.next() {
            if fs::create_dir_all(ancestor).is_err() {
                println!("Wasnt able to create directory `{}`", ancestor.to_string_lossy());
            };
            if fs::write(&absolute_path, contents).is_err() {
                println!("Wasnt able to write file to path `{}`, ignoring", absolute_path.to_string_lossy());
            };
        } else {
            println!("There was an error writing page to path `{}`", absolute_path.to_string_lossy());
        }
        
    } else {
        println!("{contents}");
    }
}

pub fn parse_token(config: &Configuration, token: &str) -> String {
    let embed_identifier = token[3..token.len()-1].trim();

    if let Some(open_index) = embed_identifier.find("[") {
        // Is potentially a folder identifier
        if let Some(close_index) = embed_identifier.find("]") {

            // Determine the item amount
            let mut elem_count = if close_index - open_index == 1 {
                usize::MAX
            } else if let Some(num_string) = &embed_identifier[(open_index + 1)..close_index].to_string().strip_prefix("..") {
                if let Ok(num) = num_string.parse::<usize>() {
                    num
                } else {
                    println!("the identifier `{embed_identifier}` does not contain a valid number");
                    usize::MAX
                }
            } else {
                println!("the identifier `{embed_identifier}` is not in the correct format. Use `identifier[..num]`");
                usize::MAX
            };
           

            let folder_embed = config.root.join("sections").join(&embed_identifier[..open_index]);
            let dirs = fs::read_dir(get_dist_path(config).join(&folder_embed));
            match dirs {
                Ok(dirs) => {
                    let mut collected_dirs: Vec<_> = dirs.filter_map(|dir| {match dir {
                        Ok(found_dir) => {
                            if found_dir.path().is_file() && 
                            found_dir.path().extension().and_then(OsStr::to_str) == Some("html") {
                                Some(found_dir)
                            } else {
                                None
                            } 
                        },
                        Err(_) => None,
                    }}).collect();
                    collected_dirs.sort_by_key(|a| a.file_name());

                    elem_count = elem_count.min(collected_dirs.len());

                    let mut content = String::default();
                    for section in &collected_dirs[0..elem_count] {
                        content.push_str(&read_section(config, &section.path()));
                    }
                    return content;
                },
                Err(_) => println!("folder identifier `{}` is unknown, replacing with empty", folder_embed.to_string_lossy()),
            }
            
        } else {
            println!("folder identifier `{embed_identifier}` is incomplete, replacing with empty");
        }

        String::default()

    } else {
        parse_single_component(config, embed_identifier)
    }
}

pub fn parse_single_component(config: &Configuration, component: &str) -> String {
    let mut embed_file_path = component.to_owned();
        embed_file_path.push_str(".html");
        let component_path = config.root.clone().join("sections").join(embed_file_path);
        match fs::read_to_string(component_path) {
            Ok(contents) => contents,
            Err(_) => {
                println!("Wasn't able to read the file contents of {component}. returning empty component");
                String::default()
            },
        }
}
