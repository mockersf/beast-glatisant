use std::collections::HashMap;
use std::ops::Deref;
use std::sync::RwLock;

use failure;
use futures::future::Future;

lazy_static! {
    static ref GIST_CACHE: RwLock<HashMap<super::ETag, Gist>> = { RwLock::new(HashMap::new()) };
}

#[derive(Deserialize, Debug, Clone)]
pub struct File {
    pub filename: String,
    pub content: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Gist {
    pub url: String,
    pub id: String,
    pub node_id: String,

    pub html_url: String,

    pub files: HashMap<String, File>,
}

pub fn get_gist(
    gist_id: &str,
    token: Option<String>,
) -> Box<Future<Item = Gist, Error = failure::Error>> {
    super::get_object(
        &format!("https://api.github.com/gists/{}", gist_id),
        token,
        GIST_CACHE.deref(),
    )
}
