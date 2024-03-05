use chrono::{ Datelike, Timelike, Utc };
use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use github_flows::{ get_octo, GithubLogin };
use octocrab_wasi::{ models::issues::Issue, params::issues::Sort, params::Direction };
use openai_flows::{ chat::{ ChatModel, ChatOptions }, OpenAIFlows };
use schedule_flows::{ schedule_cron_job, schedule_handler };
use serde_json::{ json, to_string_pretty, Value };
use std::{ collections::HashMap, env };

use http_req::{ request::{ Method, Request }, response::Response, uri::Uri };
use serde::{ Deserialize, Serialize };
use chrono::Duration;

// #[schedule_handler]
// async fn handler(body: Vec<u8>) {
//     dotenv().ok();
//     logger::init();

//     let _ = search_for_mention().await;
// }
pub async fn search_for_mention() -> anyhow::Result<()> {
    let octocrab = get_octo(&GithubLogin::Default);
    let one_hour_ago = (Utc::now() - Duration::hours(100i64))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let one_year_ago = (Utc::now() - Duration::weeks(52i64))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let query = format!("is:issue mentions:Hacktoberfest updated:>{one_year_ago}");
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
    use http_req::{ request::Method, request::Request, uri::Uri };
    let token = env::var("GITHUB_TOKEN").expect("github_token is required");
    let base_url = "https://api.github.com/graphql";
    let base_url = Uri::try_from(base_url).unwrap();
    let mut writer = Vec::new();

    let query = serde_json::json!({"query": query});
    match
        Request::new(&base_url)
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

    match
        Request::new(&url)
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
