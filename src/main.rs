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

    match &response {
        Err(e) => eprintln!("{:#?}", e),
        Ok(resp) => {
            let r = resp.get_release();
            if !r.map_or(false, |r| r.is_valid()) {
                eprintln!("release error");
                return ();
            }
            let release = r.unwrap();
            println!("{}", release.to_message());

            let pull_requests = resp.get_pull_requests();
            if pull_requests.is_none() {
                eprintln!("pr error");
                return ();
            }

            pull_requests
                .unwrap()
                .iter()
                .filter(|pr| pr.is_valid())
                .filter(|pr| pr.has_label(release.get_tag_name().into()))
                .filter(|pr| {
                    pr.has_commit(release.get_oid().into())
                        || release.get_parent_oid().map_or(false, |o| pr.has_commit(o))
                })
                .filter(|pr| pr.is_approved())
                .for_each(|pr| println!("Approval: {}", pr.to_message()));
        }
    }
}
