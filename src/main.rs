pub mod chkapr;

use clap::{crate_authors, crate_description, crate_name, crate_version, ArgSettings, Clap};

use crate::chkapr::github_repository;

#[derive(Clap, Debug)]
#[clap(
    name = crate_name!(),
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!()
)]
struct Opts {
    #[clap(
        long,
        env = "GITHUB_TOKEN",
        setting = ArgSettings::HideEnvValues,
        about = "github token"
    )]
    github_token: String,
    #[clap(long, env = "TARGET", about = "target tag and label.")]
    target: String,
    #[clap(long, env = "TARGET_REPO", about = "target repository name.")]
    repository: String,
    #[clap(
        long,
        env = "BASE_REF",
        default_value = "pay2release",
        about = "base ref name."
    )]
    base_ref: String,
    #[clap(
        long,
        env = "HEAD_REF",
        default_value = "master",
        about = "head ref name."
    )]
    head_ref: String,
    #[clap(
        long,
        env = "ORGANIZATION",
        default_value = "Pay-Baymax",
        about = "organization name."
    )]
    organization: String,
    #[clap(
        long,
        env = "APPROVABLE_TEAM",
        default_value = "tech-leads",
        about = "an approvable team."
    )]
    approvable_team: String,
}

fn main() {
    let opts = Opts::parse();

    let result = github_repository::query(
        opts.target,
        opts.repository,
        opts.github_token,
        opts.organization,
        opts.approvable_team,
        opts.base_ref,
        opts.head_ref,
    );

    println!("{:?}", result);
}
