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

pub async fn search_pull_requests_overall(query: &str) -> anyhow::Result<Vec<OuterPull>> {
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
        author: Option<Author>,
        labels: Labels,
        hasApprovedReview: Reviews,
        mergedBy: Option<Author>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct Repository {
        url: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Author {
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
    struct Reviews {
        edges: Option<Vec<ReviewEdge>>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ReviewEdge {
        node: Option<Review>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Review {
        author: Option<Author>,
    }
    
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
                .as_ref()
                .unwrap_or(&Vec::new())
                .into_iter()
                .filter_map(|edge| edge.node.as_ref())
                .map(|node| node.name.clone())
                .collect::<Vec<Option<_>>>();

            let reviews = pull
                .hasApprovedReview
                .edges
                .as_ref()
                .unwrap_or(&Vec::new())
                .into_iter()
                .filter_map(|edge| edge.node.as_ref())
                .map(|node| node.author.as_ref().and_then(|author| author.login.clone()))
                .collect::<Vec<Option<_>>>();

            all_pulls.push(OuterPull {
                title: pull.title.clone(),
                url: pull.url.clone(),
                author: pull
                    .author
                    .as_ref()
                    .and_then(|author| author.login.clone())
                    .unwrap_or_else(|| String::from("Unknown author")),
                repository: pull
                    .repository
                    .url
                    .clone()
                    .unwrap_or_else(|| String::from("Unknown repository")),
                labels: labels.into_iter().filter_map(|x| x).collect(),
                reviews: reviews.into_iter().filter_map(|x| x).collect(),
                merged_by: pull
                    .mergedBy
                    .as_ref()
                    .and_then(|author| author.login.clone())
                    .unwrap_or_else(|| String::from("Unknown merged_by")),
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
