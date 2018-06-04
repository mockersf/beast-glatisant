extern crate env_logger;
#[macro_use]
extern crate log;

extern crate actix_web;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
extern crate failure;
extern crate futures;

extern crate beast_glatisant;

use std::iter;

use actix_web::{http, middleware, server, App, HttpMessage, HttpRequest, HttpResponse, Path};
use futures::future::{self, Future};
use structopt::StructOpt;

#[derive(Deserialize, Debug, Hash, Eq, PartialEq)]
struct IssueDesignation {
    owner: String,
    repo: String,
    issue: u32,
}

#[derive(Serialize, Debug, Clone)]
struct CodeAndClippy {
    from: String,
    code: String,
    clippy: Option<String>,
}

fn extract_token(req: HttpRequest) -> Option<String> {
    req.headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|value| {
            if value
                .to_str()
                .unwrap()
                .to_lowercase()
                .starts_with("bearer ")
            {
                value.to_str().ok().map(|v| v[7..].to_string())
            } else {
                None
            }
        })
}

fn issue(
    info: (Path<IssueDesignation>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = failure::Error> {
    let token = extract_token(info.1);
    beast_glatisant::github::issue::get_issue(&info.0.owner, &info.0.repo, info.0.issue, token)
        .map(|issue| HttpResponse::Ok().json(issue))
}

fn clippied(
    info: (Path<IssueDesignation>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = failure::Error> {
    let token = extract_token(info.1.clone());
    let token2 = token.clone();
    beast_glatisant::github::issue::get_issue(
        &info.0.owner,
        &info.0.repo,
        info.0.issue,
        token.clone(),
    ).and_then(move |issue| {
        beast_glatisant::github::issue::get_comments(
            &info.0.owner,
            &info.0.repo,
            info.0.issue,
            token.clone(),
        ).map(move |comments| {
            iter::once((issue.html_url, issue.body))
                .chain(
                    comments
                        .iter()
                        .map(|comment| (comment.html_url.clone(), comment.body.clone())),
                )
                .collect()
        })
    })
        .and_then(move |issue_and_comments: Vec<(String, String)>| {
            future::join_all(
                issue_and_comments
                    .iter()
                    .map(move |(from, text)| {
                        let from = from.clone();
                        beast_glatisant::markdown::get_code_samples(&text.clone(), token2.clone())
                            .map(move |code_blocks| {
                                code_blocks
                                    .iter()
                                    .map(|code_block| (from.clone(), code_block.clone()))
                                    .collect::<Vec<(String, beast_glatisant::markdown::Code)>>()
                            })
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .and_then(|code_blocks| {
            future::join_all(
                code_blocks
                    .iter()
                    .flat_map(|cbs| cbs)
                    .map(move |(from, cb)| {
                        let cb = cb.clone();
                        let from = from.clone();
                        clippy_if_rust(&cb).map(|clippy| CodeAndClippy {
                            from: from,
                            code: cb.code,
                            clippy: clippy,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .map(|code_blocks| HttpResponse::Ok().json(code_blocks))
}

#[derive(StructOpt, Debug)]
#[structopt(name = "beast_glatisant", author = "")]
struct Config {
    /// Host to listen on
    #[structopt(long = "host", short = "h", default_value = "0.0.0.0")]
    pub host: String,
    /// Port to listen on
    #[structopt(env = "PORT", long = "port", short = "p", default_value = "7878")]
    pub port: u16,
}

fn main() {
    env_logger::init();

    let config = Config::from_args();

    let addr = format!("{}:{}", config.host, config.port);
    info!("listening on http://{}", addr);
    server::new(|| {
        App::new()
            .middleware(middleware::Logger::default())
            .resource("/{owner}/{repo}/issues/{issue}", |r| {
                r.method(http::Method::GET).with_async(issue)
            })
            .resource("/{owner}/{repo}/issues/{issue}/clippy", |r| {
                r.method(http::Method::GET).with_async(clippied)
            })
    }).bind(&addr)
        .unwrap()
        .run();
}

fn is_rust(code: &beast_glatisant::markdown::Code) -> bool {
    if let Some(ref language) = code.language {
        language.to_ascii_lowercase() == "rust"
    } else {
        false
    }
}

fn clippy_if_rust(
    code: &beast_glatisant::markdown::Code,
) -> Box<Future<Item = Option<String>, Error = failure::Error>> {
    if is_rust(code) {
        Box::new(
            beast_glatisant::playground::ask_playground_simpl(
                &code.code,
                beast_glatisant::playground::Action::Clippy,
            ).map(|v| Some(v)),
        )
    } else {
        Box::new(future::ok(None))
    }
}
