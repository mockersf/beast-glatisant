extern crate actix_web;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;

extern crate beast_glatisant;

use std::iter;

use actix_web::{http, server, App, HttpMessage, HttpRequest, Json, Path, Responder};
use structopt::StructOpt;

#[derive(Deserialize, Debug, Hash, Eq, PartialEq)]
struct IssueDesignation {
    owner: String,
    repo: String,
    issue: u32,
}

#[derive(Serialize, Debug)]
enum Response {
    Issue(beast_glatisant::github::issue::Issue),
    CodeAndClippy(Vec<CodeAndClippy>),
    Error { msg: String },
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

fn issue(info: (Path<IssueDesignation>, HttpRequest)) -> impl Responder {
    let token = extract_token(info.1);
    if let Some(issue) =
        beast_glatisant::github::issue::get_issue(&info.0.owner, &info.0.repo, info.0.issue, token)
    {
        Json(Response::Issue(issue))
    } else {
        Json(Response::Error {
            msg: "no issue matching request".to_string(),
        })
    }
}

fn clippied(info: (Path<IssueDesignation>, HttpRequest)) -> impl Responder {
    let token = extract_token(info.1);
    if let Some(issue) = beast_glatisant::github::issue::get_issue(
        &info.0.owner,
        &info.0.repo,
        info.0.issue,
        token.clone(),
    ) {
        let a: Vec<Vec<CodeAndClippy>> = iter::once((&issue.html_url, &issue.body))
            .chain(
                beast_glatisant::github::issue::get_comments(
                    &info.0.owner,
                    &info.0.repo,
                    info.0.issue,
                    token.clone(),
                ).expect("getting comments of issue")
                    .iter()
                    .map(|comment| (&comment.html_url, &comment.body)),
            )
            .map(|(from, text)| {
                beast_glatisant::markdown::get_code_samples(&text.clone(), token.clone())
                    .iter()
                    .map(move |code_block| CodeAndClippy {
                        from: from.to_string(),
                        code: code_block.code.clone(),
                        clippy: if is_rust(&code_block.code) {
                            Some(
                                beast_glatisant::playground::ask_playground_simpl(
                                    &code_block.code,
                                    beast_glatisant::playground::Action::Clippy,
                                ).to_string(),
                            )
                        } else {
                            None
                        },
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        Json(Response::CodeAndClippy(
            a.iter().flat_map(|x| x.iter().cloned()).collect(),
        ))
    } else {
        Json(Response::Error {
            msg: "no issue matching request".to_string(),
        })
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "beast_glatisant", author = "")]
struct Config {
    /// Host to listen on
    #[structopt(long = "host", short = "h", default_value = "0.0.0.0")]
    pub host: String,
    /// Port to listen on
    #[structopt(long = "port", short = "p", default_value = "7878")]
    pub port: u16,
}

fn main() {
    let config = Config::from_args();

    let addr = format!("{}:{}", config.host, config.port);
    println!("listening on http://{}", addr);
    server::new(|| {
        App::new()
            .route("/{owner}/{repo}/issues/{issue}", http::Method::GET, issue)
            .route(
                "/{owner}/{repo}/issues/{issue}/clippy",
                http::Method::GET,
                clippied,
            )
    }).bind(&addr)
        .unwrap()
        .run();
}

fn is_rust(code: &str) -> bool {
    !code.contains("for further information visit https://rust-lang-nursery.github.io/rust-clippy")
}
