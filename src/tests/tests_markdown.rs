use std::collections::HashMap;

use crate::{dist::markdown::resolve_tokens_markdown, tests::get_config};

pub fn test_md_in_out(in_text: &str, out_text: &str) {
    let config = get_config();
    let contents = resolve_tokens_markdown(&config, in_text.to_owned(), 0, &HashMap::new());
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
