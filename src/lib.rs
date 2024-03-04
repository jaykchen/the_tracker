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
    dotenv().ok();
    logger::init();

}