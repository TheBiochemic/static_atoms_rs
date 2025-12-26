use std::{
    collections::HashMap,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    usize,
};

pub mod markdown;

use crate::{Configuration, filetype::FileType};

fn get_dist_path(config: &Configuration) -> PathBuf {
    match &config.out {
        Some(dist) => dist.clone(),
        None => config.root.clone().join("dist"),
    }
}

fn copy_dir(
    config: &Configuration,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
) -> io::Result<()> {
    fs::create_dir_all(&to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(config, entry.path(), to.as_ref().join(entry.file_name()))?;
        } else {
            let to_filename = to.as_ref().join(entry.file_name());
            if config.verbose {
                println!(
                    "[verbose] copy file {} to {}",
                    entry.file_name().to_string_lossy(),
                    to_filename.to_string_lossy()
                )
            }
            fs::copy(entry.path(), to_filename)?;
        }
    }
    Ok(())
}

pub fn get_pages(config: &Configuration) -> Vec<PathBuf> {
    fn read_folder_layer(path: PathBuf, pages_vec: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(&path).unwrap_or_else(|_| panic!("Wasn't able to completely go through the input directory. Does it exist and are you allowed to open it? Path: {}", path.to_string_lossy())).flatten() {
                if let Ok(filetype) = entry.file_type() {
                        if filetype.is_file() {
                            pages_vec.push(entry.path());
                        }

                        if filetype.is_dir() {
                            read_folder_layer(entry.path(), pages_vec);
                        }
                    }
            }
    }

    let mut pages_vec: Vec<PathBuf> = Default::default();

    if !config.input_files.is_empty() {
        for input_file in &config.input_files {
            if input_file.is_dir() {
                read_folder_layer(input_file.clone(), &mut pages_vec);
            }
            if input_file.is_file() {
                pages_vec.push(input_file.clone());
            }
        }
    } else {
        pages_vec.push(config.root.clone().join("index.html"));
        let pages_path = config.root.clone().join("pages");
        read_folder_layer(pages_path, &mut pages_vec);
    }

    pages_vec.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    pages_vec
}

pub fn run_dist(config: &Configuration) {
    let media_path = config.root.clone().join("media");
    let root_path = config.root.clone().join("root");
    let dist_path = get_dist_path(config);

    if config.clean {
        println!("Clearing old dist directory...");
        fs::remove_dir_all(&dist_path)
            .expect("Wasn't able to remove the pages directory. Does it exist?");
    }

    fs::create_dir_all(&dist_path)
        .expect("Wasn't able to recreate the pages directory. Do you have the permissions?");

    // Copy the root files over
    println!("Copy project root files...");
    if copy_dir(config, root_path, &dist_path).is_err() {
        println!(
            "Something went wrong, when copying root files over to {}",
            dist_path.to_string_lossy()
        );
    }

    // Copy all the media over
    println!("Copy all Media...");
    if copy_dir(config, media_path, dist_path.join("media")).is_err() {
        println!(
            "Something went wrong, when copying the media over to {}",
            dist_path.join("media").to_string_lossy()
        );
    }

    // Create default context
    println!("Building global Context");
    let pages = get_pages(config);
    let default_context = build_default_context(config, &pages);

    // Go through the pages directory
    for page in pages {
        if FileType::has_valid_extension(&page) {
            process_page(config, page, &default_context);
        }
    }

    process_page(
        config,
        config.root.clone().join("index.html"),
        &default_context,
    );
}

pub fn build_pages_context(config: &Configuration, input_pages: &[PathBuf]) -> String {
    let mut pages_string = String::default();
    pages_string.push_str("<ul class=\"siteindex\">");

    for page in input_pages {
        let mut relative_path = page
            .strip_prefix(&config.root)
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|err| {
                println!("Wasn't able to resolve path when building pages context. {err}");
                page.to_path_buf()
            });

        let mut filestem: String = relative_path
            .file_stem()
            .map(|os_str| os_str.to_str().unwrap_or(""))
            .unwrap_or("")
            .into();
        relative_path.pop();

        if filestem == "index" || !config.hide_extension {
            filestem += ".html";
        }

        let relative_path_href = if filestem != "index.html" {
            relative_path.push(filestem);
            relative_path.to_string_lossy().to_string()
        } else {
            let path_string = relative_path.to_string_lossy().to_string();
            relative_path.push(filestem);
            path_string
        };

        let relative_path_label = relative_path.to_string_lossy().to_string();
        pages_string.push_str(&format!(
            "<li><a href=\"/{relative_path_href}\">{relative_path_label}</a></li>"
        ));
    }

    pages_string.push_str("</ul>");
    pages_string
}

pub fn build_default_context(
    config: &Configuration,
    input_pages: &[PathBuf],
) -> HashMap<String, String> {
    let pages = build_pages_context(config, input_pages);

    HashMap::from([
        (
            "_VERSION".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        ),
        ("_APPNAME".to_string(), env!("CARGO_PKG_NAME").to_string()),
        (
            "_APPLINK".to_string(),
            format!(
                "<a href=\"{}\">{}</a>",
                env!("CARGO_PKG_HOMEPAGE"),
                env!("CARGO_PKG_NAME")
            )
            .to_string(),
        ),
        ("_PAGES".to_string(), pages),
    ])
}

pub fn resolve_tokens_from_path(
    path_string: String,
    path: &PathBuf,
    config: &Configuration,
    depth: u8,
    context: &HashMap<String, String>,
) -> Option<String> {
    let relative_path = path
        .strip_prefix(config.root.clone())
        .unwrap_or(path.as_path());

    for filetype in crate::filetype::FileType::get_valid_filetypes() {
        let path_with_extension =
            if path.extension().and_then(OsStr::to_str) != Some(filetype.extension().into()) {
                let mut new_path = path.clone();
                new_path.add_extension(filetype.extension());
                new_path
            } else {
                path.clone()
            };
        if let Ok(contents) = fs::read_to_string(path_with_extension) {
            return Some(filetype.convert_content(
                relative_path.to_string_lossy().to_string() + " >> " + &path_string,
                &contents,
                config,
                depth + 1,
                context,
            ));
        }
    }

    None
}

pub fn resolve_tokens_html(
    path: String,
    config: &Configuration,
    contents: &str,
    depth: u8,
    context: &HashMap<String, String>,
) -> String {
    resolve_embeds(path, config, contents, depth, context, ("<##", '>'))
}

pub fn resolve_embeds(
    path: String,
    config: &Configuration,
    contents_str: &str,
    depth: u8,
    context: &HashMap<String, String>,
    embed_symbols: (&str, char),
) -> String {
    let mut contents = contents_str.to_string();
    let mut content_len = usize::MAX;
    let mut content_len_new = usize::MAX - 1;
    let mut last_token_index = 0;
    while let Some(index) = contents.find(embed_symbols.0) {
        if content_len == content_len_new && last_token_index == index {
            println!(
                "Cannot resolve this token properly, aborting. (Are the symbols <>'\"()[] used properly?)"
            );
            break;
        }

        if config.verbose {
            println!(
                "[verbose] {path}: found next '{}' at index {index}",
                embed_symbols.0
            )
        };
        if let Some(index_end) = find_same_level(None, &contents[index..], embed_symbols.1, false) {
            if config.verbose {
                println!(
                    "[verbose] {path}: found matching '{}' after {index_end} char(s)",
                    embed_symbols.1
                )
            };
            let new_content = if depth < config.max_depth {
                let internal_contents = &contents[index..(index + index_end + 1)];
                if config.verbose {
                    let internal_contents_short = if internal_contents.len() > 50 {
                        let end_pos = internal_contents.char_indices().nth_back(7).unwrap().0;
                        String::from(&internal_contents[..35])
                            + ".."
                            + &internal_contents[end_pos..]
                    } else {
                        internal_contents.to_string()
                    };
                    println!(
                        "[verbose] {path}: will parse the contents \"{internal_contents_short}\"",
                    )
                };
                parse_token(path.clone(), config, internal_contents, depth, context)
            } else {
                println!(
                    "Surpassed max recursion depth of {}. Replacing deeper embeds with space",
                    config.max_depth
                );
                String::default()
            };
            contents.replace_range(index..(index + index_end + 1), &new_content);
        }

        last_token_index = index;
        content_len = content_len_new;
        content_len_new = contents.len();
    }

    contents
}

pub fn write_contents(config: &Configuration, page: PathBuf, contents: String) {
    if config.write {
        let relative_path = page
            .strip_prefix(config.root.clone())
            .expect("Wasnt able to get relative Path");
        let dist_path = get_dist_path(config);
        let mut absolute_path = dist_path.join(relative_path);

        let mut file_stem = absolute_path.file_stem().unwrap_or_default().to_os_string();
        absolute_path.pop();

        // If the file is an index file, or the config is set to show extensions, add them
        if file_stem == "index" || !config.hide_extension {
            file_stem.push(".html");
        }

        absolute_path.push(file_stem);

        let mut ancestors = absolute_path.ancestors();
        ancestors.next();

        if let Some(ancestor) = ancestors.next() {
            if fs::create_dir_all(ancestor).is_err() {
                println!(
                    "Wasnt able to create directory `{}`",
                    ancestor.to_string_lossy()
                );
            };
            if fs::write(&absolute_path, contents).is_err() {
                println!(
                    "Wasnt able to write file to path `{}`, ignoring",
                    absolute_path.to_string_lossy()
                );
            };
        } else {
            println!(
                "There was an error writing page to path `{}`",
                absolute_path.to_string_lossy()
            );
        }
    } else {
        println!("{contents}");
    }
}

pub fn process_page(
    config: &Configuration,
    page: PathBuf,
    default_context: &HashMap<String, String>,
) {
    let relative_path = page
        .strip_prefix(config.root.clone())
        .unwrap_or(page.as_path());
    let path_string = relative_path.to_string_lossy();
    println!("Transforming {path_string} ...");
    let contents = resolve_tokens_from_path(path_string.into(), &page, config, 0, default_context)
        .unwrap_or_else(|| {
            panic!(
                "Wasn't able to build page, since no page content could be generated for {}",
                page.to_string_lossy()
            );
        });
    write_contents(config, page, contents)
}

pub fn find_same_level(
    start_with: Option<char>,
    input: &str,
    test_char: char,
    test_first: bool,
) -> Option<usize> {
    let mut layers: Vec<char> = Vec::new();

    if let Some(start_with) = start_with {
        layers.push(start_with);
    }

    let level_test = |character, layers: &mut Vec<char>| match character {
        a @ '(' | a @ '<' | a @ '[' => layers.push(a),
        a @ '"' | a @ '\'' => {
            if layers.last().is_some() && layers.last().unwrap().eq(&a) {
                layers.pop();
            } else {
                layers.push(a);
            }
        }
        ')' => {
            if layers.last().is_some() && layers.last().unwrap().eq(&'(') {
                layers.pop();
            }
        }
        ']' => {
            if layers.last().is_some() && layers.last().unwrap().eq(&'[') {
                layers.pop();
            }
        }
        '>' => {
            if layers.last().is_some() && layers.last().unwrap().eq(&'<') {
                layers.pop();
            }
        }
        _ => (),
    };

    if test_first {
        for (index, character) in input.chars().enumerate() {
            if character.eq(&test_char) && layers.is_empty() {
                return Some(index);
            };
            level_test(character, &mut layers);
        }
    } else {
        for (index, character) in input.chars().enumerate() {
            level_test(character, &mut layers);
            if character.eq(&test_char) && layers.is_empty() {
                return Some(index);
            };
        }
    }

    None
}

pub fn parse_token(
    path: String,
    config: &Configuration,
    token: &str,
    current_depth: u8,
    context: &HashMap<String, String>,
) -> String {
    let embed_identifier = token[3..token.len() - 1].trim();

    // Check if it might be a folder embed
    match (embed_identifier.find("["), embed_identifier.find("]")) {
        (Some(open_index), Some(close_index)) => {
            return parse_folder_embed(
                path,
                config,
                embed_identifier,
                current_depth,
                (open_index, close_index),
                context,
            );
        }
        (Some(_), None) | (None, Some(_)) => {
            println!("folder identifier `{embed_identifier}` is incomplete");
        }
        _ => (),
    };

    // Check if it is a parametric embed
    match (
        embed_identifier.find("("),
        find_same_level(None, embed_identifier, ')', false),
    ) {
        (Some(open_index), Some(close_index)) => {
            return parse_parametric_embed(
                path,
                config,
                embed_identifier,
                current_depth,
                (open_index, close_index),
                context,
            );
        }
        (Some(_), None) | (None, Some(_)) => {
            println!("parametric identifier `{embed_identifier}` is incomplete");
        }
        _ => (),
    }

    // Check if it is a variable
    match (embed_identifier.find("{"), embed_identifier.find("}")) {
        (Some(0), Some(close_index)) => {
            return parse_variable(&embed_identifier[1..close_index], context);
        }
        (Some(_), Some(_)) | (Some(_), None) | (None, Some(_)) => {
            println!("variable identifier `{embed_identifier}` is incomplete or malformed");
        }
        _ => (),
    }

    // If none of them worked, it's most likely a simple embed
    parse_single_embed(path, config, embed_identifier, current_depth, context)
}

pub fn parse_parametric_embed(
    path: String,
    config: &Configuration,
    component: &str,
    current_depth: u8,
    brackets: (usize, usize),
    context: &HashMap<String, String>,
) -> String {
    if brackets.1 - brackets.0 == 1 {
        parse_single_embed(
            path,
            config,
            component[..brackets.0].trim(),
            current_depth,
            context,
        )
    } else {
        //Parse the contents between the brackets first
        let mut local_context = context.clone();
        let mut variables_string = component[(brackets.0 + 1)..brackets.1].trim();

        while let Some(next_equals) = variables_string.find('=') {
            let variable_name = variables_string[..next_equals].trim();

            let next_double_quotes = variables_string.find('"');
            let next_single_quotes = variables_string.find('\'');
            let (find_symbol, next_string_open) = match (next_double_quotes, next_single_quotes) {
                (None, Some(index)) => ('\'', index),
                (Some(index), None) => ('"', index),
                (Some(double_index), Some(single_index)) => {
                    if double_index < single_index {
                        ('"', double_index)
                    } else {
                        ('\'', single_index)
                    }
                }
                (None, None) => break,
            };

            let next_string_close =
                match variables_string[(next_string_open + 1)..].find(find_symbol) {
                    Some(string_end_index) => (next_string_open + 1) + string_end_index,
                    None => break,
                };

            let value = &variables_string[(next_string_open + 1)..next_string_close];

            if config.verbose {
                println!(
                    "[verbose] {path}: adding value for \'{}\' context. Content length: {}",
                    variable_name,
                    value.len()
                )
            }
            local_context.insert(variable_name.to_string(), value.to_string());

            variables_string = &variables_string[(next_string_close + 1)..];
        }

        if !variables_string.is_empty() {
            println!(
                "component `{component}` couldn't be parsed completely or at all. Is it malformed?"
            );
        }

        parse_single_embed(
            path,
            config,
            component[..brackets.0].trim(),
            current_depth,
            &local_context,
        )
    }
}

pub fn parse_variable(component: &str, context: &HashMap<String, String>) -> String {
    match context.get(component).cloned() {
        Some(variable) => variable,
        None => {
            println!("The variable `{component}` is undefined, replacing with empty space.");
            Default::default()
        }
    }
}

pub fn parse_folder_embed(
    path: String,
    config: &Configuration,
    component: &str,
    current_depth: u8,
    brackets: (usize, usize),
    context: &HashMap<String, String>,
) -> String {
    // Determine the item amount
    let mut elem_count = if brackets.1 - brackets.0 == 1 {
        usize::MAX
    } else if let Some(num_string) = &component[(brackets.0 + 1)..brackets.1]
        .to_string()
        .strip_prefix("..")
    {
        if let Ok(num) = num_string.parse::<usize>() {
            num
        } else {
            println!("The identifier `{component}` does not contain a valid number");
            usize::MAX
        }
    } else {
        println!(
            "The identifier `{component}` is not in the correct format. Use `identifier[..num]`"
        );
        usize::MAX
    };

    // Collect the files that are being chained
    let folder_embed_path = config.root.join("sections").join(&component[..brackets.0]);
    match fs::read_dir(&folder_embed_path) {
        Ok(dirs) => {
            let mut collected_dirs: Vec<_> = dirs
                .filter_map(|dir| match dir {
                    Ok(found_dir) => {
                        let dir_path = found_dir.path();
                        if dir_path.is_file() && FileType::has_valid_extension(&dir_path) {
                            Some(found_dir)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                })
                .collect();
            collected_dirs.sort_by_key(|a| a.file_name());

            elem_count = elem_count.min(collected_dirs.len());

            let mut content = String::default();
            for section in &collected_dirs[0..elem_count] {
                let resolved_content = if let Some(resolved_content) = resolve_tokens_from_path(
                    path.clone(),
                    &section.path(),
                    config,
                    current_depth,
                    context,
                ) {
                    resolved_content
                } else {
                    println!(
                        "Wasn't able to find embed at path \'{}\', returning empty component.",
                        section.path().to_string_lossy()
                    );
                    String::default()
                };
                content.push_str(&resolved_content);
            }
            content
        }
        Err(_) => {
            println!(
                "folder identifier `{}` is unknown, replacing with empty",
                folder_embed_path.to_string_lossy()
            );
            Default::default()
        }
    }
}

pub fn parse_single_embed(
    path: String,
    config: &Configuration,
    component: &str,
    current_depth: u8,
    context: &HashMap<String, String>,
) -> String {
    let component_path = config.root.clone().join("sections").join(component);

    if let Some(converted_content) = resolve_tokens_from_path(
        path.clone(),
        &component_path,
        config,
        current_depth,
        context,
    ) {
        converted_content
    } else {
        println!("Wasn't able to read the file contents of `{path}`. returning empty component",);
        String::default()
    }
}
