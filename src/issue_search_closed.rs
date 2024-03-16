use chrono::{Duration, NaiveDate, Utc};

use anyhow::anyhow;
use octocrab::{models::issues::Issue, Octocrab};
use std::env;

use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OuterIssue {
    pub title: String,
    pub url: String,
    pub author: String,
    pub body: String,
    pub repository: String,
    pub repository_stars: i64,
    pub issue_labels: Vec<String>,
    pub comments: Vec<String>,             // Concat of author and comment
    pub cross_referenced_prs: Vec<String>, // URLs of cross-referenced pull requests
}

pub async fn get_issues(query: &str) -> anyhow::Result<Vec<OuterIssue>> {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct GraphQLResponse {
        data: Option<Data>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Data {
        search: Option<Search>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Search {
        issueCount: Option<i32>,
        edges: Option<Vec<Edge>>,
        pageInfo: PageInfo,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: bool,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Edge {
        node: Option<LocalIssue>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct LocalIssue {
        title: Option<String>,
        url: Option<String>,
        author: Option<Author>,
        body: Option<String>,
        repository: Option<Repository>,
        labels: Option<Labels>,
        comments: Option<Comments>,
        timelineItems: Option<TimelineItems>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Author {
        login: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Repository {
        url: Option<String>,
        stargazers: Option<Stargazers>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Stargazers {
        totalCount: Option<i64>,
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
    struct Comments {
        edges: Option<Vec<CommentEdge>>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct CommentEdge {
        node: Option<Comment>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Comment {
        author: Option<Author>,
        body: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineItems {
        edges: Option<Vec<TimelineEdge>>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineEdge {
        node: Option<CrossReferencedEvent>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct CrossReferencedEvent {
        source: Option<Source>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Source {
        __typename: Option<String>,
        url: Option<String>,
    }

    let first_comments = 10;
    let first_timeline_items = 10;
    let mut all_issues = Vec::new();
    let mut after_cursor: Option<String> = None;
    let file_path = "issues.txt";
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    let mut count = 0;

    for _ in 0..10 {
        let query_str = format!(
            r#"
            query {{
                search(query: "{}", type: ISSUE, first: 10, after: {}) {{
                    issueCount
                    edges {{
                        node {{
                            ... on Issue {{
                                title
                                url
                                body
                                author {{
                                    login
                                }}
                                repository {{
                                    url
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
                                timelineItems(first: {}, itemTypes: [CROSS_REFERENCED_EVENT]) {{
                                    edges {{
                                        node {{
                                            ... on CrossReferencedEvent {{
                                                source {{
                                                    __typename
                                                    ... on PullRequest {{
                                                        url
                                                    }}
                                                }}
                                            }}
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
            first_comments,
            first_timeline_items
        );

        let response_body = github_http_post_gql(&query_str)
            .await
            .map_err(|e| anyhow!("Failed to post GraphQL query: {}", e))?;

        let response: GraphQLResponse = serde_json::from_slice(&response_body)
            .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?;

        if let Some(data) = response.data {
            if let Some(search) = data.search {
                for edge in search.edges.unwrap_or_default() {
                    if let Some(issue) = edge.node {
                        let labels = issue.labels.map_or(Vec::new(), |labels| {
                            labels.edges.map_or(Vec::new(), |edges| {
                                edges
                                    .iter()
                                    .filter_map(|edge| {
                                        edge.node
                                            .as_ref()
                                            .map(|label| label.name.clone().unwrap_or_default())
                                    })
                                    .collect()
                            })
                        });
                        let temp_str = String::from("");
                        let comments = issue.comments.map_or(Vec::new(), |comments| {
                            comments.edges.map_or(Vec::new(), |edges| {
                                edges
                                    .iter()
                                    .filter_map(|edge| {
                                        edge.node.as_ref().map(|comment| {
                                            format!(
                                                "{}: {}",
                                                comment.author.as_ref().map_or("", |a| a
                                                    .login
                                                    .as_ref()
                                                    .unwrap_or(&temp_str)),
                                                comment.body.as_ref().unwrap_or(&"".to_string())
                                            )
                                        })
                                    })
                                    .collect()
                            })
                        });

                        let cross_referenced_prs =
                            issue.timelineItems.map_or(Vec::new(), |items| {
                                items.edges.map_or(Vec::new(), |edges| {
                                    edges
                                        .iter()
                                        .filter_map(|edge| {
                                            edge.node.as_ref().map(|item| {
                                                item.source.as_ref().map_or(
                                                    "".to_string(),
                                                    |source| {
                                                        source
                                                            .url
                                                            .as_ref()
                                                            .unwrap_or(&"".to_string())
                                                            .clone()
                                                    },
                                                )
                                            })
                                        })
                                        .collect()
                                })
                            });

                        all_issues.push(OuterIssue {
                            title: issue.title.unwrap_or_default(),
                            url: issue.url.unwrap_or_default(),
                            author: issue
                                .author
                                .map_or(String::new(), |author| author.login.unwrap_or_default()),
                            body: issue.body.unwrap_or_default(),
                            repository: issue
                                .repository
                                .clone() // Clone here
                                .map_or(String::new(), |repo| repo.url.unwrap_or_default()),
                            repository_stars: issue.repository.map_or(0, |repo| {
                                repo.stargazers
                                    .map_or(0, |stars| stars.totalCount.unwrap_or(0))
                            }),
                            issue_labels: labels,
                            comments: comments,
                            cross_referenced_prs: cross_referenced_prs,
                        });
                    }
                }
                if search.pageInfo.hasNextPage {
                    after_cursor = search.pageInfo.endCursor
                } else {
                    break;
                }
            }
        }
    }

    Ok(all_issues)
}
