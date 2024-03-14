use chrono::{Duration, NaiveDate, TimeDelta, Utc};
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

    // let query = format!("is:issue mentions:Hacktoberfest updated:>{one_year_ago}");
    // label:hacktoberfest is:issue is:open no:assignee created:2023-10-01..2023-10-03 sort:interactions-desc

    let start_date =
        NaiveDate::parse_from_str("2023-10-01", "%Y-%m-%d").expect("Failed to parse date");

    let mut date_point_vec = Vec::new();

    for i in 0..20 {
        let three_days_str = (start_date + Duration::days(2 * i as i64))
            .format("%Y-%m-%d")
            .to_string();

        date_point_vec.push(three_days_str);
    }

    let mut date_range_vec = date_point_vec
        .windows(2)
        .map(|x| x.join(".."))
        .collect::<Vec<_>>();

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    for date_range in date_range_vec {
        let query =
            format!("label:hacktoberfest-accepted is:pr is:merged created:{date_range} review:approved -label:spam -label:invalid");
        println!("query: {:?}", query.clone());
        let pulls = get_pull_requests(&query).await?;

        for pull in pulls {
            println!("pull: {:?}", pull.url);
            println!("pull: {:?}", pull.repository);

            // let body = issue.body.chars().take(200).collect::<String>();
            // let title = issue.title.chars().take(200).collect::<String>();
            // let _ = (&pool, &issue.url, &title, &body).await?;
        }
        break;
    }

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

    let res = list_comments(&pool, issue_id).await?;
    println!("Comments: {:?}", res);
    Ok(())
}
