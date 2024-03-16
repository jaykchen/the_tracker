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
pub struct OuterPull {
    pub title: String,
    pub url: String,
    pub author: String,
    pub repository: String, // URL of the repository where the pull request was opened
    pub labels: Vec<String>,
    pub reviews: Vec<String>, // authors whose review state is approved
    pub merged_by: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GraphQLResponse {
    data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    search: Search,
}

#[derive(Serialize, Deserialize, Debug)]
struct Search {
    issueCount: i32,
    edges: Vec<Edge>,
    pageInfo: PageInfo,
}

#[derive(Serialize, Deserialize, Debug)]
struct PageInfo {
    endCursor: Option<String>,
    hasNextPage: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Edge {
    node: PullRequest,
}

#[derive(Serialize, Deserialize, Debug)]
struct PullRequest {
    title: String,
    url: String,
    repository: Repository,
    author: Author,
    labels: Labels,
    hasApprovedReview: Reviews,
    mergedBy: Author,
}

#[derive(Serialize, Deserialize, Debug)]
struct Repository {
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Author {
    login: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Labels {
    edges: Vec<LabelEdge>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LabelEdge {
    node: Label,
}

#[derive(Serialize, Deserialize, Debug)]
struct Label {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Reviews {
    edges: Vec<ReviewEdge>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReviewEdge {
    node: Review,
}

#[derive(Serialize, Deserialize, Debug)]
struct Review {
    author: Author,
}

pub async fn overall_search_pull_requests(query: &str) -> Result<Vec<OuterPull>> {
    let mut all_pulls = Vec::new();
    let mut after_cursor = None;
    for _n in 0..10 {
        let query_str = format!(
            r#"
            query {{
                search(query: "{}", type: ISSUE, first: 100, after: {}) {{
                    issueCount
                    edges {{
                        node {{
                            ... on PullRequest {{
                                title
                                url
                                repository {{
                                    url
                                }}
                                author {{
                                    login
                                }}
                                labels(first: 10) {{
                                    edges {{
                                        node {{
                                            name
                                        }}
                                    }}
                                }}
                                hasApprovedReview: reviews(first: 5, states: [APPROVED]) {{
                                    edges {{
                                        node {{
                                            author {{
                                                login
                                            }}
                                            state
                                        }}
                                    }}
                                }}
                                mergedBy {{
                                    login
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
            query,
            after_cursor
                .as_ref()
                .map_or(String::from("null"), |c| format!("\"{}\"", c))
        );

        let response_body = github_http_post_gql(&query_str).await?;
        let response: GraphQLResponse = serde_json::from_slice(&response_body)?;

        for edge in response.data.search.edges {
            let pull = edge.node;

            let labels = pull
                .labels
                .edges
                .into_iter()
                .map(|edge| edge.node.name)
                .collect();

            let reviews = pull
                .hasApprovedReview
                .edges
                .into_iter()
                .map(|edge| edge.node.author.login)
                .collect();

            all_pulls.push(OuterPull {
                title: pull.title,
                url: pull.url,
                author: pull.author.login,
                repository: pull.repository.url,
                labels,
                reviews,
                merged_by: pull.mergedBy.login,
            });
        }

        match response.data.search.pageInfo {
            PageInfo {
                hasNextPage: true,
                endCursor: Some(cursor),
            } => after_cursor = Some(cursor),
            _ => break,
        }
    }

    Ok(all_pulls)
}
