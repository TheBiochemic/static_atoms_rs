use std::collections::HashMap;

use crate::{dist::resolve_tokens_markdown, tests::get_config};

// Currently commented out, because markdown support is WIP
/*#[test]
fn test_minimal_markdown() {
    let config = get_config();
    let in_text = "# h1 Heading\nSome paragraph afterwards".to_owned();
    let out_text = "<h1>h1 Heading</h1>\n<p>Some paragraph afterwards</p>".to_owned();
    let contents = resolve_tokens_markdown(&config, in_text.clone(), 0, &HashMap::new());

    assert_eq!(out_text, contents);
}*/
