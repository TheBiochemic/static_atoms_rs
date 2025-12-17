use std::{collections::HashMap, ops::Sub};

use crate::Configuration;

pub fn resolve_tokens_markdown(
    config: &Configuration,
    contents: String,
    depth: u8,
    context: &HashMap<String, String>,
) -> String {
    let mut converted = String::new();

    #[derive(PartialEq)]
    enum TopLevelBlock {
        Nothing,
        Paragraph,
        CodeBlockSpace,
    }

    let mut top_level_block = TopLevelBlock::Nothing;
    let mut inner_content = String::new();

    fn finish_blocks(
        config: &Configuration,
        depth: u8,
        context: &HashMap<String, String>,
        converted: &mut String,
        inner_content: &mut String,
        block: &mut TopLevelBlock,
    ) {
        match block {
            TopLevelBlock::Nothing => (),
            TopLevelBlock::Paragraph => {
                let paragraph = resolve_markdown_paragraph(&config, inner_content, depth, context);
                converted.push_str(&paragraph);
                converted.push_str("</p>")
            }
            TopLevelBlock::CodeBlockSpace => {
                converted.push_str(inner_content);
                converted.push_str("</code></pre>")
            }
        }
        *block = TopLevelBlock::Nothing;
        *inner_content = Default::default();
    }

    for line in contents.lines() {
        //If the line is the start of a code block, create the code block
        if line.starts_with("    ") {
            if top_level_block != TopLevelBlock::CodeBlockSpace {
                finish_blocks(
                    config,
                    depth,
                    context,
                    &mut converted,
                    &mut inner_content,
                    &mut top_level_block,
                );
                converted.push_str("<pre><code>");
                top_level_block = TopLevelBlock::CodeBlockSpace;
                let code_content = line.to_owned().split_off(4);
                inner_content.push_str(code_content.as_str());
                continue;
            } else {
                let code_content = "\n".to_owned() + &line.to_owned().split_off(4);
                inner_content.push_str(code_content.as_str());
                continue;
            }
        }

        //If the line is empty, ignore it
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            finish_blocks(
                config,
                depth,
                context,
                &mut converted,
                &mut inner_content,
                &mut top_level_block,
            );
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
                config,
                depth,
                context,
                &mut converted,
                &mut inner_content,
                &mut top_level_block,
            );
            converted.push_str("</hr>");
            continue;
        }

        // If the line is a heading
        let mut header_type = 0usize;
        for character in trimmed_line.chars() {
            if character == '#' {
                header_type += 1;
            };
        }

        if header_type > 0 && header_type < 6 {
            finish_blocks(
                config,
                depth,
                context,
                &mut converted,
                &mut inner_content,
                &mut top_level_block,
            );
            let paragraph = resolve_markdown_paragraph(
                &config,
                trimmed_line[(header_type + 1)..].trim(),
                depth,
                context,
            );
            converted.push_str(format!("<h{header_type}>{paragraph}</h{header_type}>").as_str());
            continue;
        }

        // If the line is just a simple paragraph
        if top_level_block != TopLevelBlock::Paragraph {
            finish_blocks(
                config,
                depth,
                context,
                &mut converted,
                &mut inner_content,
                &mut top_level_block,
            );
            converted.push_str("<p>");
            top_level_block = TopLevelBlock::Paragraph;
        } else {
            inner_content.push(' ');
        }

        inner_content.push_str(trimmed_line);
    }

    finish_blocks(
        config,
        depth,
        context,
        &mut converted,
        &mut inner_content,
        &mut top_level_block,
    );
    converted
}

fn resolve_markdown_paragraph(
    config: &&Configuration,
    paragraph: &str,
    depth: u8,
    context: &HashMap<String, String>,
) -> String {
    // Relevant data
    let mut output_text = String::from(paragraph);

    // replacements array; Format (position, new_string, old_length)
    let mut replacements: Vec<(usize, &str, usize)> = Default::default();

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
        replacements.push((*code_section.start(), "<code>", 1));
        replacements.push((*code_section.end(), "</code>", 1));
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
        replacements.push((bold_section[0], "<strong>", 2));
        replacements.push((bold_section[1], "</strong>", 2));
    }

    for bold_section in bold_underscore_sections {
        replacements.push((bold_section[0], "<strong>", 2));
        replacements.push((bold_section[1], "</strong>", 2));
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
        replacements.push((em_section[0], "<em>", 1));
        replacements.push((em_section[1], "</em>", 1));
    }

    for em_section in em_underscore_sections {
        replacements.push((em_section[0], "<em>", 1));
        replacements.push((em_section[1], "</em>", 1));
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
            replacement.1,
        );
    }

    output_text
}
