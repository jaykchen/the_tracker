use chrono::{Duration, NaiveDate, TimeDelta, Utc};
use dotenv::dotenv;
use octocrab::search;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::env;
use the_tracker::db_updater_local::*;
use the_tracker::issues_tracker_local::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    search_pulls().await?;
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
    let query_vec = inner_query_by_date_range(
        start_date,
        n_days,
        issue_label,
        pr_label,
        is_issue,
        is_start,
    );

    let query = "repo:SarthakKeshari/calc_for_everything is:pr is:merged label:hacktoberfest-accepted created:2023-10-01..2023-10-02 review:approved -label:spam -label:invalid";

    let query = "label:hacktoberfest-accepted is:pr is:merged created:2023-10-01..2023-10-03 review:approved -label:spam -label:invalid";

    let query = "label:hacktoberfest is:issue is:closed created:2023-10-01..2023-10-03 -label:spam -label:invalid";
    let query = "label:hacktoberfest is:issue is:open no:assignee created:2023-10-01..2023-10-03 -label:spam -label:invalid";

    let iss = search_issues_open(&query).await?;

    for issue in iss {
        println!("issue: {:?}", issue);
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
    let query_vec = inner_query_by_date_range(
        start_date,
        n_days,
        issue_label,
        pr_label,
        is_issue,
        is_start,
    );

    let query = "label:hacktoberfest-accepted is:pr is:merged created:2023-10-01..2023-10-03 review:approved -label:spam -label:invalid";

    let query = "repo:SarthakKeshari/calc_for_everything is:pr is:merged label:hacktoberfest-accepted created:2023-10-01..2023-10-30 review:approved -label:spam -label:invalid";
    let pulls = get_per_repo_pull_requests(&query).await?;

    for issue in pulls {
        println!("issue: {:?}", issue);
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimplePull {
    pub title: String,
    pub url: String,
    pub author: String,
    pub connected_issues: Vec<String>,
    pub labels: Vec<String>,
    pub reviews: Vec<String>,      // authors whose review state is approved
    pub merged_by: Option<String>, // This field can be empty if the PR is not merged
}

pub async fn get_per_repo_pull_requests(query: &str) -> anyhow::Result<Vec<SimplePull>> {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct GraphQLResponse {
        data: Data,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Data {
        search: Search,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Search {
        issueCount: i32,
        nodes: Vec<Node>,
        pageInfo: PageInfo,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Node {
        title: String,
        url: String,
        author: Author,
        timelineItems: TimelineItems,
        labels: Labels,
        reviews: Reviews,
        mergedBy: Option<Author>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Author {
        login: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineItems {
        nodes: Vec<TimelineEvent>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineEvent {
        subject: Option<Subject>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Subject {
        url: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Labels {
        nodes: Vec<Label>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Label {
        name: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Reviews {
        nodes: Vec<Review>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Review {
        author: Author,
        state: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: bool,
    }

    let mut simplified_pulls = Vec::new();
    let mut after_cursor: Option<String> = None;

    for _n in 0..10 {
        let query_str = format!(
            r#"
            query {{
                search(query: "{}", type: ISSUE, first: 100, after: {}) {{
                    issueCount
                    nodes {{
                        ... on PullRequest {{
                            title
                            url
                            author {{
                                login
                            }}
                            timelineItems(first: 5, itemTypes: [CONNECTED_EVENT]) {{
                                nodes {{
                                    ... on ConnectedEvent {{
                                        subject {{
                                            ... on Issue {{
                                                url
                                            }}
                                        }}
                                    }}
                                }}
                            }}
                            labels(first: 10) {{
                                nodes {{
                                    name
                                }}
                            }}
                            reviews(first: 5, states: [APPROVED]) {{
                                nodes {{
                                    author {{
                                        login
                                    }}
                                    state
                                }}
                            }}
                            mergedBy {{
                                login
                            }}
                        }}
                    }}
                    pageInfo {{
                        endCursor
                        hasNextPage
                    }}
                }}
            }}
            "#,
            query,
            after_cursor
                .as_ref()
                .map_or(String::from("null"), |c| format!("\"{}\"", c))
        );

        let response_body = github_http_post_gql(&query_str).await?;
        let response: GraphQLResponse = serde_json::from_slice(&response_body)?;

        for node in response.data.search.nodes {
            let connected_issues = node
                .timelineItems
                .nodes
                .iter()
                .filter_map(|event| event.subject.as_ref().map(|subject| subject.url.clone()))
                .collect::<Vec<String>>();

            let labels = node
                .labels
                .nodes
                .iter()
                .map(|label| label.name.clone())
                .collect::<Vec<String>>();

            let reviews = node
                .reviews
                .nodes
                .iter()
                .filter(|review| review.state == "APPROVED")
                .map(|review| review.author.login.clone())
                .collect::<Vec<String>>();

            simplified_pulls.push(SimplePull {
                title: node.title,
                url: node.url,
                author: node.author.login,
                connected_issues,
                labels,
                reviews,
                merged_by: node.mergedBy.as_ref().map(|author| author.login.clone()),
            });
        }
        match response.data.search.pageInfo {
            PageInfo {
                hasNextPage: true,
                endCursor: Some(cursor),
            } => after_cursor = Some(cursor),
            _ => break,
        }
    }

    Ok(simplified_pulls)
}
