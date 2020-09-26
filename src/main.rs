pub mod chkapr;

use clap::{crate_authors, crate_description, crate_name, crate_version, ArgSettings, Clap};

use crate::chkapr::github;

#[derive(Clap, Debug)]
#[clap(
    name = crate_name!(),
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!()
)]
struct Opts {
    /// github token
    #[clap(
        long,
        env = "GITHUB_TOKEN",
        setting = ArgSettings::HideEnvValues,
    )]
    github_token: String,

    /// target tag and label
    #[clap(long, env = "TARGET")]
    target: String,

    /// target repository name
    #[clap(long, env = "TARGET_REPO")]
    repository: String,

    /// base ref name
    #[clap(long, env = "BASE_REF", default_value = "pay2release")]
    base_ref: String,

    /// head ref name
    #[clap(long, env = "HEAD_REF", default_value = "master")]
    head_ref: String,

    /// organization name
    #[clap(long, env = "ORGANIZATION", default_value = "Pay-Baymax")]
    organization: String,

    /// an approvable team
    #[clap(long, env = "APPROVABLE_TEAM", default_value = "tech-leads")]
    approvable_team: String,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let response = github::query(
        opts.target,
        opts.repository,
        opts.github_token,
        opts.organization,
        opts.approvable_team,
        opts.base_ref,
        opts.head_ref,
    )
    .await;

    println!("{:?}", response);
}
