#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;

extern crate actix_web;
extern crate failure;
extern crate futures;
extern crate http;

#[macro_use]
extern crate lazy_static;

extern crate comrak;
extern crate linkify;

pub mod github;
pub mod markdown;
pub mod playground;
