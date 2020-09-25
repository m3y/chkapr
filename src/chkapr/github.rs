use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// data
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    data: Data,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    repository: Repository,
}

/// repository
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Repository {
    name: String,
    pull_requests: HashMap<String, Vec<PullRequest>>,
    release: Release,
}

/// pull requests
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PullRequest {
    number: i32,
    commits: HashMap<String, Vec<HashMap<String, Commit>>>,
    labels: HashMap<String, Vec<Label>>,
    reviews: HashMap<String, Vec<Review>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Commit {
    oid: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Label {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Review {
    author: Author,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Author {
    login: String,
    organization: Organization,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Organization {
    team: Team,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Team {
    slug: String,
    members: HashMap<String, Vec<Member>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Member {
    login: String,
}

/// release
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Release {
    tag: Tag,
    tag_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Tag {
    target: Target,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Target {
    oid: String,
    parents: HashMap<String, Vec<Parent>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Parent {
    authored_by_committer: bool,
    oid: String,
}

/// query
pub fn query(
    target: String,
    repository: String,
    github_token: String,
    organization: String,
    approvable_team: String,
    base_ref: String,
    head_ref: String,
) -> Result<Response> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("rust reqwest")
        .build()?;
    let query = r#"
        query ($owner: String!, $team: String!, $base: String!, $head: String!, $name: String!, $tagName: String!) {
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

    resp.json::<Response>().context("error")
}
