use std::collections::HashMap;

use crate::{
    Configuration,
    dist::{find_same_level, resolve_embeds},
};

pub fn resolve_tokens_markdown(
    path: String,
    config: &Configuration,
    contents: &str,
    depth: u8,
    context: &HashMap<String, String>,
    custom_tag_type: (&str, &str),
    ignore_list_type: bool,
) -> String {
    let contents = resolve_embeds(path.clone(), config, contents, depth, context, ("[##", ']'));

    let mut converted = String::new();

    #[derive(PartialEq)]
    enum ListType {
        UnorderedPlus,
        UnorderedDash,
        UnorderedAsterisk,
        OrderedDot,
        OrderedBracket,
    }

    #[derive(PartialEq)]
    enum TopLevelBlock {
        Nothing,
        Paragraph(String),
        CodeBlockSpace(String),
        CodeBlockFence(usize, String),
        BlockQuote(String),
        List {
            indent: usize,
            list_type: ListType,
            use_paragraph: bool,
            list_items: Vec<String>,
        },
    }

    let mut top_level_block = TopLevelBlock::Nothing;
    let mut prev_line_was_empty = false;

    fn finish_blocks(
        path: String,
        config: &Configuration,
        depth: u8,
        context: &HashMap<String, String>,
        converted: &mut String,
        block: &mut TopLevelBlock,
        custom_tag_type: &(&str, &str),
    ) {
        match block {
            TopLevelBlock::Nothing => (),
            TopLevelBlock::Paragraph(content) => {
                let paragraph = resolve_markdown_paragraph(content);
                converted.push_str(&paragraph);
                converted.push_str(custom_tag_type.1)
            }
            TopLevelBlock::CodeBlockSpace(content) | TopLevelBlock::CodeBlockFence(_, content) => {
                converted.push_str(content);
                converted.push_str("</code></pre>")
            }
            TopLevelBlock::BlockQuote(content) => {
                let resolved = resolve_tokens_markdown(
                    path,
                    config,
                    content,
                    depth,
                    context,
                    ("<p>", "</p>"),
                    false,
                );

                converted.push_str(&resolved);
                converted.push_str("</blockquote>")
            }
            TopLevelBlock::List {
                indent: _,
                list_type,
                use_paragraph,
                list_items,
            } => {
                for list_item in list_items {
                    println!("LIST_ITEM BEFORE RESOLVE: {list_item}");
                    let resolved = resolve_tokens_markdown(
                        path.clone(),
                        config,
                        list_item,
                        depth,
                        context,
                        if *use_paragraph {
                            ("<p>", "</p>")
                        } else {
                            ("", "")
                        },
                        true,
                    );
                    converted.push_str("<li>");
                    converted.push_str(&resolved);
                    converted.push_str("</li>");
                }

                converted.push_str(match &list_type {
                    ListType::OrderedBracket | ListType::OrderedDot => "</ol>",
                    ListType::UnorderedAsterisk
                    | ListType::UnorderedDash
                    | ListType::UnorderedPlus => "</ul>",
                });
            }
        }
        *block = TopLevelBlock::Nothing;
    }

    // resolve all embeds
    // TODO: Build a function for [## embed()], etc.

    for line in contents.lines() {
        let line_no_prefix = line.trim_start();
        let trimmed_line = line_no_prefix.trim_end();

        // If the line is a list type
        // Detect the type of list of a line and if it's a ordered list, also it's number
        type TrimOffset = usize;
        type ParsedNumber = usize;
        fn detect_list_type(
            line_no_prefix: &str,
            ignore_list_type: bool,
        ) -> Option<(ListType, ParsedNumber, TrimOffset)> {
            let list_type_prefix = line_no_prefix.get(0..=1);
            match list_type_prefix {
                Some("* ") => {
                    return if ignore_list_type {
                        Some((ListType::UnorderedAsterisk, 0, 2))
                    } else {
                        Some((ListType::UnorderedAsterisk, 0, 2))
                    };
                }
                Some("+ ") => {
                    return if ignore_list_type {
                        Some((ListType::UnorderedAsterisk, 0, 2))
                    } else {
                        Some((ListType::UnorderedPlus, 0, 2))
                    };
                }
                Some("- ") => {
                    return if ignore_list_type {
                        Some((ListType::UnorderedAsterisk, 0, 2))
                    } else {
                        Some((ListType::UnorderedDash, 0, 2))
                    };
                }
                _ => (),
            };

            let mut list_number_str = "".to_owned();
            let mut list_type = None;
            let mut space_after_list_symbol = false;
            for single_char in line_no_prefix.chars() {
                match single_char {
                    digit_char if digit_char.is_ascii_digit() => {
                        list_number_str.push(digit_char);
                    }
                    ')' => list_type = Some(ListType::OrderedBracket),
                    '.' => list_type = Some(ListType::OrderedDot),
                    space if space.is_ascii_whitespace() && list_type.is_some() => {
                        space_after_list_symbol = true;
                        break;
                    }
                    _ => break,
                }
            }

            let parsed: Result<usize, _> = list_number_str.parse();
            if let (Some(list_type), Ok(parsed_num)) = (list_type, parsed) {
                if space_after_list_symbol {
                    return Some((list_type, parsed_num, list_number_str.len() + 1));
                }
            }

            None
        }

        // If a previous line was already a list, continue the list
        if let TopLevelBlock::List {
            indent,
            list_type,
            use_paragraph,
            list_items,
        } = &mut top_level_block
        {
            // If the List has been initialized in a previous line, lets continue it
            // first detect some of the information, like what type of list it is
            let new_indent = line.len() - line_no_prefix.len();

            if trimmed_line.is_empty() {
                list_items.last_mut().unwrap().push('\n');
                prev_line_was_empty = true;
                continue;
            }

            // Check if that line is a list
            let detected_list_type = detect_list_type(line_no_prefix, ignore_list_type);
            if let Some((new_list_type, _, offset)) = detected_list_type {
                if prev_line_was_empty {
                    *use_paragraph = true;
                }

                if new_indent <= (*indent + 1) {
                    // if this line is part of the list, and in the same indent
                    println!("SAME INDENT -> INDENT: {indent}, NEW_INDENT: {new_indent}");

                    if new_list_type == *list_type {
                        *indent = new_indent;
                        let mut new_item = String::from(trimmed_line.get(offset..).unwrap_or(""));
                        new_item.push('\n');
                        list_items.push(new_item);
                        prev_line_was_empty = false;
                        continue;
                    } else {
                        finish_blocks(
                            path.clone(),
                            config,
                            depth,
                            context,
                            &mut converted,
                            &mut top_level_block,
                            &custom_tag_type,
                        );
                    }
                } else {
                    // if the line is part of the list, but a sub-list item
                    println!("SUB-LIST ITEM -> INDENT: {indent}, NEW_INDENT: {new_indent}");

                    let last_item = list_items.last_mut().unwrap();
                    last_item.push_str(line.get(*indent..).unwrap_or(""));
                    last_item.push('\n');
                    prev_line_was_empty = false;
                    continue;
                }
            } else {
                // if the detected line is not a list element
                if prev_line_was_empty {
                    finish_blocks(
                        path.clone(),
                        config,
                        depth,
                        context,
                        &mut converted,
                        &mut top_level_block,
                        &custom_tag_type,
                    );
                }
            }
        }

        // If the line is not a list, or not a list anymore (needs to be detected)
        if !matches!(
            top_level_block,
            TopLevelBlock::List {
                indent: _,
                list_type: _,
                use_paragraph: _,
                list_items: _
            }
        ) {
            let detected_list_type = detect_list_type(line_no_prefix, ignore_list_type);
            let indent = line.len() - line_no_prefix.len();
            if detected_list_type.is_some() {
                finish_blocks(
                    path.clone(),
                    config,
                    depth,
                    context,
                    &mut converted,
                    &mut top_level_block,
                    &custom_tag_type,
                );
            }

            match detected_list_type {
                Some((list_type @ ListType::UnorderedAsterisk, _, offset))
                | Some((list_type @ ListType::UnorderedDash, _, offset))
                | Some((list_type @ ListType::UnorderedPlus, _, offset)) => {
                    converted.push_str("<ul>");
                    let mut first_item = String::from(trimmed_line.get(offset..).unwrap_or(""));
                    first_item.push('\n');
                    top_level_block = TopLevelBlock::List {
                        indent,
                        list_type,
                        use_paragraph: false,
                        list_items: vec![first_item],
                    };
                    prev_line_was_empty = false;
                    continue;
                }
                Some((list_type @ ListType::OrderedBracket, list_number, offset))
                | Some((list_type @ ListType::OrderedDot, list_number, offset)) => {
                    converted.push_str("<ol");
                    if list_number != 1 {
                        let formatted_str = format!(" start=\"{list_number}\"");
                        converted.push_str(&formatted_str);
                    }
                    converted.push_str(">");

                    let indent = line.len() - line_no_prefix.len();

                    let mut first_item = String::from(trimmed_line.get(offset..).unwrap_or(""));

                    first_item.push('\n');

                    top_level_block = TopLevelBlock::List {
                        indent,
                        list_type,
                        use_paragraph: false,
                        list_items: vec![first_item],
                    };
                    prev_line_was_empty = false;
                    continue;
                }
                None => (),
            }
        }

        //If the line is the start of a space code block, create the code block
        if line.starts_with("    ")
            && !matches!(
                top_level_block,
                TopLevelBlock::CodeBlockFence(_, _)
                    | TopLevelBlock::List {
                        indent: _,
                        list_type: _,
                        use_paragraph: _,
                        list_items: _
                    }
            )
        {
            if let TopLevelBlock::CodeBlockSpace(content) = &mut top_level_block {
                let code_content = "\n".to_owned() + &line.to_owned().split_off(4);
                content.push_str(code_content.as_str());
                prev_line_was_empty = false;
                continue;
            } else {
                finish_blocks(
                    path.clone(),
                    config,
                    depth,
                    context,
                    &mut converted,
                    &mut top_level_block,
                    &custom_tag_type,
                );
                converted.push_str("<pre><code>");
                top_level_block = TopLevelBlock::CodeBlockSpace(line.to_owned().split_off(4));
                prev_line_was_empty = false;
                continue;
            }
        }

        // If the line starts with a fence code block
        if line_no_prefix.starts_with("```") {
            if let TopLevelBlock::CodeBlockFence(_, _) = top_level_block {
                if trimmed_line == "```" {
                    finish_blocks(
                        path.clone(),
                        config,
                        depth,
                        context,
                        &mut converted,
                        &mut top_level_block,
                        &custom_tag_type,
                    );
                    prev_line_was_empty = false;
                    continue;
                }
            } else {
                finish_blocks(
                    path.clone(),
                    config,
                    depth,
                    context,
                    &mut converted,
                    &mut top_level_block,
                    &custom_tag_type,
                );
                let code_suffix = line_no_prefix.to_owned().split_off(3).trim().to_owned();
                if code_suffix.is_empty() {
                    converted.push_str("<pre><code>");
                } else {
                    converted.push_str("<pre><code class=\"language-");
                    converted.push_str(&code_suffix);
                    converted.push_str("\">");
                }
                let indent = line.len() - line_no_prefix.len();
                top_level_block = TopLevelBlock::CodeBlockFence(indent, String::new());
                prev_line_was_empty = false;
                continue;
            }
        }

        if let TopLevelBlock::CodeBlockFence(indent, content) = &mut top_level_block {
            let calculated_indent = line.len() - line_no_prefix.len();
            let actual_offset = calculated_indent.saturating_sub(*indent);
            if !content.is_empty() {
                content.push('\n');
            }

            let offset_string = " ".repeat(actual_offset);
            content.push_str(offset_string.as_str());
            content.push_str(trimmed_line);
            prev_line_was_empty = false;
            continue;
        }

        //If the line is empty, ignore it
        if trimmed_line.is_empty() {
            finish_blocks(
                path.clone(),
                config,
                depth,
                context,
                &mut converted,
                &mut top_level_block,
                &custom_tag_type,
            );
            prev_line_was_empty = true;
            continue;
        }

        // If the line is a horizontal line
        let mut insert_hr = false;
        for (starts_with, test_char) in [("***", '*'), ("___", '_'), ("---", '-')] {
            if trimmed_line.starts_with(starts_with) {
                let mut all_same_symbol = true;
                for line_char in trimmed_line.chars() {
                    if line_char != test_char {
                        all_same_symbol = false;
                        break;
                    }
                }

                if all_same_symbol {
                    insert_hr = true;
                    break;
                }
            }
        }

        if insert_hr {
            finish_blocks(
                path.clone(),
                config,
                depth,
                context,
                &mut converted,
                &mut top_level_block,
                &custom_tag_type,
            );
            converted.push_str("</hr>");
            prev_line_was_empty = false;
            continue;
        }

        // If the line is a heading
        if trimmed_line.starts_with("#") {
            let mut header_type = 0usize;
            for character in trimmed_line.chars() {
                if character == '#' {
                    header_type += 1;
                };
            }

            if header_type > 0 && header_type < 6 {
                finish_blocks(
                    path.clone(),
                    config,
                    depth,
                    context,
                    &mut converted,
                    &mut top_level_block,
                    &custom_tag_type,
                );
                let paragraph =
                    resolve_markdown_paragraph(trimmed_line[(header_type + 1)..].trim());
                converted
                    .push_str(format!("<h{header_type}>{paragraph}</h{header_type}>").as_str());
                prev_line_was_empty = false;
                continue;
            }
        }

        // If the line is a block quote
        if trimmed_line.starts_with(">") {
            if let TopLevelBlock::BlockQuote(content) = &mut top_level_block {
                content.push_str(trimmed_line.strip_prefix(">").unwrap_or(""));
                content.push('\n');
                prev_line_was_empty = false;
                continue;
            } else {
                finish_blocks(
                    path.clone(),
                    config,
                    depth,
                    context,
                    &mut converted,
                    &mut top_level_block,
                    &custom_tag_type,
                );

                converted.push_str("<blockquote>");

                let mut content = String::new();
                content.push_str(trimmed_line.strip_prefix(">").unwrap_or(""));
                content.push('\n');

                top_level_block = TopLevelBlock::BlockQuote(content);
                prev_line_was_empty = false;
                continue;
            }
        }

        // If the line is just a simple paragraph
        if let TopLevelBlock::Paragraph(content) = &mut top_level_block {
            content.push(' ');
            content.push_str(trimmed_line);
            prev_line_was_empty = false;
        } else {
            if let TopLevelBlock::List {
                indent: _,
                list_type: _,
                use_paragraph: _,
                list_items,
            } = &mut top_level_block
            {
                let last_elem = list_items.last_mut().unwrap();
                last_elem.push(' ');
                last_elem.push_str(trimmed_line);
                prev_line_was_empty = false;
            } else {
                finish_blocks(
                    path.clone(),
                    config,
                    depth,
                    context,
                    &mut converted,
                    &mut top_level_block,
                    &custom_tag_type,
                );

                converted.push_str(custom_tag_type.0);
                top_level_block = TopLevelBlock::Paragraph(trimmed_line.to_string());
                prev_line_was_empty = false;
            }
        }
    }

    finish_blocks(
        path,
        config,
        depth,
        context,
        &mut converted,
        &mut top_level_block,
        &custom_tag_type,
    );
    converted
}

fn resolve_markdown_paragraph(paragraph: &str) -> String {
    // Relevant data
    let mut output_text = String::from(paragraph);

    // replacements array; Format (position, new_string, old_length)
    let mut replacements: Vec<(usize, String, usize)> = Default::default();

    // Build the vec for code snippet sections
    let code_sections: Vec<_> = {
        let all_code_snippets: Vec<_> = output_text.match_indices("`").collect();
        let mut all_code_snippets_deduped = all_code_snippets.clone();
        all_code_snippets_deduped.retain(|elem| {
            let self_index = elem.0;
            let neighbours = all_code_snippets
                .iter()
                .find(|elem| elem.0 + 1 == self_index || elem.0 - 1 == self_index);
            neighbours.is_none()
        });

        all_code_snippets_deduped
            .chunks_exact(2)
            .map(|elem| elem[0].0..=elem[1].0)
            .collect()
    };

    for code_section in &code_sections {
        replacements.push((*code_section.start(), "<code>".into(), 1));
        replacements.push((*code_section.end(), "</code>".into(), 1));
    }

    // get and filter all images and links
    let all_image_links: Vec<_> = output_text
        .match_indices("[")
        .filter_map(|elem| {
            for section in &code_sections {
                if section.contains(&elem.0) {
                    return None;
                }
            }

            let mut tag = "a";

            let prev_char = elem.0.checked_sub(1);
            if let Some(prev_index) = prev_char {
                if output_text.chars().nth(prev_index).unwrap_or('.') == '!' {
                    tag = "img";
                }
            }

            let found_close_bracket = find_same_level(None, &output_text[elem.0..], ']', false)?;
            if output_text.chars().nth(elem.0 + found_close_bracket + 1)? != '(' {
                return None;
            }
            let link_close_bracket = find_same_level(
                None,
                &output_text[(elem.0 + found_close_bracket + 2)..],
                ')',
                false,
            )?;

            let link_content = output_text[(elem.0 + found_close_bracket + 2)
                ..=(elem.0 + found_close_bracket + 1 + link_close_bracket)]
                .to_string();

            Some((
                elem.0,
                tag,
                found_close_bracket,
                link_close_bracket,
                link_content,
            ))
        })
        .collect();

    // Build the actual link and image tags out of the collected info
    for link in all_image_links {
        println!(
            "LINK_STARTS: tag_{}, {} + {} (closes: {}); LINK: {}",
            link.1, link.0, link.2, link.3, link.4
        );

        let mut title = None;
        let mut link_text = link.4.to_string();
        let first_title_section = link.4.find(" \"");
        if let Some(first_section) = first_title_section {
            let last_section = &link.4[first_section + 2..].find("\"");
            if let Some(last_section) = last_section {
                title = Some(&link.4[(first_section + 2)..=(first_section + last_section + 1)]);
                link_text = link.4[..first_section].to_string();
            }
        }

        match link.1 {
            "a" => {
                let mut start_tag = String::default();
                start_tag.push_str("<a");
                start_tag.push_str(" href=\"");
                start_tag.push_str(&link_text);
                start_tag.push('"');
                if let Some(title) = title {
                    start_tag.push_str(" title=\"");
                    start_tag.push_str(title);
                    start_tag.push('"');
                }
                start_tag.push('>');

                println!("START_TAG: {start_tag}");

                replacements.push((link.0, start_tag, 1));
                replacements.push((link.0 + link.2, "</a>".into(), link.3 + 3));
            }
            "img" => {
                let mut img_tag = String::default();
                img_tag.push_str("<img");
                img_tag.push_str(" src=\"");
                img_tag.push_str(&link_text);
                img_tag.push('"');

                if let Some(alt_text) = output_text.get((link.0 + 1)..(link.0 + link.2)) {
                    img_tag.push_str(" alt=\"");
                    img_tag.push_str(alt_text);
                    img_tag.push('"');
                }

                if let Some(title) = title {
                    img_tag.push_str(" title=\"");
                    img_tag.push_str(title);
                    img_tag.push('"');
                }
                img_tag.push('>');
                replacements.push((link.0 - 1, img_tag, link.2 + link.3 + 4));
            }
            _ => (),
        }
    }

    // get and filter all valid bold sections
    let all_bold_asterisks: Vec<_> = output_text
        .match_indices("**")
        .map(|elem| elem.0)
        .filter(|elem| {
            for section in &code_sections {
                if section.contains(elem) {
                    return false;
                }
            }
            true
        })
        .collect();

    let all_bold_underscores: Vec<_> = output_text
        .match_indices("__")
        .map(|elem| elem.0)
        .filter(|elem| {
            let prev_char = if *elem > 0 {
                output_text.chars().nth(*elem - 1).unwrap_or('.')
            } else {
                '.'
            };
            let next_char = output_text.chars().nth(*elem + 2).unwrap_or('.');

            if prev_char.is_alphanumeric() && next_char.is_alphanumeric() {
                return false;
            }

            for section in &code_sections {
                if section.contains(elem) {
                    return false;
                }
            }
            true
        })
        .collect();

    let bold_sections: Vec<_> = all_bold_asterisks.chunks_exact(2).collect();
    let bold_underscore_sections: Vec<_> = all_bold_underscores.chunks_exact(2).collect();

    for bold_section in bold_sections {
        replacements.push((bold_section[0], "<strong>".into(), 2));
        replacements.push((bold_section[1], "</strong>".into(), 2));
    }

    for bold_section in bold_underscore_sections {
        replacements.push((bold_section[0], "<strong>".into(), 2));
        replacements.push((bold_section[1], "</strong>".into(), 2));
    }

    // emphasize sections
    let all_em: Vec<_> = output_text.match_indices("*").map(|elem| elem.0).collect();
    let all_em_underscore: Vec<_> = output_text.match_indices("_").map(|elem| elem.0).collect();
    let mut all_em_deduped = all_em.clone();
    let mut all_em_underscore_deduped = all_em_underscore.clone();

    all_em_deduped.retain(|elem| {
        let self_index = *elem;
        let neighbours = all_em.iter().find(|neighbour| {
            **neighbour + 1 == self_index || (**neighbour > 0 && **neighbour - 1 == self_index)
        });
        neighbours.is_none()
    });

    all_em_underscore_deduped.retain(|elem| {
        let self_index = *elem;
        let prev_char = if self_index > 0 {
            output_text.chars().nth(self_index - 1).unwrap_or('.')
        } else {
            '.'
        };
        let next_char = output_text.chars().nth(self_index + 1).unwrap_or('.');

        if prev_char.is_alphanumeric() && next_char.is_alphanumeric() {
            return false;
        }

        let neighbours = all_em_underscore.iter().find(|neighbour| {
            **neighbour + 1 == self_index || (**neighbour > 0 && **neighbour - 1 == self_index)
        });
        neighbours.is_none()
    });

    let em_sections: Vec<_> = all_em_deduped.chunks_exact(2).collect();
    let em_underscore_sections: Vec<_> = all_em_underscore_deduped.chunks_exact(2).collect();

    for em_section in em_sections {
        replacements.push((em_section[0], "<em>".into(), 1));
        replacements.push((em_section[1], "</em>".into(), 1));
    }

    for em_section in em_underscore_sections {
        replacements.push((em_section[0], "<em>".into(), 1));
        replacements.push((em_section[1], "</em>".into(), 1));
    }

    // Finalize Replacements array
    replacements.sort_by_key(|elem| elem.0);
    replacements.reverse();

    for replacement in &replacements {
        println!(
            "REPLACEMENTS: @{} -> {} (-{} chars)",
            replacement.0, replacement.1, replacement.2
        )
    }

    for replacement in replacements {
        output_text.replace_range(
            replacement.0..(replacement.0 + replacement.2),
            &replacement.1,
        );
    }

    output_text
}
