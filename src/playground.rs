use actix_web::{client, HttpMessage};
use failure;
use futures::future::Future;
use std::time::Duration;

#[derive(Serialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Run,
    Test,
    Format,
    Clippy,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Channel {
    Stable,
    Beta,
    Nightly,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Debug,
    Release,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CrateType {
    Bin,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Query {
    code: String,
    channel: Channel,
    mode: Mode,
    crate_type: CrateType,
    test: bool,
}
impl Query {
    fn from(action: Action, code: String) -> Self {
        Query {
            code,
            channel: Channel::Stable,
            mode: Mode::Debug,
            test: action == Action::Test,
            crate_type: CrateType::Bin,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Response {
    success: bool,
    stdout: String,
    stderr: String,
    code: Option<String>,
}

pub fn ask_playground(
    code: &str,
    action: Action,
) -> impl Future<Item = Response, Error = failure::Error> {
    debug!("calling playground for {:?}", action);
    client::post(match action {
        Action::Run => "https://play.rust-lang.org/execute",
        Action::Test => "https://play.rust-lang.org/execute",
        Action::Clippy => "https://play.rust-lang.org/clippy",
        Action::Format => "https://play.rust-lang.org/format",
    }).timeout(Duration::new(30, 0))
        .json(&Query::from(action, wrap_in_main_if_not_present(code)))
        .unwrap()
        .send()
        .map_err(|err| err.into())
        .and_then(|resp| resp.json().map_err(|err| err.into()))
}

fn wrap_in_main_if_not_present(code: &str) -> String {
    if code.contains("fn main()") {
        code.to_string()
    } else {
        ["fn main() {", code, "}"].concat()
    }
}

pub fn ask_playground_simpl(
    code: &str,
    action: Action,
) -> impl Future<Item = String, Error = failure::Error> {
    ask_playground(code, action).map(move |playground| match (action, playground) {
        (
            _,
            Response {
                success: false,
                stderr,
                ..
            },
        ) => stderr,
        (Action::Clippy, Response { stderr, .. }) => stderr
            .split('\n')
            .skip(1)
            .take_while(|line| !line.contains("Finished dev"))
            .map(|line| format!("{}\n", line))
            .collect::<Vec<String>>()
            .concat(),
        (
            Action::Format,
            Response {
                code: Some(code), ..
            },
        ) => code,
        (_, Response { stdout, .. }) => stdout,
    })
}
