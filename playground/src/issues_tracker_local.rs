use chrono::Utc;

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
    pub labels: Vec<String>,
    pub comments: String,
}

pub async fn get_issues(query: &str) -> anyhow::Result<Vec<OuterIssue>> {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct GraphQLResponse {
        pub data: Data,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Data {
        pub search: Search,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Search {
        pub issueCount: i32,
        pub edges: Vec<Edge>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Edge {
        pub node: LocalIssue,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct LocalIssue {
        pub title: String,
        pub number: i32,
        pub author: Author,
        pub repository: Repository,
        pub labels: Labels,
        pub comments: Comments,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Author {
        pub login: String,
        pub avatarUrl: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Repository {
        pub name: String,
        pub owner: Owner,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Owner {
        pub login: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Labels {
        pub edges: Vec<LabelEdge>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct LabelEdge {
        pub node: Label,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Label {
        pub name: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Comments {
        pub edges: Vec<CommentEdge>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct CommentEdge {
        pub node: Comment,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Comment {
        pub author: Author,
        pub body: String,
    }

    let first_issues = 10;
    let first_comments = 10;

    let query = "label:hacktoberfest is:issue is:open no:assignee created:>2023-10-01";
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
        query.replace("\"", "\\\""), // Basic attempt to escape quotes in query string
        first_issues,
        first_comments
    );

    let response_body = github_http_post_gql(&query_str).await?;

    let response: GraphQLResponse = serde_json::from_slice(&response_body)?;

    let issues: Vec<OuterIssue> = response
        .data
        .search
        .edges
        .iter()
        .map(|edge| {
            let issue = &edge.node;

            let labels: Vec<String> = issue
                .labels
                .edges
                .iter()
                .map(|label_edge| label_edge.node.name.clone())
                .collect();

            let comments: String = issue
                .comments
                .edges
                .iter()
                .map(|comment_edge| {
                    format!(
                        "{}: {}",
                        comment_edge.node.author.login, comment_edge.node.body
                    )
                })
                .collect::<Vec<String>>()
                .join("\n"); // Using newline as a separator

            OuterIssue {
                title: issue.title.clone(),
                number: issue.number,
                author: issue.author.login.clone(),
                repository: format!("{}/{}", issue.repository.owner.login, issue.repository.name),
                labels,
                comments,
            }
        })
        .collect::<Vec<_>>();

    Ok(issues)
}
