use dotenv::dotenv;
use std::collections::HashSet;
use octocrab::Octocrab;
use chrono::{Utc, Duration};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let octocrab = Octocrab::builder().personal_token(token).build().expect("token invalid");


    let one_year_ago = (Utc::now() - Duration::weeks(52i64))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let query = format!("is:issue mentions:Hacktoberfest updated:>{one_year_ago}");
    println!("query: {:?}", query.clone());

    let issues = octocrab
        .search()
        .issues_and_pull_requests(&query)
        .sort("comments")
        .order("desc")
        .send()
        .await?;

    for issue in issues.items {
        println!("issue: {:?}", issue.title);
    }


    Ok(())
}


