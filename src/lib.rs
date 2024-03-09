pub mod db_updater;
pub mod issues_tracker;
use chrono::{Datelike, Timelike, Utc};
use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use github_flows::{get_octo, GithubLogin};
use octocrab_wasi::{models::issues::Issue, params::issues::Sort, params::Direction};
use openai_flows::{
    chat::{ChatModel, ChatOptions},
    OpenAIFlows,
};
use schedule_flows::{schedule_cron_job, schedule_handler};
use serde_json::{json, to_string_pretty, Value};
use std::{collections::HashMap, env};

use chrono::Duration;
use http_req::{
    request::{Method, Request},
    response::Response,
    uri::Uri,
};
pub use issues_tracker::search_for_mention;
use serde::{Deserialize, Serialize};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    let now = Utc::now();
    let now_minute = now.minute() + 2;
    let cron_time = format!("{:02} {:02} {:02} * *", now_minute, now.hour(), now.day());
    schedule_cron_job(cron_time, String::from("cron_job_evoked")).await;
}

#[schedule_handler]
async fn handler(body: Vec<u8>) {
    let _ = inner(body).await;
}

pub async fn inner(body: Vec<u8>) -> anyhow::Result<()> {
    dotenv().ok();
    logger::init();

    // let _ = search_for_mention().await;
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    let issue_id = "https://github.com/jaykchen/issue-labeler/issues/24";

    let res = list_comments(&pool, issue_id).await?;
    log::info!("Comments: {:?}", res);

    Ok(())
}

use db_updater::*;
use sqlx::postgres::PgPool;

async fn run_db() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    let _ = add_project_test_1(&pool).await?;
    //
    // let project_id = "jaykchen/issue-labeler";

    // let res = list_projects(&pool).await?;
    // println!("Projects: {:?}", res);
    // let res = list_issues(&pool, project_id).await?;
    let _ = add_issue_test_1(&pool).await?;
    let _ = add_comment_test_1(&pool).await?;

    let issue_id = "https://github.com/jaykchen/issue-labeler/issues/24";

    let res = list_comments(&pool, issue_id).await?;
    println!("Comments: {:?}", res);
    Ok(())
}
