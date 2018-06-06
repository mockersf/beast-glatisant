use std::collections::HashMap;
use std::sync::RwLock;

use actix_web::{client::{self, ClientResponse},
                HttpMessage};
use failure;
use futures::future::{self, Future};
use http::{header::{AUTHORIZATION, ETAG, IF_NONE_MATCH, USER_AGENT},
           StatusCode};
use serde::Deserialize;

pub mod gist;
pub mod graphql_issue_list;
pub mod issue;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct ETag(String);

lazy_static! {
    static ref URL_TO_ETAG_CACHE: RwLock<HashMap<String, ETag>> = { RwLock::new(HashMap::new()) };
}

pub fn get_object<T>(
    url: &str,
    token: Option<String>,
    etag_cache: &'static RwLock<HashMap<ETag, T>>,
) -> Box<Future<Item = T, Error = failure::Error>>
where
    T: Clone,
    for<'de> T: Deserialize<'de>,
{
    let cached_etag = URL_TO_ETAG_CACHE.read().unwrap().get(url).cloned();

    let mut request = client::get(url);
    request.header(USER_AGENT, "actix");
    if let Some(token) = token {
        request.header(AUTHORIZATION, format!("bearer {}", token));
    }
    if let Some(ref etag) = cached_etag {
        request.header(IF_NONE_MATCH, etag.0.clone());
    }
    let resp = request.finish().unwrap().send();
    let key = url.to_string();
    Box::new(resp.map_err(move |err| err.into()).and_then(move |resp| {
        match (cached_etag, resp.status()) {
            (Some(etag), StatusCode::NOT_MODIFIED) => {
                debug!("retrieved {} from cache", key);
                get_from_cache(etag, etag_cache)
            }
            (Some(old_etag), _) => {
                debug!("updating {} in cache", key);
                etag_cache.write().unwrap().remove(&old_etag);
                add_to_cache_and_return(key, resp, etag_cache)
            }
            (None, _) => {
                debug!("adding {} to cache", key);
                add_to_cache_and_return(key, resp, etag_cache)
            }
        }
    }))
}

fn get_from_cache<T>(
    etag: ETag,
    cache: &RwLock<HashMap<ETag, T>>,
) -> Box<Future<Item = T, Error = failure::Error>>
where
    T: Clone,
    T: 'static,
{
    let cache_reader = cache.read().unwrap();
    match cache_reader.get(&etag.clone()).cloned() {
        Some(v) => Box::new(future::ok(v)),
        None => unreachable!(),
    }
}

fn add_to_cache_and_return<T>(
    key: String,
    resp: ClientResponse,
    cache: &'static RwLock<HashMap<ETag, T>>,
) -> Box<Future<Item = T, Error = failure::Error>>
where
    T: Clone,
    for<'de> T: Deserialize<'de>,
    T: 'static,
{
    let new_etag = resp.headers()
        .get(ETAG)
        .map(|etag| ETag(etag.to_str().unwrap().to_string()));
    Box::new(
        resp.json()
            .map(move |object: T| {
                if let Some(ref etag) = new_etag {
                    URL_TO_ETAG_CACHE.write().unwrap().insert(key, etag.clone());
                    cache.write().unwrap().insert(etag.clone(), object.clone());
                }
                object
            })
            .map_err(|err| err.into()),
    )
}
