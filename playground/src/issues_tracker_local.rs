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

// #[schedule_handler]
// async fn handler(body: Vec<u8>) {
//     dotenv().ok();
//     logger::init();

//     let _ = search_for_initial_hits().await;
// }
pub async fn search_for_initial_hits() -> anyhow::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .expect("token invalid");
    let one_hour_ago = (Utc::now() - Duration::hours(100i64))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let one_year_ago = (Utc::now() - Duration::weeks(52i64))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    // label:hacktoberfest is:issue is:open no:assignee
    let query = format!("label:hacktoberfest is:issue is:open no:assignee updated:>{one_year_ago}");
    let encoded_query = urlencoding::encode(&query);

    let query_url = format!("https://api.github.com/search/issues?q={encoded_query}");
    log::error!("query: {:?}", query_url.clone());

    // let issues = octocrab
    //     .search()
    //     .issues_and_pull_requests(&query)
    //     .sort("comments")
    //     .order("desc")
    //     .send()
    //     .await?;

    if let Ok(writer) = github_http_get(&query_url).await {
        let issues: Vec<Issue> = serde_json::from_slice(&writer).unwrap();

        for issue in issues {
            log::error!("issue: {:?}", issue.title);
        }
    }

    Ok(())
}

pub async fn github_http_post_gql(query: &str) -> anyhow::Result<Vec<u8>> {
    use http_req::{request::Method, request::Request, uri::Uri};
    let token = env::var("GITHUB_TOKEN").expect("github_token is required");
    let base_url = "https://api.github.com/graphql";
    let base_url = Uri::try_from(base_url).unwrap();
    let mut writer = Vec::new();

    let query = serde_json::json!({"query": query});
    match Request::new(&base_url)
        .method(Method::POST)
        .header("User-Agent", "flows-network connector")
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Length", &query.to_string().len())
        .body(&query.to_string().into_bytes())
        .send(&mut writer)
    {
        Ok(res) => {
            if !res.status_code().is_success() {
                log::error!("Github http error {:?}", res.status_code());
                return Err(anyhow::anyhow!("Github http error {:?}", res.status_code()));
            }
            Ok(writer)
        }
        Err(_e) => {
            log::error!("Error getting response from Github: {:?}", _e);
            Err(anyhow::anyhow!(_e))
        }
    }
}

pub async fn github_http_get(url: &str) -> anyhow::Result<Vec<u8>> {
    let token = std::env::var("GITHUB_TOKEN").expect("github_token is required");
    let mut writer = Vec::new();
    let url = Uri::try_from(url).unwrap();

    match Request::new(&url)
        .method(Method::GET)
        .header("User-Agent", "flows-network connector")
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .header("CONNECTION", "close")
        .send(&mut writer)
    {
        Ok(res) => {
            if !res.status_code().is_success() {
                println!("Github http error {:?}", res.status_code());
                return Err(anyhow::anyhow!("Github http error {:?}", res.status_code()));
            }
            Ok(writer)
        }
        Err(_e) => {
            println!("Error getting response from Github: {:?}", _e);
            Err(anyhow::anyhow!(_e))
        }
    }
}

pub async fn get_project_logo(owner: &str, repo: &str) -> anyhow::Result<String> {
    #[derive(Serialize, Deserialize)]
    struct GraphQLResponse {
        data: RepositoryData,
    }

    #[derive(Serialize, Deserialize)]
    struct RepositoryData {
        repository: OwnerData,
    }

    #[derive(Serialize, Deserialize)]
    struct OwnerData {
        owner: OwnerInfo,
    }

    #[derive(Serialize, Deserialize)]
    struct OwnerInfo {
        login: String,
        avatarUrl: String,
    }

    let query_str = format!(
        r#"
        query {{
            repository(owner: "{owner}", name: "{repo}") {{
                owner {{
                    login
                    ... on User {{
                        avatarUrl
                    }}
                    ... on Organization {{
                        avatarUrl
                    }}
                }}
            }}
        }}
        "#,
    );

    let response = github_http_post_gql(&query_str).await?;

    let parsed_response: GraphQLResponse = serde_json::from_slice(&response)?;
    let owner_info = parsed_response.data.repository.owner;
    Ok(owner_info.avatarUrl)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OuterIssue {
    pub title: String,
    pub number: i32,
    pub author: String,
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
        stargazers: Option<Stargazers>, // Add stargazers information
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
    let first_issues = 100;
    let first_comments = 10;
    // let query = "label:hacktoberfest is:issue is:open no:assignee created:>2023-10-01";
    let query_str = format!(
        r#"
        query {{
            search(query: "{}", type: ISSUE, first: {}) {{
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
            }}
        }}
        "#,
        query, // Basic attempt to escape quotes in query string
        first_issues,
        first_comments
    );
    // query.replace("\"", "\\\""), // Basic attempt to escape quotes in query string

    let response_body = github_http_post_gql(&query_str)
        .await
        .map_err(|e| anyhow!("Failed to post GraphQL query: {}", e))?;

    let response: GraphQLResponse = serde_json::from_slice(&response_body)
        .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?;

    let issues = response.data.and_then(|data| data.search).map_or(
        Ok::<Vec<OuterIssue>, anyhow::Error>(vec![]), // Explicit type annotations for Ok
        |search| {
            search.edges.map_or(
                Ok(vec![]), // Here, the error type can be inferred from the outer context.
                |edges| {
                    let issues = edges
                        .iter()
                        .filter_map(|edge| {
                            edge.node.as_ref().map(|issue| {
                                let labels = issue
                                    .labels
                                    .as_ref()
                                    .and_then(|l| l.edges.as_ref())
                                    .map_or(vec![], |edges| {
                                        edges
                                            .iter()
                                            .filter_map(|edge| {
                                                edge.node
                                                    .as_ref()
                                                    .and_then(|label| label.name.clone())
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
                                                            .unwrap_or(&String::new())
                                                            .to_owned(), // Correctly handle the conversion here.
                                                        comment
                                                            .body
                                                            .as_ref()
                                                            .unwrap_or(&String::new())
                                                    )
                                                })
                                            })
                                            .collect::<Vec<String>>()
                                            .join("\n")
                                    });

                                OuterIssue {
                                    title: issue.title.clone().unwrap_or_default(),
                                    number: issue.number.unwrap_or_default(),
                                    url: issue.url.clone().unwrap_or_default(),
                                    author: issue
                                        .author
                                        .as_ref()
                                        .and_then(|a| a.login.clone())
                                        .unwrap_or_default(),
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
                                    labels,
                                    comments,
                                }
                            })
                        })
                        .collect();
                    Ok(issues)
                },
            )
        },
    )?; // Use a single `?` here to propagate the error up if there is one.

    Ok(issues)
}
