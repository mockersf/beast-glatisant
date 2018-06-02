use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
use failure;
use futures::future::{self, Future};
use http::uri::Uri;
use linkify::LinkFinder;
use serde_urlencoded;

use github;

#[derive(Clone, Debug)]
pub struct Code {
    pub code: String,
    pub gist_id: Option<String>,
}

#[derive(Deserialize)]
struct PlaygroundQueryParams {
    gist: String,
}

pub fn get_code_samples(
    doc: &str,
    token: Option<String>,
) -> Box<Future<Item = Vec<Code>, Error = failure::Error>> {
    let arena = Arena::new();

    let root = parse_document(&arena, doc, &ComrakOptions::default());

    let mut code_blocks: Vec<Box<Future<Item = Code, Error = failure::Error>>> = vec![];

    fn iter_nodes<'a>(
        node: &'a AstNode<'a>,
        code_blocks: &mut Vec<Box<Future<Item = Code, Error = failure::Error>>>,
        token: Option<String>,
    ) {
        match &mut node.data.borrow_mut().value {
            &mut NodeValue::CodeBlock(ref code) => {
                if let Ok(code_block) = String::from_utf8(code.literal.clone()) {
                    code_blocks.push(Box::new(future::ok(Code {
                        code: code_block,
                        gist_id: None,
                    })));
                }
            }
            &mut NodeValue::Link(ref link) => {
                if let Ok(link) = String::from_utf8(link.url.clone()) {
                    if let Ok(url) = link.parse::<Uri>() {
                        if url.host() == Some("play.rust-lang.org") {
                            if let Ok(query_params) = serde_urlencoded::from_str::<
                                PlaygroundQueryParams,
                            >(url.query().unwrap_or(""))
                            {
                                let code =
                                    github::gist::get_gist(&query_params.gist, token.clone())
                                        .map(|gist| {
                                            gist.files.values().next().unwrap().content.clone()
                                        })
                                        .map(|code| Code {
                                            code: code,
                                            gist_id: Some(query_params.gist),
                                        });
                                code_blocks.push(Box::new(code));
                            }
                        }
                    }
                }
            }
            &mut NodeValue::Text(ref mut text) => {
                let finder = LinkFinder::new();
                let text = String::from_utf8(text.to_vec()).unwrap();
                finder.links(&text).for_each(|link| {
                    if let Ok(url) = link.as_str().parse::<Uri>() {
                        if url.host() == Some("play.rust-lang.org") {
                            if let Ok(query_params) = serde_urlencoded::from_str::<
                                PlaygroundQueryParams,
                            >(url.query().unwrap_or(""))
                            {
                                let code =
                                    github::gist::get_gist(&query_params.gist, token.clone())
                                        .map(|gist| {
                                            gist.files.values().next().unwrap().content.clone()
                                        })
                                        .map(|code| Code {
                                            code: code,
                                            gist_id: Some(query_params.gist),
                                        });
                                code_blocks.push(Box::new(code));
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

    Box::new(future::join_all(code_blocks))
}
