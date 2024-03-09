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
