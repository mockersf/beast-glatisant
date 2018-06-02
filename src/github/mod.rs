use std::collections::HashMap;
use std::sync::RwLock;

use reqwest;
use reqwest::header::{Authorization, Bearer, EntityTag, Headers, IfNoneMatch};
use reqwest::StatusCode;
use serde::Deserialize;

pub mod gist;
pub mod issue;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct ETag(String);

lazy_static! {
    static ref URL_TO_ETAG_CACHE: RwLock<HashMap<String, ETag>> = { RwLock::new(HashMap::new()) };
}

pub fn get_object<T>(
    url: &str,
    token: Option<String>,
    etag_cache: &RwLock<HashMap<ETag, T>>,
) -> Option<T>
where
    T: Clone,
    for<'de> T: Deserialize<'de>,
{
    let client = ::CLIENT.lock().unwrap();

    let cached_etag = URL_TO_ETAG_CACHE.read().unwrap().get(url).cloned();

    let mut headers = Headers::new();
    if let Some(ref etag) = cached_etag {
        headers.set(IfNoneMatch::Items(vec![EntityTag::new(
            false,
            etag.clone().0,
        )]));
    }
    if let Some(token) = token {
        headers.set(Authorization(Bearer { token }));
    }

    let mut resp = client
        .get(url)
        .headers(headers)
        .send()
        .expect("able to query github");

    match (cached_etag, resp.status()) {
        (Some(etag), StatusCode::NotModified) => {
            let issue_cache = etag_cache.read().unwrap();
            issue_cache.get(&etag.clone()).cloned()
        }
        (Some(old_etag), _) => {
            etag_cache.write().unwrap().remove(&old_etag);
            let new_etag = resp.headers()
                .get::<reqwest::header::ETag>()
                .map(|etag| ETag(etag.tag().to_string()));
            resp.json().ok().map(|object: T| {
                if let Some(ref etag) = new_etag {
                    URL_TO_ETAG_CACHE
                        .write()
                        .unwrap()
                        .insert(url.to_string(), etag.clone());
                    etag_cache
                        .write()
                        .unwrap()
                        .insert(etag.clone(), object.clone());
                }
                object
            })
        }
        (None, _) => {
            let new_etag = resp.headers()
                .get::<reqwest::header::ETag>()
                .map(|etag| ETag(etag.tag().to_string()));
            resp.json().ok().map(|object: T| {
                if let Some(ref etag) = new_etag {
                    URL_TO_ETAG_CACHE
                        .write()
                        .unwrap()
                        .insert(url.to_string(), etag.clone());
                    etag_cache
                        .write()
                        .unwrap()
                        .insert(etag.clone(), object.clone());
                }
                object
            })
        }
    }
}
