use dotenv::dotenv;

use octocrab::Octocrab;
use serde::{Deserialize, Serialize};

use the_tracker::issues_tracker_local::*;
use the_tracker::the_runner::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let   query ="label:hacktoberfest is:issue is:open created:>=2023-10-01 updated:>=2023-10-30 -label:spam -label:invalid";

    let _ = search_pulls().await?;

    // for date_range in date_range_vec {
    //     let query =
    //         format!("label:hacktoberfest is:issue is:open no:assignee created:{date_range}");
    //     println!("query: {:?}", query.clone());
    //     let issues = get_issues(&query).await?;


    // }

    Ok(())
}

async fn test_search_issue_comments() -> anyhow::Result<()> {
    let   query ="label:hacktoberfest is:issue is:open created:>=2023-10-01 updated:>=2023-10-30 -label:spam -label:invalid";

    let issues = search_issues_w_update_comments(&query).await?;
    // for date_range in date_range_vec {
    //     let query =
    //         format!("label:hacktoberfest is:issue is:open no:assignee created:{date_range}");
    //     println!("query: {:?}", query.clone());
    //     let issues = get_issues(&query).await?;

    for issue in issues {
        let comment = if issue.comments.is_empty() {
            "No comments".to_string()
        } else {
            issue.comments[0].clone()
        };
        println!("issue: {:?}", comment);

        let body = issue.body.chars().take(200).collect::<String>();
        let title = issue.title.chars().take(200).collect::<String>();
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

    let query = "label:hacktoberfest is:issue is:open no:assignee created:2023-10-01..2023-10-01 -label:spam -label:invalid";
    let query = "label:hacktoberfest is:issue is:closed created:2023-10-01..2023-10-30 -label:spam -label:invalid";

    let iss = search_issues_closed(query).await?;

    for issue in iss {
        if issue.close_pull_request.is_empty() {
        } else {
            println!("issue: {:?}", issue.close_pull_request);
            println!("issue: {:?}", issue.close_author);
            println!("issue: {:?}", issue.close_reason);
        };

    }
    Ok(())
}
async fn search_pulls() -> anyhow::Result<()> {
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

    let query = "label:hacktoberfest-accepted is:pr is:merged merged:2023-10-01..2023-10-02 review:approved -label:spam -label:invalid";

    let pulls = search_pull_requests(query).await?;

    println!("pulls: {:?}", pulls.len());
    for issue in pulls {
        println!("issue: {:?}", issue.merged_by);
    }
    Ok(())
}
