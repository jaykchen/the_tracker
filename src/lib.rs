pub mod db_updater_local;
pub mod issues_tracker_local;
pub mod the_runner;
use chrono::{NaiveDateTime, NaiveTime, Timelike, Utc};
use lazy_static::lazy_static;

pub static ISSUE_LABEL: &str = "hacktoberfest";
pub static PR_LABEL: &str = "hacktoberfest-accepted";
pub static START_DATE: &str = "2023-10-01";
pub static END_DATE: &str = "2023-10-30";

lazy_static! {
    static ref TODAY_PLUS_TEN_MINUTES: NaiveDateTime = Utc::now()
        .date()
        .naive_utc()
        .and_time(NaiveTime::from_hms(0, 10, 0));
    static ref TODAY_THIS_HOUR: u32 = Utc::now().hour();
}
