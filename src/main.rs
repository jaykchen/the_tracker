use chrono::{Duration, NaiveDate, TimeDelta, Utc};
use dotenv::dotenv;
use octocrab::Octocrab;
use sqlx::postgres::PgPool;
use the_tracker::db_updater_local::*;
use the_tracker::issues_tracker_local::*;

use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .expect("token invalid");

    let start_date = "2023-10-01";
    let issue_label = "hacktoberfest";
    let pr_label = "hacktoberfest-accepted";
    let n_days = 3;
    let is_issue = true;
    let is_start = true;
    let query_vec = inner_query_by_date_range(
        start_date,
        n_days,
        issue_label,
        pr_label,
        is_issue,
        is_start,
    );

    let query = "repo:SarthakKeshari/calc_for_everything is:pr is:merged label:hacktoberfest-accepted created:2023-10-01..2023-10-03 review:approved -label:spam -label:invalid";

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    // for query in query_vec {
    println!("query: {:?}", query.clone());
    let pulls = get_per_repo_pull_requests(&query).await?;

    for pull in pulls {
        println!("pull: {:?}", pull);

        // let _ = add_pull_request_with_check(
        //     &pool,
        //     &pull.url,
        //     &pull.title,
        //     &pull.author,
        //     &pull.repository,
        //     &pull.merged_by,
        //     pull.cross_referenced_issues,
        // )
        // .await?;

        // pub async fn add_pull_request_with_check(
        //     pool: &sqlx::PgPool,
        //     pull_id: &str,
        //     title: &str,
        //     author: &str,
        //     repository: &str,
        //     merged_by: &str,
        //     cross_referenced_issues: Vec<String>,
        // let body = issue.body.chars().take(200).collect::<String>();
        // let title = issue.title.chars().take(200).collect::<String>();
        // let _ = (&pool, &issue.url, &title, &body).await?;
    }
    // }

    // for date_range in date_range_vec {
    //     let query =
    //         format!("label:hacktoberfest is:issue is:open no:assignee created:{date_range}");
    //     println!("query: {:?}", query.clone());
    //     let issues = get_issues(&query).await?;

    //     for issue in issues {
    //         println!("issue: {:?}", issue.url);

    //         let body = issue.body.chars().take(200).collect::<String>();
    //         let title = issue.title.chars().take(200).collect::<String>();
    //         let _ = add_issue_with_check(&pool, &issue.url, &title, &body).await?;
    //     }
    // }

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

    // let res = list_comments(&pool, issue_id).await?;
    // println!("Comments: {:?}", res);
    Ok(())
}
