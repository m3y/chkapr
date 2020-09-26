use anyhow::Result;
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
    pull_requests: HashMap<String, Option<Vec<PullRequest>>>,
    release: Option<Release>,
}

/// pull requests
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PullRequest {
    number: i32,
    commits: HashMap<String, Vec<HashMap<String, Commit>>>,
    labels: HashMap<String, Option<Vec<Label>>>,
    reviews: HashMap<String, Option<Vec<Review>>>,
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
    team: Option<Team>,
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
    tag_name: String,
    tag: Tag,
}

impl Release {
    fn is_valid(&self) -> bool {
        self.tag_name != "" && self.tag.target.oid != ""
    }

    fn to_string(&self) -> String {
        if self.is_valid() {
            return format!("{}({})", self.tag_name, self.tag.target.oid);
        }

        format!(
            "The structure of release is not correct. [name: {}]",
            self.tag_name
        )
    }
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
    parents: HashMap<String, Option<Vec<Parent>>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Parent {
    authored_by_committer: bool,
    oid: String,
}

/// query
pub async fn query(
    target: String,
    repository: String,
    github_token: String,
    organization: String,
    approvable_team: String,
    base_ref: String,
    head_ref: String,
) -> Result<Response> {
    let client = reqwest::Client::builder()
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

    Ok(client
        .post("https://api.github.com/graphql")
        .bearer_auth(github_token)
        .json(&request)
        .send()
        .await?
        .json::<Response>()
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_is_valid() {
        assert_eq!(false, Release::new("", "", "", false).is_valid());
        assert_eq!(
            false,
            Release::new("canary_release", "", "", false).is_valid()
        );
        assert_eq!(
            false,
            Release::new("", "xxxxyyyyzzzz", "", false).is_valid()
        );
        assert_eq!(
            true,
            Release::new("canary_release", "xxxxyyyyzzzz", "", false).is_valid()
        );
    }

    #[test]
    fn test_release_to_string() {
        assert_eq!(
            "canary_release(xxxxxyyyyyzzzzz)",
            Release::new("canary_release", "xxxxxyyyyyzzzzz", "", false).to_string()
        );
        assert_eq!(
            "The structure of release is not correct. [name: ]",
            Release::new("", "", "", false).to_string()
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            "sre-test-k8s",
            Response::from_jsonfile(PathBuf::from("tests/fixtures/test_data.json"))
                .data
                .repository
                .name
        );
    }

    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;

    impl Response {
        fn from_jsonfile(path: PathBuf) -> Response {
            let f = File::open(path).unwrap();
            let reader = BufReader::new(f);
            serde_json::from_reader(reader).unwrap()
        }
    }

    // for test
    impl Release {
        fn new(
            tag_name: &str,
            oid: &str,
            parent_oid: &str,
            authored_by_committer: bool,
        ) -> Release {
            let mut parents = HashMap::new();
            parents.insert(
                "nodes".to_string(),
                Some(vec![Parent {
                    oid: parent_oid.to_string(),
                    authored_by_committer: authored_by_committer,
                }]),
            );
            Release {
                tag_name: tag_name.to_string(),
                tag: Tag {
                    target: Target {
                        oid: oid.to_string(),
                        parents: parents,
                    },
                },
            }
        }
    }
}
