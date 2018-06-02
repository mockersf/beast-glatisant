#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate reqwest;

#[macro_use]
extern crate lazy_static;

extern crate comrak;
extern crate linkify;

use std::sync::Mutex;

pub mod github;
pub mod markdown;
pub mod playground;

lazy_static! {
    static ref CLIENT: Mutex<reqwest::Client> = {
        Mutex::new(
            reqwest::Client::builder()
                .build()
                .expect("able to build reqwest client"),
        )
    };
}
