use dotenv::dotenv;

use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

use lazy_static::*;
use the_tracker::db_updater_local;
use the_tracker::issues_tracker_local::*;
use the_tracker::the_runner::*;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
pub static START_DATE: &str = "2023-10-01";
pub static END_DATE: &str = "2023-10-30";

lazy_static! {
    static ref THIS_HOUR: String = (NaiveDate::parse_from_str("2023-10-01", "%Y-%m-%d").unwrap()
        + Duration::hours(Utc::now().hour() as i64))
    .to_string();
    static ref NEXT_HOUR: String = (NaiveDate::parse_from_str("2023-10-01", "%Y-%m-%d").unwrap()
        + Duration::hours(Utc::now().hour() as i64 + 1))
    .to_string();
    static ref TODAY_PLUS_TEN_MINUTES: NaiveDateTime = Utc::now()
        .date()
        .naive_utc()
        .and_time(NaiveTime::from_hms(0, 10, 0));
    static ref TODAY_THIS_HOUR: u32 = Utc::now().hour();
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let   query ="label:hacktoberfest is:issue is:open created:>=2023-10-01 updated:>=2023-10-30 -label:spam -label:invalid";

    let _ = search_pulls().await?;
    // let pool = db_updater_local::get_pool().await;
    // let _ = run_hourly(&pool).await?;

    // for date_range in date_range_vec {
    //     let query =
    //         format!("label:hacktoberfest is:issue is:open no:assignee created:{date_range}");
    //     println!("query: {:?}", query.clone());
    //     let issues = get_issues(&query).await?;

    // }

    Ok(())
}

async fn test_search_issue_comments() -> anyhow::Result<()> {
    let   query ="label:hacktoberfest is:issue is:open created:>=2023-10-01 updated:>=2023-10-30T00:10:00 -label:spam -label:invalid";

    let issues = search_issues_w_update_comments(&query).await?;
    // for date_range in date_range_vec {
    //     let query =
    //         format!("label:hacktoberfest is:issue is:open no:assignee created:{date_range}");
    //     println!("query: {:?}", query.clone());
    //     let issues = get_issues(&query).await?;

    for issue in issues {
        let comment = if issue.issue_status.is_empty() {
            "No comments".to_string()
        } else {
            issue.issue_status.clone()
        };
        println!("issue: {:?}", comment);

        let body = issue
            .issue_description
            .chars()
            .take(200)
            .collect::<String>();
        let title = issue.issue_title.chars().take(200).collect::<String>();
    }
    // }

    Ok(())
}

async fn search_issues() -> anyhow::Result<()> {
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
    let query_vec = inner_query_vec_by_date_range(
        start_date,
        n_days,
        issue_label,
        pr_label,
        is_issue,
        is_start,
    );

    let query = "repo:SarthakKeshari/calc_for_everything is:pr is:merged label:hacktoberfest-accepted created:2023-10-01..2023-10-02 review:approved -label:spam -label:invalid";

    let query = "label:hacktoberfest-accepted is:pr is:merged created:2023-10-01..2023-10-03 review:approved -label:spam -label:invalid";

    let query = "label:hacktoberfest is:issue is:closed created:2023-10-01..2023-10-30 -label:spam -label:invalid";
    let query = "label:hacktoberfest is:issue is:open created:2023-10-01..2023-10-01 -label:spam -label:invalid";

    let iss = search_issues_w_update_comments(query).await?;

    for issue in iss {
        println!("issue: {:?}", issue.issue_assignees);
    }
    Ok(())
}
async fn search_pulls() -> anyhow::Result<()> {
    let start_date = "2023-10-01";
    let issue_label = "hacktoberfest";
    let pr_label = "hacktoberfest-accepted";
    let n_days = 3;
    let is_issue = false;
    let is_start = false;
    let query = inner_query_1_hour(
        start_date,
        &THIS_HOUR,
        &NEXT_HOUR,
        issue_label,
        pr_label,
        is_issue,
        is_start,
        false,
    );

    println!("query: {:?}", query.clone());
    // let query = "label:hacktoberfest-accepted is:pr is:merged merged:2023-10-01..2023-10-01 review:approved -label:spam -label:invalid";
    // let query = "is:pr is:merged label:hacktoberfest-accepted created:2023-10-01..2023-10-30 review:approved -label:spam -label:invalid";

    let pulls = search_pull_requests(&query).await?;

    println!("pulls: {:?}", pulls.len());
    for issue in pulls {
        if issue.connected_issues.len() > 0 {
            println!("issue: {:?}", issue.connected_issues);
        }
    }
    Ok(())
}
