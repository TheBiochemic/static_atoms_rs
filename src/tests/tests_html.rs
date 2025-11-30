use std::{collections::HashMap, env};

use crate::{
    dist::{build_default_context, resolve_tokens},
    filetype::FileType,
    tests::{create_test_page, get_config},
};

#[test]
fn parse_simple() {
    let config = get_config();
    let in_text = "<html><body><p>TEST</p></body></html>".to_owned();
    let contents = resolve_tokens("".into(), &config, in_text.clone(), 0, &HashMap::new());

    assert_eq!(in_text, contents);
}

#[test]
fn parse_embed() {
    let config = get_config();
    create_test_page(FileType::FileHTML, &config, None, "my_embed", "<p>TEST</p>");
    let in_text = "<html><body><## my_embed></body></html>".to_owned();
    let out_text = "<html><body><p>TEST</p></body></html>".to_owned();
    let contents = resolve_tokens("".into(), &config, in_text.clone(), 0, &HashMap::new());

    assert_eq!(out_text, contents);
}

#[test]
fn parse_embed_brackets() {
    let config = get_config();
    create_test_page(FileType::FileHTML, &config, None, "my_embed", "<p>TEST</p>");
    let in_text = "<html><body><## my_embed()></body></html>".to_owned();
    let out_text = "<html><body><p>TEST</p></body></html>".to_owned();
    let contents = resolve_tokens("".into(), &config, in_text.clone(), 0, &HashMap::new());

    assert_eq!(out_text, contents);
}

#[test]
fn parse_variable_embed() {
    let config = get_config();
    let in_text = "<html><body><## {my_embed}></body></html>".to_owned();
    let out_text = "<html><body><p>TEST</p></body></html>".to_owned();
    let mut context = HashMap::new();
    context.insert("my_embed".to_owned(), "<p>TEST</p>".to_owned());

    let contents = resolve_tokens("".into(), &config, in_text.clone(), 0, &context);

    assert_eq!(out_text, contents);
}

#[test]
fn parse_folder_embed() {
    let config = get_config();
    let in_text = "<html><body><## embed_list[]></body></html>".to_owned();
    let out_text = "<html><body><p>1</p><p>2</p><p>3</p><p>4</p></body></html>".to_owned();
    create_test_page(
        FileType::FileHTML,
        &config,
        Some("embed_list"),
        "1",
        "<p>1</p>",
    );
    create_test_page(
        FileType::FileHTML,
        &config,
        Some("embed_list"),
        "2",
        "<p>2</p>",
    );
    create_test_page(
        FileType::FileHTML,
        &config,
        Some("embed_list"),
        "3",
        "<p>3</p>",
    );
    create_test_page(
        FileType::FileHTML,
        &config,
        Some("embed_list"),
        "4",
        "<p>4</p>",
    );

    let contents = resolve_tokens("".into(), &config, in_text.clone(), 0, &HashMap::new());

    assert_eq!(out_text, contents);
}

#[test]
fn parse_folder_limit_embed() {
    let config = get_config();
    let in_text = "<html><body><## embed_list[..2]></body></html>".to_owned();
    let out_text = "<html><body><p>1</p><p>2</p></body></html>".to_owned();
    create_test_page(
        FileType::FileHTML,
        &config,
        Some("embed_list"),
        "1",
        "<p>1</p>",
    );
    create_test_page(
        FileType::FileHTML,
        &config,
        Some("embed_list"),
        "2",
        "<p>2</p>",
    );
    create_test_page(
        FileType::FileHTML,
        &config,
        Some("embed_list"),
        "3",
        "<p>3</p>",
    );
    create_test_page(
        FileType::FileHTML,
        &config,
        Some("embed_list"),
        "4",
        "<p>4</p>",
    );

    let contents = resolve_tokens("".into(), &config, in_text.clone(), 0, &HashMap::new());

    assert_eq!(out_text, contents);
}

#[test]
fn parse_default_variables() {
    let config = get_config();
    let version = env!("CARGO_PKG_VERSION").to_string();
    let appname = env!("CARGO_PKG_NAME").to_string();
    let applink = format!(
        "<a href=\"{}\">{}</a>",
        env!("CARGO_PKG_HOMEPAGE"),
        env!("CARGO_PKG_NAME")
    )
    .to_string();
    let pages = "<ul class=\"siteindex\"></ul>";
    let context = build_default_context(&config);

    let in_text =
        "<html><body><## {_VERSION}><## {_APPNAME}><## {_APPLINK}><## {_PAGES}></body></html>"
            .to_owned();

    let out_text = format!("<html><body>{version}{appname}{applink}{pages}</body></html>");

    let contents = resolve_tokens("".into(), &config, in_text.clone(), 0, &context);

    assert_eq!(out_text, contents);
}

#[test]
fn parse_parametric() {
    let config = get_config();
    let in_text = "<html><body><## embed_name(var1=\"v1\" var2=\"v2\")></body></html>".to_owned();
    let out_text = "<html><body><p>v1v2</p></body></html>".to_owned();
    create_test_page(
        FileType::FileHTML,
        &config,
        None,
        "embed_name",
        "<p><## {var1}><## {var2}></p>",
    );

    let contents = resolve_tokens("".into(), &config, in_text.clone(), 0, &HashMap::new());

    assert_eq!(out_text, contents);
}

#[test]
fn parse_parametric_edge_cases() {
    let config = get_config();
    let in_text =
        "<html><body><## embed_name2(var1=\"v1()\" var2=\"<## {var3}>\" var3=\"v2\")></body></html>".to_owned();
    let out_text = "<html><body><p>v1()v2</p></body></html>".to_owned();
    create_test_page(
        FileType::FileHTML,
        &config,
        None,
        "embed_name2",
        "<p><## {var1}><## {var2}></p>",
    );

    let contents = resolve_tokens("".into(), &config, in_text.clone(), 0, &HashMap::new());

    assert_eq!(out_text, contents);
}
