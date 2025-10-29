use std::{collections::HashMap, ffi::{OsStr, OsString}, fmt::Debug, fs, io, path::{Path, PathBuf}, time::SystemTime};

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

    // Create default context
    println!("Building global Context");
    let pages = {
        let mut pages_string = String::default();
        pages_string.push_str("<ul class=\"siteindex\">");
            if fs::exists(config.root.clone().join("index.html")).unwrap_or(false) {
                pages_string.push_str("<li><a href=\"/index.html\">index.html</a></li>");
            }

            let mut dirs: Vec<fs::DirEntry> = 
                fs::read_dir(config.root.clone().join("pages"))
                    .map(|elem| 
                        elem.into_iter()
                            .filter_map(|elem| elem.ok())
                            .collect::<Vec<_>>()
                    ).unwrap_or_default();

            dirs.sort_by_key(|elem| elem.file_name());

            if !dirs.is_empty() {
                pages_string.push_str("<li>");
                if fs::exists(config.root.clone().join("index.html")).unwrap_or(false) {
                    pages_string.push_str("<a href=\"/pages/index.html\">pages/index.html</a>");
                }
                pages_string.push_str("<ul class=\"siteindex\">");

                for dir in dirs.iter().filter(|elem| (elem.file_name() != "index.html")) {
                    if dir.path().is_file() && dir.path().extension().and_then(OsStr::to_str) == Some("html") {
                        let filename_string = dir.file_name().into_string().unwrap_or_default();
                        pages_string.push_str(&format!("<li><a href=\"/pages/{filename_string}\">{filename_string}</a></li>"));
                    }
                }

                pages_string.push_str("</ul></li>");
            }

        pages_string.push_str("</ul>");
        pages_string
    };
    let default_context = HashMap::from([
        ("_VERSION".to_string(), env!("CARGO_PKG_VERSION").to_string()),
        ("_APPNAME".to_string(), env!("CARGO_PKG_NAME").to_string()),
        ("_APPLINK".to_string(), format!("<a href=\"{}\">{}</a>", env!("CARGO_PKG_HOMEPAGE"), env!("CARGO_PKG_NAME")).to_string()),
        ("_PAGES".to_string(), pages)
    ]);

    // Go through the pages directory
    for page in fs::read_dir(pages_path).expect("Wasn't able to go through the pages directory. Does it exist and are you allowed to open it?") {
        match page {
            Ok(found_page) => {
                if 
                    found_page.path().is_file() && 
                    found_page.path().extension().and_then(OsStr::to_str) == Some("html") {
                    process_page(config, found_page.path(), &default_context);
                };
            },
            Err(_) => println!("Couldn't open pages path, ignoring it"),
        }
    }

    process_page(config, config.root.clone().join("index.html"), &default_context);
}

pub fn read_section(path: &PathBuf) -> String {
    let file_error = format!("Wasn't able to read the file contents of `{}`", path.to_string_lossy());
    fs::read_to_string(path).expect(&file_error)
}

pub fn read_section_or_default(path: &PathBuf) -> String {
    match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => {
            println!("Wasn't able to read the file contents of `{}`. returning empty component", path.to_string_lossy());
            String::default()
        },
    }
}

pub fn resolve_tokens(config: &Configuration, mut contents: String, depth: u8, context: &HashMap<String, String>) -> String {
    while let Some(index) = contents.find("<##") {
        if let Some(index_end) = contents[index..].find(">") {
            let new_content = if depth < config.max_depth {
                    parse_token(config, &contents[index..(index + index_end + 1)], depth + 1, context)
                } else {
                    println!("Surpassed max recursion depth of {}. Replacing deeper embeds with space", config.max_depth);
                    String::default()
                };
            contents.replace_range(index..(index + index_end + 1), &new_content);
        }
    };

    contents
}

pub fn process_page(config: &Configuration, page: PathBuf, default_context: &HashMap<String, String>) {
    println!("Transforming {} ...", page.to_str().unwrap_or_default());
    
    let contents = resolve_tokens(config, read_section(&page), 0, default_context);

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

pub fn parse_token(config: &Configuration, token: &str, current_depth: u8, context: &HashMap<String, String>) -> String {
    let embed_identifier = token[3..token.len()-1].trim();

    // Check if it might be a folder embed
    match (embed_identifier.find("["), embed_identifier.find("]")) {
        (Some(open_index), Some(close_index)) => {
            return parse_folder_embed(config, embed_identifier, current_depth, (open_index, close_index), context)
        },
        (Some(_), None) | (None, Some(_)) => {
            println!("folder identifier `{embed_identifier}` is incomplete");
        },
        _ => ()
    };

    // Check if it is a parametric embed
    match (embed_identifier.find("("), embed_identifier.find(")")) {
        (Some(open_index), Some(close_index)) => {
            return parse_parametric_embed(config, embed_identifier, current_depth, (open_index, close_index), context)
        },
        (Some(_), None) | (None, Some(_)) => {
            println!("parametric identifier `{embed_identifier}` is incomplete");
        },
        _ => ()
    }

    // Check if it is a variable
    match (embed_identifier.find("{"), embed_identifier.find("}")) {
        (Some(0), Some(close_index)) => {
            return parse_variable(&embed_identifier[1..close_index], context)
        },
        (Some(_), Some(_)) | (Some(_), None) | (None, Some(_)) => {
            println!("variable identifier `{embed_identifier}` is incomplete or malformed");
        },
        _ => ()
    }

    // If none of them worked, it's most likely a simple embed
    parse_single_embed(config, embed_identifier, current_depth, context)
}

pub fn parse_parametric_embed(
    config: &Configuration, 
    component: &str, 
    current_depth: u8, 
    brackets: (usize, usize), 
    context: &HashMap<String, String>
) -> String {
    if brackets.1 - brackets.0 == 1 {
        parse_single_embed(config, component[..brackets.0].trim(), current_depth, context)
    } else {
        //Parse the contents between the brackets first
        let mut local_context = context.clone();
        let mut variables_string = component[(brackets.0 + 1)..brackets.1].trim();

        while let Some(next_equals) = variables_string.find('=') {

            let variable_name = variables_string[..next_equals].trim();

            let next_string_open = match variables_string.find('"') {
                Some(string_start_index) => string_start_index,
                None => break,
            };

            let next_string_close = match variables_string[(next_string_open + 1)..].find('"') {
                Some(string_end_index) => (next_string_open + 1) + string_end_index,
                None => break,
            };

            let value = &variables_string[(next_string_open + 1)..next_string_close];

            local_context.insert(variable_name.to_string(), value.to_string());

            variables_string = &variables_string[(next_string_close + 1)..];
        }

        if !variables_string.is_empty() {
            println!("component `{component}` couldn't be parsed completely or at all. Is it malformed?");
        }

        parse_single_embed(config, component[..brackets.0].trim(), current_depth, &local_context)
    }
}

pub fn parse_variable(
    component: &str, 
    context: &HashMap<String, String>
) -> String {
    match context.get(component).cloned() {
        Some(variable) => variable,
        None => {
            println!("The variable `{component}` is undefined, replacing with empty space.");
            Default::default()
        },
    }
}

pub fn parse_folder_embed(config: &Configuration, component: &str, current_depth: u8, brackets: (usize, usize), context: &HashMap<String, String>) -> String {
    // Determine the item amount
    let mut elem_count = if brackets.1 - brackets.0 == 1 {
        usize::MAX
    } else if let Some(num_string) = &component[(brackets.0 + 1)..brackets.1].to_string().strip_prefix("..") {
        if let Ok(num) = num_string.parse::<usize>() {
            num
        } else {
            println!("The identifier `{component}` does not contain a valid number");
            usize::MAX
        }
    } else {
        println!("The identifier `{component}` is not in the correct format. Use `identifier[..num]`");
        usize::MAX
    };

    // Collect the files that are being chained
    let folder_embed_path = config.root.join("sections").join(&component[..brackets.0]);
    match fs::read_dir(&folder_embed_path) {
        Ok(dirs) => {
            let mut collected_dirs: Vec<_> = dirs.filter_map(|dir| {
                match dir {
                    Ok(found_dir) => {
                        if found_dir.path().is_file() && 
                        found_dir.path().extension().and_then(OsStr::to_str) == Some("html") {
                            Some(found_dir)
                        } else {
                            None
                        } 
                    },
                    Err(_) => None,
                }
            }).collect();
            collected_dirs.sort_by_key(|a| a.file_name());

            elem_count = elem_count.min(collected_dirs.len());

            let mut content = String::default();
            for section in &collected_dirs[0..elem_count] {
                content.push_str(&resolve_tokens(config, read_section(&section.path()), current_depth, context));
            }
            content
        },
        Err(_) => {
            println!("folder identifier `{}` is unknown, replacing with empty", folder_embed_path.to_string_lossy());
            Default::default()
        },
    }
}

pub fn parse_single_embed(config: &Configuration, component: &str, current_depth: u8, context: &HashMap<String, String>) -> String {
    let mut embed_file_path = component.to_owned();
        embed_file_path.push_str(".html");
        let component_path = config.root.clone().join("sections").join(embed_file_path);
        resolve_tokens(config, read_section_or_default(&component_path), current_depth, context)
}
