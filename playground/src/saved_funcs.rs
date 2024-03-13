use chrono::Utc;

use anyhow::anyhow;
use octocrab::{models::issues::Issue, Octocrab};
use std::env;

use chrono::Duration;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OuterIssue {
    pub title: String,
    pub number: i32,
    pub author: String,
    pub body: String, // newly added field
    pub repository: String,
    pub url: String,
    pub labels: Vec<String>,
    pub comments: String,
}

pub async fn get_issues(query: &str) -> anyhow::Result<Vec<OuterIssue>> {
    #[derive(Serialize, Deserialize, Debug)]
    struct GraphQLResponse {
        data: Option<Data>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Data {
        search: Option<Search>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Search {
        issueCount: Option<i32>,
        edges: Option<Vec<Edge>>,
        pageInfo: PageInfo,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: Option<bool>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Edge {
        node: Option<LocalIssue>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct LocalIssue {
        title: Option<String>,
        number: Option<i32>,
        author: Option<Author>,
        body: Option<String>,
        repository: Option<Repository>,
        url: Option<String>,
        labels: Option<Labels>,
        comments: Option<Comments>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Author {
        login: Option<String>,
        avatarUrl: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Repository {
        name: Option<String>,
        owner: Option<Owner>,
        stargazers: Option<Stargazers>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Stargazers {
        totalCount: Option<i32>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Owner {
        login: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Labels {
        edges: Option<Vec<LabelEdge>>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct LabelEdge {
        node: Option<Label>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Label {
        name: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Comments {
        edges: Option<Vec<CommentEdge>>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CommentEdge {
        node: Option<Comment>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Comment {
        author: Option<Author>,
        body: Option<String>,
    }
    let first_comments = 10;
    let mut all_issues = Vec::new();
    let mut after_cursor: Option<String> = None;
    let file_path = "issues.txt";
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    let mut count = 0;
    for _n in 1..55 {
        let query_str = format!(
            r#"
            query {{
                search(query: "{}", type: ISSUE, first: 100, after: {}) {{
                    issueCount
                    edges {{
                        node {{
                            ... on Issue {{
                                title
                                number
                                url
                                body
                                author {{
                                    login
                                    avatarUrl
                                }}
                                repository {{
                                    name
                                    owner {{
                                        login
                                    }}
                                    stargazers {{
                                        totalCount
                                    }}
                                }}
                                labels(first: 10) {{
                                    edges {{
                                        node {{
                                            name
                                        }}
                                    }}
                                }}
                                comments(first: {}) {{
                                    edges {{
                                        node {{
                                            author {{
                                                login
                                            }}
                                            body
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                    pageInfo {{
                        endCursor
                        hasNextPage
                    }}
                }}
            }}
            "#,
            query.replace("\"", "\\\""),
            after_cursor
                .as_ref()
                .map_or(String::from("null"), |c| format!("\"{}\"", c)),
            first_comments
        );

        let response_body = github_http_post_gql(&query_str)
            .await
            .map_err(|e| anyhow!("Failed to post GraphQL query: {}", e))?;

        let response: GraphQLResponse = serde_json::from_slice(&response_body)
            .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?;

        if let Some(data) = response.data {
            if let Some(search) = data.search {
                if let Some(edges) = search.edges {
                    for edge in edges {
                        if let Some(issue) = edge.node {
                            let labels = issue
                                .labels
                                .as_ref()
                                .and_then(|l| l.edges.as_ref())
                                .map_or(vec![], |edges| {
                                    edges
                                        .iter()
                                        .filter_map(|edge| {
                                            edge.node.as_ref().and_then(|label| label.name.clone())
                                        })
                                        .collect()
                                });

                            let comments = issue
                                .comments
                                .as_ref()
                                .and_then(|c| c.edges.as_ref())
                                .map_or(String::new(), |edges| {
                                    edges
                                        .iter()
                                        .filter_map(|edge| {
                                            edge.node.as_ref().map(|comment| {
                                                format!(
                                                    "{}: {}",
                                                    comment
                                                        .author
                                                        .as_ref()
                                                        .and_then(|a| a.login.as_ref())
                                                        .unwrap_or(&String::new()),
                                                    comment.body.as_ref().unwrap_or(&String::new())
                                                )
                                            })
                                        })
                                        .collect::<Vec<String>>()
                                        .join("\n")
                                });

                            let issue_url = issue.url.clone().unwrap_or_default();
                            writeln!(file, "{}", issue_url)?;
                            count += 1;
                            // println!(
                            //     "issue {count}: {:?}",
                            //     issue.title.clone().unwrap_or_default()
                            // );

                            all_issues.push(OuterIssue {
                                title: issue.title.clone().unwrap_or_default(),
                                number: issue.number.unwrap_or_default(),
                                author: issue
                                    .author
                                    .as_ref()
                                    .and_then(|a| a.login.clone())
                                    .unwrap_or_default(),
                                body: issue.body.clone().unwrap_or_default(),
                                repository: format!(
                                    "{}/{}",
                                    issue
                                        .repository
                                        .as_ref()
                                        .and_then(|r| r
                                            .owner
                                            .as_ref()
                                            .and_then(|o| o.login.as_ref()))
                                        .unwrap_or(&String::new()),
                                    issue
                                        .repository
                                        .as_ref()
                                        .and_then(|r| r.name.as_ref())
                                        .unwrap_or(&String::new())
                                ),
                                url: issue.url.clone().unwrap_or_default(),
                                labels,
                                comments,
                            });
                        }
                    }
                }

                match search.pageInfo.hasNextPage {
                    Some(true) => {
                        println!("{:?}", search.pageInfo.endCursor.clone());

                        after_cursor = search.pageInfo.endCursor
                    }
                    _ => break,
                }
            }
        }
    }

    Ok(all_issues)
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OuterPull {
    pub title: String,
    pub number: i32,
    pub author: String,
    pub labels: Vec<String>,
    pub reviews: Vec<String>, // Reviews by authors
    pub assignees: Vec<String>,
    pub timeline_comments: Vec<String>, // Comments from timeline items
}

pub async fn get_pull_request_details(query: &str) -> anyhow::Result<OuterPull> {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct GraphQLResponse {
        data: Option<Data>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Data {
        repository: Option<Repository>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Repository {
        pullRequest: Option<PullRequest>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct PullRequest {
        title: Option<String>,
        number: Option<i32>,
        author: Option<Author>,
        labels: Option<Labels>,
        reviews: Option<Reviews>,
        assignees: Option<Assignees>,
        timelineItems: Option<TimelineItems>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Author {
        login: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Labels {
        edges: Option<Vec<LabelEdge>>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct LabelEdge {
        node: Option<Label>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Label {
        name: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Reviews {
        edges: Option<Vec<ReviewEdge>>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct ReviewEdge {
        node: Option<Review>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Review {
        author: Option<Author>,
        state: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Assignees {
        edges: Option<Vec<AssigneeEdge>>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct AssigneeEdge {
        node: Option<Assignee>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Assignee {
        name: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineItems {
        edges: Option<Vec<TimelineEdge>>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineEdge {
        node: Option<TimelineItem>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "__typename")]
    enum TimelineItem {
        ClosedEvent {
            actor: Option<Author>,
            createdAt: Option<String>,
        },
        MergedEvent {
            actor: Option<Author>,
            createdAt: Option<String>,
        },
        IssueComment {
            author: Option<Author>,
            body: Option<String>,
            createdAt: Option<String>,
        },
        CrossReferencedEvent {
            actor: Option<Author>,
            source: Option<CrossReferencedSource>,
            createdAt: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "__typename")]
    enum CrossReferencedSource {
        Issue {
            number: Option<i32>,
            title: Option<String>,
            url: Option<String>,
        },
    }

    let response_body = github_http_post_gql(query)
        .await
        .map_err(|e| anyhow!("Failed to post GraphQL query: {}", e))?;

    let response: GraphQLResponse = serde_json::from_slice(&response_body)
        .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?;

    // Process the response and populate the OuterIssue struct
    if let Some(data) = response.data {
        if let Some(repo) = data.repository {
            if let Some(pr) = repo.pullRequest {
                // Process labels
                let labels =
                    pr.labels
                        .as_ref()
                        .and_then(|l| l.edges.as_ref())
                        .map_or(vec![], |edges| {
                            edges
                                .iter()
                                .filter_map(|edge| {
                                    edge.node.as_ref().and_then(|label| label.name.clone())
                                })
                                .collect()
                        });

                // Process reviews
                let reviews =
                    pr.reviews
                        .as_ref()
                        .and_then(|r| r.edges.as_ref())
                        .map_or(vec![], |edges| {
                            edges
                                .iter()
                                .filter_map(|edge| {
                                    edge.node.as_ref().map(|review| {
                                        format!(
                                            "{}: {}",
                                            review
                                                .author
                                                .as_ref()
                                                .and_then(|a| a.login.as_ref())
                                                .unwrap_or(&String::new()),
                                            review.state.as_ref().unwrap_or(&String::new())
                                        )
                                    })
                                })
                                .collect()
                        });

                // Add processing for assignees and timeline items similarly

                return Ok(OuterPull {
                    title: pr.title.unwrap_or_default(),
                    number: pr.number.unwrap_or(0),
                    author: pr
                        .author
                        .as_ref()
                        .and_then(|a| a.login.clone())
                        .unwrap_or_default(),
                    labels,
                    reviews,
                    // Populate assignees and timeline_comments based on your processing
                    assignees: vec![],
                    timeline_comments: vec![],
                });
            }
        }
    }

    Err(anyhow!("Failed to fetch pull request details"))
}
