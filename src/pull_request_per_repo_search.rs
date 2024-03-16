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
pub struct SimplePull {
    pub title: String,
    pub url: String,
    pub author: String,
    pub connected_issues: Vec<String>,
    pub labels: Vec<String>,
    pub reviews: Vec<String>,      // authors whose review state is approved
    pub merged_by: Option<String>, // This field can be empty if the PR is not merged
}

pub async fn get_per_repo_pull_requests(query: &str) -> anyhow::Result<Vec<SimplePull>> {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct GraphQLResponse {
        data: Data,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Data {
        search: Search,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Search {
        issueCount: i32,
        nodes: Vec<Node>,
        pageInfo: PageInfo,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Node {
        title: String,
        url: String,
        author: Author,
        timelineItems: TimelineItems,
        labels: Labels,
        reviews: Reviews,
        mergedBy: Option<Author>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Author {
        login: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineItems {
        nodes: Vec<TimelineEvent>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineEvent {
        subject: Option<Subject>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Subject {
        url: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Labels {
        nodes: Vec<Label>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Label {
        name: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Reviews {
        nodes: Vec<Review>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Review {
        author: Author,
        state: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: bool,
    }

    let mut simplified_pulls = Vec::new();
    let mut after_cursor: Option<String> = None;

    for _n in 0..10 {
        let query_str = format!(
            r#"
            query {{
                search(query: "{}", type: ISSUE, first: 100, after: {}) {{
                    issueCount
                    nodes {{
                        ... on PullRequest {{
                            title
                            url
                            author {{
                                login
                            }}
                            timelineItems(first: 5, itemTypes: [CONNECTED_EVENT]) {{
                                nodes {{
                                    ... on ConnectedEvent {{
                                        subject {{
                                            ... on Issue {{
                                                url
                                            }}
                                        }}
                                    }}
                                }}
                            }}
                            labels(first: 10) {{
                                nodes {{
                                    name
                                }}
                            }}
                            reviews(first: 5, states: [APPROVED]) {{
                                nodes {{
                                    author {{
                                        login
                                    }}
                                    state
                                }}
                            }}
                            mergedBy {{
                                login
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

        for node in response.data.search.nodes {
            let connected_issues = node
                .timelineItems
                .nodes
                .iter()
                .filter_map(|event| event.subject.as_ref().map(|subject| subject.url.clone()))
                .collect::<Vec<String>>();

            let labels = node
                .labels
                .nodes
                .iter()
                .map(|label| label.name.clone())
                .collect::<Vec<String>>();

            let reviews = node
                .reviews
                .nodes
                .iter()
                .filter(|review| review.state == "APPROVED")
                .map(|review| review.author.login.clone())
                .collect::<Vec<String>>();

            simplified_pulls.push(SimplePull {
                title: node.title,
                url: node.url,
                author: node.author.login,
                connected_issues,
                labels,
                reviews,
                merged_by: node.mergedBy.as_ref().map(|author| author.login.clone()),
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

    Ok(simplified_pulls)
}
