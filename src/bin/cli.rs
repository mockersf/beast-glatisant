extern crate beast_glatisant;

#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
enum CommandOpt {
    /// list issues of a repo
    #[structopt(name = "list")]
    ListIssues,

    /// get issue of a repo by #
    #[structopt(name = "issue")]
    GetIssue {
        /// Issue number
        issue: u32,
    },

    /// get code blocks of an issue
    #[structopt(name = "codes")]
    GetCodeBlock {
        /// Issue number
        issue: u32,
    },

    /// run clippy on code blocks of an issue
    #[structopt(name = "clippy")]
    RunClippy {
        /// Issue number
        issue: u32,
    },
}

#[derive(StructOpt, Debug)]
#[structopt(name = "beast_glatisant", author = "")]
struct Config {
    /// GitHub repo owner
    pub owner: String,
    /// GitHub repo
    pub repo: String,
    /// GitHub token
    #[structopt(long = "token")]
    pub token: Option<String>,

    #[structopt(flatten)]
    command: CommandOpt,
}

fn main() {
    let config = Config::from_args();

    let token = config.token.clone();

    match config.command {
        CommandOpt::ListIssues => {
            beast_glatisant::github::issue::get_issues(&config.owner, &config.repo, config.token)
                .filter(|issue| issue.pull_request.is_none())
                .for_each(|issue| println!("{:#?}", issue))
        }

        CommandOpt::GetIssue { issue } => beast_glatisant::github::issue::get_issue(
            &config.owner,
            &config.repo,
            issue,
            config.token,
        ).iter()
            .for_each(|issue| println!("{:#?}", issue)),

        CommandOpt::GetCodeBlock { issue } => beast_glatisant::github::issue::get_issue(
            &config.owner,
            &config.repo,
            issue,
            config.token.clone(),
        ).iter()
            .map(|issue| &issue.body)
            .chain(
                beast_glatisant::github::issue::get_comments(
                    &config.owner,
                    &config.repo,
                    issue,
                    config.token.clone(),
                ).expect("getting comments of issue")
                    .iter()
                    .map(|comment| &comment.body),
            )
            .for_each(|body| {
                beast_glatisant::markdown::get_code_samples(&body, token.clone())
                    .iter()
                    .for_each(|code_block| {
                        println!("{}\n==============================", code_block.code.trim())
                    })
            }),

        CommandOpt::RunClippy { issue } => beast_glatisant::github::issue::get_issue(
            &config.owner,
            &config.repo,
            issue,
            config.token.clone(),
        ).iter()
            .map(|issue| &issue.body)
            .chain(
                beast_glatisant::github::issue::get_comments(
                    &config.owner,
                    &config.repo,
                    issue,
                    config.token.clone(),
                ).expect("getting comments of issue")
                    .iter()
                    .map(|comment| &comment.body),
            )
            .for_each(|body| {
                beast_glatisant::markdown::get_code_samples(&body, token.clone())
                    .iter()
                    .for_each(|code_block| {
                        println!(
                            "{}\n{}\n==============================",
                            code_block.code.trim(),
                            if is_rust(&code_block.code) {
                                beast_glatisant::playground::ask_playground_simpl(
                                    &code_block.code,
                                    beast_glatisant::playground::Action::Clippy,
                                ).trim()
                                    .to_string()
                            } else {
                                "not rust code".to_string()
                            },
                        )
                    })
            }),
    }
}

fn is_rust(code: &str) -> bool {
    !code.contains("for further information visit https://rust-lang-nursery.github.io/rust-clippy")
}
