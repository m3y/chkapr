use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Repository {
    name: String,
    pull_requests: PullRequestNodes,
    release: Option<Release>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PullRequestNodes {
    nodes: Option<Vec<PullRequest>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    number: i32,
    commits: CommitNodes,
    labels: LabelNodes,
    reviews: ReviewNodes,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommitNodes {
    nodes: Vec<Commits>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Commits {
    commit: Commit,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Commit {
    oid: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LabelNodes {
    nodes: Option<Vec<Label>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Label {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewNodes {
    nodes: Option<Vec<Review>>,
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
    members: MemberNodes,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MemberNodes {
    nodes: Vec<Member>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Member {
    login: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub tag_name: String,
    tag: Tag,
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
    parents: ParentNodes,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParentNodes {
    nodes: Option<Vec<Parent>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Parent {
    authored_by_committer: bool,
    oid: String,
}

impl Response {
    pub fn get_pull_requests(&self) -> Option<&Vec<PullRequest>> {
        self.data.repository.pull_requests.nodes.as_ref()
    }

    pub fn get_release(&self) -> Option<&Release> {
        self.data.repository.release.as_ref()
    }
}

impl PullRequest {
    pub fn is_valid(&self) -> bool {
        self.commits.nodes.len() > 0
    }

    pub fn to_message(&self) -> String {
        format!("Pull Requests (#{})", self.number)
    }

    pub fn has_commit(&self, commit_hash: String) -> bool {
        self.commits
            .nodes
            .iter()
            .map(|c| &c.commit)
            .any(|c| c.oid == commit_hash)
    }

    pub fn has_label(&self, label: String) -> bool {
        match &self.labels.nodes {
            Some(v) => v.iter().any(|l| l.name == label),
            None => false,
        }
    }

    pub fn is_approved(&self) -> bool {
        match &self.reviews.nodes {
            Some(v) => v.iter().any(|r| r.is_approved()),
            None => false,
        }
    }
}

impl Team {
    fn has_member(&self, login: String) -> bool {
        self.members.nodes.iter().any(|m| m.login == login)
    }
}

impl Review {
    fn is_approved(&self) -> bool {
        let login = &self.author.login;
        if let Some(approvable_team) = &self.author.organization.team {
            approvable_team.has_member(login.into())
        } else {
            false
        }
    }
}

impl Release {
    pub fn is_valid(&self) -> bool {
        self.tag_name != "" && self.tag.target.oid != ""
    }

    pub fn get_tag_name(&self) -> &String {
        &self.tag_name
    }

    pub fn get_oid(&self) -> &String {
        &self.tag.target.oid
    }

    pub fn get_parent_oid(&self) -> Option<String> {
        match &self.tag.target.parents.nodes {
            Some(parents) => {
                let oids = parents
                    .iter()
                    .filter(|p| p.authored_by_committer)
                    .map(|p| &p.oid)
                    .collect::<Vec<_>>();
                Some(oids[0].to_string())
            }
            None => None,
        }
    }

    pub fn to_message(&self) -> String {
        if self.is_valid() {
            return format!("{}({})", self.tag_name, self.tag.target.oid);
        }

        format!(
            "The structure of release is not correct. [name: {}]",
            self.tag_name
        )
    }
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

    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;

    #[test]
    fn test_release_is_valid_from_response() {
        let response = Response::from_jsonfile(PathBuf::from("tests/fixtures/test_data.json"));
        match response.get_release() {
            Some(release) => assert_eq!(true, release.is_valid()),
            None => assert!(false),
        }
    }

    #[test]
    fn test_release_to_message() {
        let response = Response::from_jsonfile(PathBuf::from("tests/fixtures/test_data.json"));
        match response.get_release() {
            Some(release) => assert_eq!(
                "canary_release(46f663b32c01d20ce14f58b5d81ac0f813c4b691)",
                release.to_message()
            ),
            None => assert!(false),
        }
    }

    #[test]
    fn test_pull_requests_is_valid() {
        let response = Response::from_jsonfile(PathBuf::from("tests/fixtures/test_data.json"));
        let pull_request = response.get_pull_requests().unwrap().get(0);

        assert_eq!(true, pull_request.unwrap().is_valid());
    }

    #[test]
    fn test_pull_requests_has_commit() {
        let response = Response::from_jsonfile(PathBuf::from("tests/fixtures/test_data.json"));
        let pull_request = response.get_pull_requests().unwrap().get(0);

        assert_eq!(
            true,
            pull_request
                .unwrap()
                .has_commit("9ff12322cd86e6dc1f254209c04f4dde40876588".to_string())
        );

        assert_eq!(
            false,
            pull_request
                .unwrap()
                .has_commit("c04f4dde408765889ff12322cd86e6dc1f254209".to_string())
        );
    }

    #[test]
    fn test_pull_requests_has_label() {
        let response = Response::from_jsonfile(PathBuf::from("tests/fixtures/test_data.json"));
        let pull_request = response.get_pull_requests().unwrap().get(0);

        assert_eq!(
            true,
            pull_request
                .unwrap()
                .has_label("canary_release".to_string())
        );

        assert_eq!(
            false,
            pull_request
                .unwrap()
                .has_label("canary_rollback".to_string())
        );
    }

    #[test]
    //fn test_team_has_member() {
    //    let response = Response::from_jsonfile(PathBuf::from("tests/fixtures/test_data.json"));
    //    let pull_request = response.get_pull_requests().unwrap().get(0);
    //    let reviews = pull_request.unwrap().reviews.nodes.unwrap();
    //    let review = reviews.get(0).unwrap();
    //    //let team = review.author.organization.team.as_ref();

    //    assert_eq!(
    //        true,
    //        review
    //            .author
    //            .organization
    //            .team
    //            .unwrap()
    //            .has_member("paypay-ci".to_string())
    //    );
    //    assert_eq!(
    //        false,
    //        review
    //            .author
    //            .organization
    //            .team
    //            .unwrap()
    //            .has_member("m3y".to_string())
    //    );
    //}
    //#[test]
    //fn test_review_is_approved() {
    //    let response = Response::from_jsonfile(PathBuf::from("tests/fixtures/test_data.json"));
    //    let pull_request = response.get_pull_requests().unwrap().get(0);
    //    let reviews = pull_request.unwrap().reviews.nodes.unwrap();
    //    let review = reviews.get(0).unwrap();

    //    assert_eq!(true, review.is_approved());
    //}
    #[test]
    fn test_pull_requests_is_approved() {
        let response = Response::from_jsonfile(PathBuf::from("tests/fixtures/test_data.json"));
        let pull_request = response.get_pull_requests().unwrap().get(0);

        assert_eq!(true, pull_request.unwrap().is_approved());
    }

    impl Response {
        fn from_jsonfile(path: PathBuf) -> Response {
            let f = File::open(path).unwrap();
            let reader = BufReader::new(f);
            serde_json::from_reader(reader).unwrap()
        }
    }
}
