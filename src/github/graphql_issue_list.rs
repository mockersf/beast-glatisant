use std::iter;

use actix_web::{client, HttpMessage};
use chrono::prelude::*;
use failure;
use futures::future::Future;
use http::header::{AUTHORIZATION, USER_AGENT};

macro_rules! query {
    () => {
r#"{{
    "query":"{{repository(owner: \"{}\", name: \"{}\") {{issues(first: 100, states: OPEN, orderBy: {{field: UPDATED_AT, direction: DESC}}) {{nodes {{number url title url body lastEditedAt createdAt updatedAt comments(last: 100) {{nodes {{url body lastEditedAt createdAt updatedAt}}}}}}}}}}}}"
}}"#
    };
}

use serde_json;
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct Test {
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLReply {
    pub data: RepositoryNode,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryNode {
    pub repository: Repository,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub issues: Issues,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issues {
    pub nodes: Vec<Issue>,
}

// #[derive(Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct IssueNode {
//     #[serde(flatten)]
//     pub issues: Vec<Issue>,
// }

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub number: u32,
    pub url: String,
    pub title: String,
    pub body: String,
    pub last_edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub comments: Comments,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comments {
    pub nodes: Vec<Comment>,
}

// #[derive(Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct CommentNode {
//     #[serde(flatten)]
//     pub comments: Vec<Comment>,
// }

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub url: String,
    pub body: String,
    pub last_edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn graphql(
    owner: &str,
    repo: &str,
    token: &str,
) -> Box<Future<Item = GraphQLReply, Error = failure::Error>> {
    let mut request = client::post("https://api.github.com/graphql");
    request.header(USER_AGENT, "actix");
    request.header(AUTHORIZATION, format!("bearer {}", token));
    let resp = request.body(format!(query!(), owner, repo)).unwrap().send();

    Box::new(
        resp.map_err(|err| err.into())
            .and_then(|resp| resp.json().limit(1_048_576).map_err(|err| err.into())),
    )
}

impl GraphQLReply {
    pub fn list(self) -> Vec<Body> {
        self.data
            .repository
            .issues
            .nodes
            .iter()
            .map(|issue| {
                iter::once(Body {
                    body: issue.body.clone(),
                    last_update: latest_date(issue.created_at, None, issue.last_edited_at),
                    url: issue.url.clone(),
                }).chain(issue.comments.nodes.iter().map(|comment| Body {
                    body: comment.body.clone(),
                    last_update: latest_date(
                        comment.created_at,
                        Some(comment.updated_at),
                        comment.last_edited_at,
                    ),
                    url: comment.url.clone(),
                }))
            }).flat_map(|i| i)
            .collect()
    }
}

fn latest_date(
    created: DateTime<Utc>,
    updated: Option<DateTime<Utc>>,
    edited: Option<DateTime<Utc>>,
) -> DateTime<Utc> {
    edited.unwrap_or_else(|| updated.unwrap_or(created))
}

#[derive(Serialize)]
pub struct Body {
    pub body: String,
    pub last_update: DateTime<Utc>,
    pub url: String,
}
