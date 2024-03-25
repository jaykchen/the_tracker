use crate::*;
use hyper::client::conn;
use lazy_static::lazy_static;
pub static ISSUE_LABEL: &str = "hacktoberfest";
pub static PR_LABEL: &str = "hacktoberfest-accepted";
pub static START_DATE: &str = "2023-10-01";
pub static END_DATE: &str = "2023-10-30";
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use mysql_async::Pool;
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

pub fn inner_query_1_hour(
    start_date: &str,
    start_hour: &str,
    end_hour: &str,
    issue_label: &str,
    pr_label: &str,
    is_issue: bool,
    is_comment: bool,
    is_start: bool,
) -> String {
    let date_range = format!("{}..{}", start_hour, end_hour);

    let query = if is_issue && is_start {
        format!("label:{issue_label} is:issue is:open no:assignee created:{date_range} -label:spam -label:invalid")
    } else if is_issue && !is_start {
        format!("label:{issue_label} is:issue is:closed updated:{date_range} -label:spam -label:invalid")
    } else if is_comment {
        format!("label:{issue_label} is:issue is:open created:>={start_date} updated:{date_range} -label:spam -label:invalid")
    } else {
        format!("label:{pr_label} is:pr is:merged merged:{date_range} review:approved -label:spam -label:invalid")
    };

    query
}

pub fn inner_query_n_days(
    start_date: &str,
    n_days: i64,
    issue_label: &str,
    pr_label: &str,
    is_issue: bool,
    is_start: bool,
) -> String {
    let start_date =
        NaiveDate::parse_from_str(start_date, "%Y-%m-%d").expect("Failed to parse date");

    let end_date = (start_date + Duration::days(n_days))
        .format("%Y-%m-%d")
        .to_string();

    let date_range = format!("{}..{}", start_date, end_date);

    let query = if is_issue && is_start {
        format!("label:{issue_label} is:issue is:open no:assignee created:{date_range} -label:spam -label:invalid")
    } else if is_issue && !is_start {
        format!("label:{issue_label} is:issue is:closed created:{date_range} -label:spam -label:invalid")
    } else {
        format!("label:{pr_label} is:pr is:merged created:{date_range} review:approved -label:spam -label:invalid")
    };

    query
}

pub fn inner_query_vec_by_date_range(
    start_date: &str,
    n_days: i64,
    issue_label: &str,
    pr_label: &str,
    is_issue: bool,
    is_start: bool,
) -> Vec<String> {
    let start_date =
        NaiveDate::parse_from_str(start_date, "%Y-%m-%d").expect("Failed to parse date");

    let date_point_vec = (0..20)
        .map(|i| {
            (start_date + Duration::days(n_days * i as i64))
                .format("%Y-%m-%d")
                .to_string()
        })
        .collect::<Vec<_>>();

    let date_range_vec = date_point_vec
        .windows(2)
        .map(|x| x.join(".."))
        .collect::<Vec<_>>();

    let mut out = Vec::new();
    for date_range in date_range_vec {
        let query = if is_issue && is_start {
            format!("label:{issue_label} is:issue is:open no:assignee created:{date_range} -label:spam -label:invalid")
        } else if is_issue && !is_start {
            format!("label:{issue_label} is:issue is:closed created:{date_range} -label:spam -label:invalid")
        } else {
            format!("label:{pr_label} is:pr is:merged created:{date_range} review:approved -label:spam -label:invalid")
        };
        out.push(query);
    }

    out
}

pub async fn run_hourly(pool: &Pool) -> anyhow::Result<()> {
    // let query ="label:hacktoberfest is:issue is:open created:>=2023-10-01 updated:2023-10-03..2023-10-04 -label:spam -label:invalid";
    let query_comment = inner_query_1_hour(
        &START_DATE,
        &THIS_HOUR,
        &NEXT_HOUR,
        ISSUE_LABEL,
        PR_LABEL,
        false,
        true,
        false,
    );
    log::info!("query_comment: {:?}", query_comment);
    // let _ = search_issues_w_update_comments(&query_comment).await;

    // let query_open ="label:hacktoberfest is:issue is:open no:assignee created:2023-10-01..2023-10-02 -label:spam -label:invalid";
    let query_open = inner_query_1_hour(
        &START_DATE,
        &THIS_HOUR,
        &NEXT_HOUR,
        ISSUE_LABEL,
        PR_LABEL,
        true,
        false,
        true,
    );
    log::info!("query_open: {:?}", query_open);
    let open_issue_obj = search_issues_open(&query_open).await?;

    // for iss in open_issue_obj {
    //     let project_logo = &iss.repository_avatar.clone();
    //     let issue_id = iss.url.clone();
    //     let project_id = iss.repository.clone();
    //     let title = iss.title.clone();
    //     let description = iss.body.clone();
    //     add_issue_checked(
    //         pool,
    //         &issue_id,
    //         &project_id,
    //         &title,
    //         &description,
    //         &project_logo,
    //     )
    //     .await;
    // }

    // let query_closed =
    //     "label:hacktoberfest is:issue is:closed updated:>=2023-10-01 -label:spam -label:invalid";
    let query_closed = inner_query_1_hour(
        &START_DATE,
        &THIS_HOUR,
        &NEXT_HOUR,
        ISSUE_LABEL,
        PR_LABEL,
        true,
        false,
        false,
    );
    log::info!("query_closed: {:?}", query_closed);
    let close_issue_obj = search_issues_closed(&query_closed).await?;

    for iss in close_issue_obj {
        let issue_id = iss.issue_id.clone();
        let issue_linked_pr = iss.issue_linked_pr.clone().unwrap_or_default();
        // let issue_assignee = iss.assignee.clone();
        let issue_assignee = String::new();

        update_issue(pool, &issue_id, &issue_assignee, &issue_linked_pr).await;
    }

    // let query_pr_overall ="label:hacktoberfest-accepted is:pr is:merged updated:2023-10-01..2023-10-02 review:approved -label:spam -label:invalid";
    let query_pull_request = inner_query_1_hour(
        &START_DATE,
        &THIS_HOUR,
        &NEXT_HOUR,
        ISSUE_LABEL,
        PR_LABEL,
        false,
        false,
        false,
    );
    log::info!("query_pull_request: {:?}", query_pull_request);
    let pull_request_obj = search_pull_requests(&query_pull_request).await?;

    for pull in pull_request_obj {
        let pull_id = pull.pull_id.clone();

        let title = pull.title.clone();
        let author = pull.author.clone().unwrap_or_default();
        let repository = pull
            .pull_id
            .clone()
            .rsplitn(3, '/')
            .nth(2)
            .unwrap_or("unknown")
            .to_string();
        let merged_by = pull.merged_by.unwrap_or_default();
        let connected_issue = pull.connected_issues.clone();

        add_pull_request(
            pool,
            &pull_id,
            &title,
            &author,
            &repository,
            &merged_by,
            &connected_issue,
        )
        .await;
    }

    Ok(())
}
