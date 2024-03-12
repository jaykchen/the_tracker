use chrono::{Duration, Utc};
use dotenv::dotenv;
use octocrab::Octocrab;
use playground::db_updater_local::*;
use playground::issues_tracker_local::*;
use sqlx::postgres::PgPool;

use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .expect("token invalid");

    let one_year_ago = (Utc::now() - Duration::weeks(52i64))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let query = format!("is:issue mentions:Hacktoberfest updated:>{one_year_ago}");
    println!("query: {:?}", query.clone());

    // let issues = octocrab
    //     .search()
    //     .issues_and_pull_requests(&query)
    //     .sort("comments")
    //     .order("desc")
    //     .send()
    //     .await?;

    let issues = get_issues(&query).await?;

    for issue in issues {
        println!("issue: {:?}", issue.title);
    }

    Ok(())
}

async fn run_db() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    // let _ = add_project_test_1(&pool).await?;

    // let project_id = "jaykchen/issue-labeler";

    // let res = list_projects(&pool).await?;
    // println!("Projects: {:?}", res);
    // let res = list_issues(&pool, project_id).await?;
    // let _ = add_comment_test_1(&pool).await?;
    // let _ = add_issue_test_1(&pool).await?;

    let issue_id = "https://github.com/jaykchen/issue-labeler/issues/24";

    let res = list_comments(&pool, issue_id).await?;
    println!("Comments: {:?}", res);
    Ok(())
}
