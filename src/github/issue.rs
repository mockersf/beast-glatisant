use std::collections::HashMap;
use std::ops::Deref;
use std::sync::RwLock;

use reqwest;
use reqwest::header::{Authorization, Bearer, Headers};

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

pub struct IssueList {
    current_page: Vec<Issue>,
    current: usize,
    next_page: Option<String>,
    token: Option<String>,
    client: reqwest::Client,
}
impl Iterator for IssueList {
    type Item = Issue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.current_page.len() {
            self.current += 1;
            return Some(self.current_page[self.current - 1].clone());
        }
        if let Some(ref next_page) = self.next_page.clone() {
            let mut headers = Headers::new();
            if let Some(ref token) = self.token {
                headers.set(Authorization(Bearer {
                    token: token.clone(),
                }));
            }

            let mut resp = self.client
                .get(next_page)
                .headers(headers)
                .send()
                .expect("able to query github issues");

            if let Some(links) = resp.headers().get::<reqwest::header::Link>() {
                let next = links.values().iter().find(|link| {
                    if let Some(rels) = link.rel() {
                        rels.contains(&reqwest::header::RelationType::Next)
                    } else {
                        false
                    }
                });
                self.next_page = next.map(|link| link.link().to_string());
            }

            self.current_page = resp.json().expect("valid JSON for issues");
            self.current = 1;

            return Some(self.current_page[self.current - 1].clone());
        }
        None
    }
}

pub fn get_issues(owner: &str, repo: &str, token: Option<String>) -> IssueList {
    IssueList {
        current_page: vec![],
        current: 0,
        next_page: Some(format!(
            "https://api.github.com/repos/{}/{}/issues",
            owner, repo
        )),
        token,
        client: reqwest::Client::builder()
            .build()
            .expect("able to build reqwest client"),
    }
}

pub fn get_comments(
    owner: &str,
    repo: &str,
    issue_number: u32,
    token: Option<String>,
) -> Option<Vec<Comment>> {
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
) -> Option<Issue> {
    super::get_object(
        &format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            owner, repo, issue_number
        ),
        token,
        ISSUE_CACHE.deref(),
    )
}
