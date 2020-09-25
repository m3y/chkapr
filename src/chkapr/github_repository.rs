use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// data
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubResponse {
    data: GithubData,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubData {
    repository: GithubRepository,
}

/// repository
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubRepository {
    name: String,
    pull_requests: HashMap<String, Vec<GithubPullRequests>>,
    release: GithubRelease,
}

/// pull requests
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubPullRequests {
    commits: HashMap<String, Vec<GithubCommits>>,
    labels: HashMap<String, Vec<GithubLabels>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubCommits {
    commit: GithubCommit,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubCommit {
    oid: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubLabels {
    name: String,
}

/// release
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubRelease {
    tag: GithubTag,
    tag_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubTag {
    target: GithubTarget,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubTarget {
    oid: String,
    parents: HashMap<String, Vec<GithubParents>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GithubParents {
    authored_by_committer: bool,
    oid: String,
}

pub fn query(
    target: String,
    repository: String,
    github_token: String,
    organization: String,
    approvable_team: String,
    base_ref: String,
    head_ref: String,
) -> Result<GithubResponse> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("rust reqwest")
        .build()?;
    let query = r#"
        query ($owner: String = "Pay-Baymax", $team: String = "tech-leads", $base: String = "pay2release", $head: String = "master", $name: String!, $tagName: String!) {
          repository(name: $name, owner: $owner) {
            name
            pullRequests(first: 5, baseRefName: $base, headRefName: $head, orderBy: {field: CREATED_AT, direction: DESC}) {
              nodes {
                number
                commits(last: 10) {
                  nodes {
                    commit {
                      oid
                    }
                  }
                }
                labels(last: 10) {
                  nodes {
                    name
                  }
                }
                reviews(states: APPROVED, last: 10) {
                  nodes {
                    author {
                      login
                      ... on User {
                        organization(login: $owner) {
                          team(slug: $team) {
                            slug
                            members {
                              nodes {
                                login
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
            release(tagName: $tagName) {
              tagName
              tag {
                target {
                  oid
                  ... on Commit {
                    oid
                    parents(last: 2) {
                      nodes {
                          authoredByCommitter
                          oid
                      }
                    }
                  }
                }
              }
            }
          }
        }"#;

    let request = json!({
        "query": query,
        "variables": {
            "name": repository,
            "owner": organization,
            "tagName": target,
            "team": approvable_team,
            "base": base_ref,
            "head": head_ref,
        }
    });

    let resp = client
        .post("https://api.github.com/graphql")
        .bearer_auth(github_token)
        .json(&request)
        .send()?;

    resp.json::<GithubResponse>().context("error")
}
