use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
use linkify::LinkFinder;
use reqwest::Url;

use github;

pub struct Code {
    pub code: String,
    pub gist_id: Option<String>,
}

pub fn get_code_samples(doc: &str, token: Option<String>) -> Vec<Code> {
    let arena = Arena::new();

    let root = parse_document(&arena, doc, &ComrakOptions::default());

    let mut code_blocks: Vec<Code> = vec![];

    fn iter_nodes<'a>(node: &'a AstNode<'a>, code_blocks: &mut Vec<Code>, token: Option<String>) {
        match &mut node.data.borrow_mut().value {
            &mut NodeValue::CodeBlock(ref code) => {
                if let Ok(code_block) = String::from_utf8(code.literal.clone()) {
                    code_blocks.push(Code {
                        code: code_block,
                        gist_id: None,
                    });
                }
            }
            &mut NodeValue::Link(ref link) => {
                if let Ok(link) = String::from_utf8(link.url.clone()) {
                    if let Ok(url) = Url::parse(&link) {
                        if url.domain() == Some("play.rust-lang.org") {
                            if let Some(gist_id) =
                                url.query_pairs().find(|(k, _)| k == "gist").map(|(_, v)| v)
                            {
                                let gist = github::gist::get_gist(&gist_id, token).unwrap();
                                code_blocks.push(Code {
                                    code: gist.files.values().next().unwrap().content.clone(),
                                    gist_id: Some(gist_id.into_owned()),
                                });
                            }
                        }
                    }
                }
            }
            &mut NodeValue::Text(ref mut text) => {
                let finder = LinkFinder::new();
                let text = String::from_utf8(text.to_vec()).unwrap();
                finder.links(&text).for_each(|link| {
                    if let Ok(url) = Url::parse(&link.as_str()) {
                        if url.domain() == Some("play.rust-lang.org") {
                            if let Some(gist_id) =
                                url.query_pairs().find(|(k, _)| k == "gist").map(|(_, v)| v)
                            {
                                let gist = github::gist::get_gist(&gist_id, token.clone()).unwrap();
                                code_blocks.push(Code {
                                    code: gist.files.values().next().unwrap().content.clone(),
                                    gist_id: Some(gist_id.into_owned()),
                                });
                            }
                        }
                    }
                });
            }
            _ => {
                for c in node.children() {
                    iter_nodes(c, code_blocks, token.clone());
                }
            }
        }
    }
    iter_nodes(root, &mut code_blocks, token);

    code_blocks
}
