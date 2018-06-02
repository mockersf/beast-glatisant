use std::collections::HashMap;
use std::ops::Deref;
use std::sync::RwLock;

use failure;
use futures::future::Future;

lazy_static! {
    static ref ISSUE_CACHE: RwLock<HashMap<super::ETag, Issue>> = { RwLock::new(HashMap::new()) };
}
lazy_static! {
    static ref COMMENTS_CACHE: RwLock<HashMap<super::ETag, Vec<Comment>>> =
        { RwLock::new(HashMap::new()) };
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Open,
    Closed,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PullRequest {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Issue {
    pub url: String,
    pub id: u32,
    pub node_id: String,
    pub number: u32,

    pub html_url: String,

    pub comments_url: String,
    pub comments: u32,

    pub created_at: String,
    pub updated_at: String,

    pub state: State,
    pub title: String,
    pub body: String,
    pub pull_request: Option<PullRequest>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Comment {
    pub url: String,
    pub id: u32,
    pub node_id: String,

    pub html_url: String,

    pub issue_url: String,

    pub created_at: String,
    pub updated_at: String,

    pub body: String,
}

pub fn get_comments(
    owner: &str,
    repo: &str,
    issue_number: u32,
    token: Option<String>,
) -> Box<Future<Item = Vec<Comment>, Error = failure::Error>> {
    super::get_object(
        &format!(
            "https://api.github.com/repos/{}/{}/issues/{}/comments",
            owner, repo, issue_number
        ),
        token,
        COMMENTS_CACHE.deref(),
    )
}

pub fn get_issue(
    owner: &str,
    repo: &str,
    issue_number: u32,
    token: Option<String>,
) -> Box<Future<Item = Issue, Error = failure::Error>> {
    super::get_object(
        &format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            owner, repo, issue_number
        ),
        token,
        ISSUE_CACHE.deref(),
    )
}
