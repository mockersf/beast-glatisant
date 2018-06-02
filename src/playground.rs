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
    fn from(action: &Action, code: &str) -> Self {
        Query {
            code: code.to_string(),
            channel: Channel::Stable,
            mode: Mode::Debug,
            test: *action == Action::Test,
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

pub fn ask_playground(code: &str, action: Action) -> Response {
    let client = ::CLIENT.lock().unwrap();

    client
        .post(match action {
            Action::Run => "https://play.rust-lang.org/execute",
            Action::Test => "https://play.rust-lang.org/execute",
            Action::Clippy => "https://play.rust-lang.org/clippy",
            Action::Format => "https://play.rust-lang.org/format",
        })
        .json(&Query::from(&action, code))
        .send()
        .expect("able to query playground")
        .json()
        .expect("playground response is not an error")
}

pub fn ask_playground_simpl(code: &str, action: Action) -> String {
    match (action, ask_playground(code, action)) {
        (
            _,
            Response {
                success: false,
                stderr,
                ..
            },
        ) => stderr,
        (Action::Clippy, Response { stderr, .. }) => {
            // let first_line = stderr.find('\n').map(|n| n + 1).unwrap_or(0);
            // stderr.split_off(first_line)
            stderr
                .split('\n')
                .skip(1)
                .take_while(|line| !line.contains("Finished dev"))
                .map(|line| format!("{}\n", line))
                .collect::<Vec<String>>()
                .concat()
        }
        (
            Action::Format,
            Response {
                code: Some(code), ..
            },
        ) => code,
        (_, Response { stdout, .. }) => stdout,
    }
}
