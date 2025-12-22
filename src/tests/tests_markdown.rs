use std::collections::HashMap;

use crate::{dist::markdown::resolve_tokens_markdown, tests::get_config};

pub fn test_md_in_out(in_text: &str, out_text: &str) {
    let config = get_config();
    let contents = resolve_tokens_markdown(
        &config,
        in_text.to_owned(),
        0,
        &HashMap::new(),
        ("<p>", "</p>"),
    );
    assert_eq!(out_text.to_owned(), contents.to_owned());
}

#[test]
fn test_minimal_md() {
    test_md_in_out(
        "# h1 Heading\nSome paragraph afterwards",
        "<h1>h1 Heading</h1><p>Some paragraph afterwards</p>",
    );
}

#[test]
fn test_multiline_paragraph_md() {
    test_md_in_out(
        "Some paragraph\nsome more paragraph\nthe last line",
        "<p>Some paragraph some more paragraph the last line</p>",
    );
}

#[test]
fn test_inbetween_heading_md() {
    test_md_in_out(
        "Some paragraph\n## some heading\nthe last line",
        "<p>Some paragraph</p><h2>some heading</h2><p>the last line</p>",
    );
}

#[test]
fn test_emphasis_md() {
    test_md_in_out(
        "this `paragraph` has **multiple** different __inline__ _stylings_. Even *these* ones!\n## Heading with inline **emphasized** component!",
        "<p>this <code>paragraph</code> has <strong>multiple</strong> different <strong>inline</strong> <em>stylings</em>. Even <em>these</em> ones!</p><h2>Heading with inline <strong>emphasized</strong> component!</h2>",
    );
}

#[test]
fn test_extra_emphasis_md() {
    test_md_in_out(
        "this text is pa**rtly emphasized**. _this_ one is **ne*st*ed**.\nThis_is_a __multiline emphasis,\nthat crosses across__ a newline boundary",
        "<p>this text is pa<strong>rtly emphasized</strong>. <em>this</em> one is <strong>ne<em>st</em>ed</strong>. This_is_a <strong>multiline emphasis, that crosses across</strong> a newline boundary</p>",
    );
}

#[test]
fn test_space_code_md() {
    test_md_in_out(
        "    var foo = function (bar) {\n      return bar++;\n    };\n    \n    console.log(foo(5));",
        "<pre><code>var foo = function (bar) {\n  return bar++;\n};\n\nconsole.log(foo(5));</code></pre>",
    );
}

#[test]
fn test_inline_code_md() {
    test_md_in_out(
        "this is example `text` and it contains `a few ``inline code` snippets `(that ```sometimes``` dont` behave)",
        "<p>this is example <code>text</code> and it contains <code>a few ``inline code</code> snippets <code>(that ```sometimes``` dont</code> behave)</p>",
    );
}

#[test]
fn test_horizontal_md() {
    test_md_in_out(
        "***\n**********\n___\n_______\n---\n---------\n**",
        "</hr></hr></hr></hr></hr></hr><p>**</p>",
    );
}

#[test]
fn test_blockquotes_md() {
    test_md_in_out(
        "> this is the first level\n >> this is second\n > > > this is third level",
        "<blockquote><p>this is the first level</p><blockquote><p>this is second</p><blockquote><p>this is third level</p></blockquote></blockquote></blockquote>",
    );
}

#[test]
fn test_formatted_blockquotes_md() {
    test_md_in_out(
        "> ## this is a headline\n >> this is *emphasized*\n > > > this is __bold__",
        "<blockquote><h2>this is a headline</h2><blockquote><p>this is <em>emphasized</em></p><blockquote><p>this is <strong>bold</strong></p></blockquote></blockquote></blockquote>",
    );
}

#[test]
fn test_code_block_fence_md() {
    test_md_in_out(
        "```\nThis is test code\n\nthis is text after the empty space\n```\n\n  ```\n    inset text\n\n  ```",
        "<pre><code>This is test code\n\nthis is text after the empty space</code></pre><pre><code>  inset text\n</code></pre>",
    );
}

#[test]
fn test_links_md() {
    test_md_in_out(
        "i have an inline [link here, click me](https://test.com/1) (if you want)\n[here](https://test.com/2 \"tooltip\") \
        are multiple [links](https://test.com/3) available to click. [This is not] a link",
        "<p>i have an inline <a href=\"https://test.com/1\">link here, click me</a> (if you want) <a href=\"https://test.com/2\" \
        title=\"tooltip\">here</a> are multiple <a href=\"https://test.com/3\">links</a> available to click. [This is not] a link</p>",
    );
}

#[test]
fn test_image_md() {
    test_md_in_out(
        "this is an image: ![Image Alt](https://test.com/image.png \"tooltip\")\n\nand this is an image within a link: [![Image Alt2](https://test.com/image2.png)](https://test.com/1)",
        "<p>this is an image: <img src=\"https://test.com/image.png\" alt=\"Image Alt\" title=\"tooltip\"></p>\
        <p>and this is an image within a link: <a href=\"https://test.com/1\"><img src=\"https://test.com/image2.png\" alt=\"Image Alt2\"></a></p>",
    );
}

#[test]
fn test_list_unordered_md() {
    test_md_in_out(
        "+ primary list item\n  - second level *is emphasized*\n    * third level first\n    - third level second\n    + third level third\n * back to first",
        "<ul><li>primary list item <ul><li>second level <em>is emphasized</em><ul><li>third level first</li><li>third level second</li><li>third level third</li></ul></li></ul></li><li>back to first</li></ul>",
    );
}

#[test]
fn test_list_ordered_md() {
    test_md_in_out(
        "1) first line\n2) second line\n99. line with dot\n1. another line with dots. *Three* digits\n   - nested list",
        "<ol><li>first line</li><li>second line</li></ol><ol start=\"99\"></li>line with dot</li><li>another line with dots. <em>Three</em> digits<ul><li>nested list</li></ul></li></ol>",
    );
}
