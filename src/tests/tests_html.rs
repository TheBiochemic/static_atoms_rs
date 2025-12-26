use std::{collections::HashMap, env, fs::exists, path::PathBuf};

use crate::{
    dist::{build_default_context, get_pages, resolve_tokens_html, run_dist},
    filetype::FileType,
    tests::{
        create_index_page, create_test_page, create_test_section, get_config, get_config_multi,
    },
};

#[test]
fn parse_simple() {
    let config = get_config();
    let in_text = "<html><body><p>TEST</p></body></html>".to_owned();
    let contents = resolve_tokens_html("".into(), &config, &in_text, 0, &HashMap::new());

    assert_eq!(in_text, contents);
}

#[test]
fn parse_embed() {
    let config = get_config();
    create_test_section(
        FileType::FileHTML,
        &config,
        vec![],
        "my_embed",
        "<p>TEST</p>",
    );
    let in_text = "<html><body><## my_embed></body></html>".to_owned();
    let out_text = "<html><body><p>TEST</p></body></html>".to_owned();
    let contents = resolve_tokens_html("".into(), &config, &in_text, 0, &HashMap::new());

    assert_eq!(out_text, contents);
}

#[test]
fn parse_embed_brackets() {
    let config = get_config();
    create_test_section(
        FileType::FileHTML,
        &config,
        vec![],
        "my_embed",
        "<p>TEST</p>",
    );
    let in_text = "<html><body><## my_embed()></body></html>".to_owned();
    let out_text = "<html><body><p>TEST</p></body></html>".to_owned();
    let contents = resolve_tokens_html("".into(), &config, &in_text, 0, &HashMap::new());

    assert_eq!(out_text, contents);
}

#[test]
fn parse_variable_embed() {
    let config = get_config();
    let in_text = "<html><body><## {my_embed}></body></html>".to_owned();
    let out_text = "<html><body><p>TEST</p></body></html>".to_owned();
    let mut context = HashMap::new();
    context.insert("my_embed".to_owned(), "<p>TEST</p>".to_owned());

    let contents = resolve_tokens_html("".into(), &config, &in_text, 0, &context);

    assert_eq!(out_text, contents);
}

#[test]
fn parse_folder_embed() {
    let config = get_config();
    let in_text = "<html><body><## embed_list[]></body></html>".to_owned();
    let out_text = "<html><body><p>1</p><p>2</p><p>3</p><p>4</p></body></html>".to_owned();
    create_test_section(
        FileType::FileHTML,
        &config,
        vec!["embed_list"],
        "1",
        "<p>1</p>",
    );
    create_test_section(
        FileType::FileHTML,
        &config,
        vec!["embed_list"],
        "2",
        "<p>2</p>",
    );
    create_test_section(
        FileType::FileHTML,
        &config,
        vec!["embed_list"],
        "3",
        "<p>3</p>",
    );
    create_test_section(
        FileType::FileHTML,
        &config,
        vec!["embed_list"],
        "4",
        "<p>4</p>",
    );

    let contents = resolve_tokens_html("".into(), &config, &in_text, 0, &HashMap::new());

    assert_eq!(out_text, contents);
}

#[test]
fn parse_folder_limit_embed() {
    let config = get_config();

    let in_text = "<html><body><## embed_list[..2]></body></html>".to_owned();
    let out_text = "<html><body><p>1</p><p>2</p></body></html>".to_owned();
    create_test_section(
        FileType::FileHTML,
        &config,
        vec!["embed_list"],
        "1",
        "<p>1</p>",
    );
    create_test_section(
        FileType::FileHTML,
        &config,
        vec!["embed_list"],
        "2",
        "<p>2</p>",
    );
    create_test_section(
        FileType::FileHTML,
        &config,
        vec!["embed_list"],
        "3",
        "<p>3</p>",
    );
    create_test_section(
        FileType::FileHTML,
        &config,
        vec!["embed_list"],
        "4",
        "<p>4</p>",
    );

    let contents = resolve_tokens_html("".into(), &config, &in_text, 0, &HashMap::new());

    assert_eq!(out_text, contents);
}

#[test]
fn parse_default_variables() {
    let config = get_config();

    create_test_page(
        FileType::FileHTML,
        &config,
        vec![],
        "testpage",
        "<p>test</p>",
    );

    let pages_vec = get_pages(&config);
    let version = env!("CARGO_PKG_VERSION").to_string();
    let appname = env!("CARGO_PKG_NAME").to_string();
    let applink = format!(
        "<a href=\"{}\">{}</a>",
        env!("CARGO_PKG_HOMEPAGE"),
        env!("CARGO_PKG_NAME")
    )
    .to_string();
    let pages = "<ul class=\"siteindex\"><li><a href=\"/\">index.html</a></li><li><a href=\"/pages/testpage.html\">pages/testpage.html</a></li></ul>";
    let context = build_default_context(&config, &pages_vec);

    let in_text =
        "<html><body><## {_VERSION}><## {_APPNAME}><## {_APPLINK}><## {_PAGES}></body></html>"
            .to_owned();

    let out_text = format!("<html><body>{version}{appname}{applink}{pages}</body></html>");

    let contents = resolve_tokens_html("".into(), &config, &in_text, 0, &context);

    assert_eq!(out_text, contents);
}

#[test]
fn parse_parametric() {
    let config = get_config();
    let in_text = "<html><body><## embed_name(var1=\"v1\" var2=\"v2\")></body></html>".to_owned();
    let out_text = "<html><body><p>v1v2</p></body></html>".to_owned();
    create_test_section(
        FileType::FileHTML,
        &config,
        vec![],
        "embed_name",
        "<p><## {var1}><## {var2}></p>",
    );

    let contents = resolve_tokens_html("".into(), &config, &in_text, 0, &HashMap::new());

    assert_eq!(out_text, contents);
}

#[test]
fn parse_parametric_edge_cases() {
    let config = get_config();
    let in_text =
        "<html><body><## embed_name2(var1=\"v1()\" var2=\"<## {var3}>\")></body></html>".to_owned();
    let out_text = "<html><body><p>v1()v2</p></body></html>".to_owned();
    create_test_section(
        FileType::FileHTML,
        &config,
        vec![],
        "embed_name2",
        "<p><## {var1}><## {var2}></p>",
    );

    let mut context = HashMap::new();
    context.insert("var3".to_string(), "v2".to_string());
    let contents = resolve_tokens_html("".into(), &config, &in_text, 0, &context);

    assert_eq!(out_text, contents);
}

#[test]
fn parse_multi_page() {
    let config = get_config_multi();

    create_index_page(FileType::FileHTML, &config, "<p>index</p>");

    create_test_page(FileType::FileHTML, &config, vec![], "page1", "<p>page1</p>");

    create_test_page(FileType::FileHTML, &config, vec![], "page2", "<p>page2</p>");

    create_test_page(
        FileType::FileHTML,
        &config,
        vec!["sub"],
        "subpage1",
        "<p>subpage1</p>",
    );

    create_test_page(
        FileType::FileHTML,
        &config,
        vec!["sub"],
        "subpage2",
        "<p>subpage2</p>",
    );

    create_test_page(
        FileType::FileHTML,
        &config,
        vec!["sub", "sub2"],
        "subpage3",
        "<p>subpage3</p>",
    );

    run_dist(&config);

    let root = config.root.clone();

    assert!(exists(root.join(PathBuf::from("dist/index.html"))).unwrap_or(false));
    assert!(exists(root.join(PathBuf::from("dist/pages/page1.html"))).unwrap_or(false));
    assert!(exists(root.join(PathBuf::from("dist/pages/page2.html"))).unwrap_or(false));
    assert!(exists(root.join(PathBuf::from("dist/pages/sub/subpage1.html"))).unwrap_or(false));
    assert!(exists(root.join(PathBuf::from("dist/pages/sub/subpage2.html"))).unwrap_or(false));
    assert!(exists(root.join(PathBuf::from("dist/pages/sub/sub2/subpage3.html"))).unwrap_or(false));
}
