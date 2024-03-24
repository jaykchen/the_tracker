use anyhow::anyhow;

use http_req::{
    request::{Method, Request},
    uri::Uri,
};

use serde::{Deserialize, Serialize};
use std::env;

pub async fn github_http_post_gql(query: &str) -> anyhow::Result<Vec<u8>> {
    let token = env::var("GITHUB_TOKEN").expect("github_token is required");
    let base_url = "https://api.github.com/graphql";
    let base_url = Uri::try_from(base_url).unwrap();
    let mut writer = Vec::new();

    let query = serde_json::json!({"query": query});
    match Request::new(&base_url)
        .method(Method::POST)
        .header("User-Agent", "flows-network connector")
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Length", &query.to_string().len())
        .body(&query.to_string().into_bytes())
        .send(&mut writer)
    {
        Ok(res) => {
            if !res.status_code().is_success() {
                log::error!("Github http error {:?}", res.status_code());
                return Err(anyhow::anyhow!("Github http error {:?}", res.status_code()));
            }
            Ok(writer)
        }
        Err(_e) => {
            log::error!("Error getting response from Github: {:?}", _e);
            Err(anyhow::anyhow!(_e))
        }
    }
}

pub async fn search_issues_w_update_comments(query: &str) -> anyhow::Result<Vec<OuterIssue>> {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct GraphQLResponse {
        data: Option<Data>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Data {
        search: Option<Search>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Search {
        issueCount: Option<i32>,
        nodes: Option<Vec<IssueNode>>,
        pageInfo: Option<PageInfo>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: bool,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct IssueNode {
        url: Option<String>,
        body: Option<String>,
        comments: Option<Comments>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Comments {
        nodes: Option<Vec<Comment>>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Comment {
        author: Option<Author>,
        body: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Author {
        login: Option<String>,
    }

    let mut all_issues = Vec::new();
    let mut after_cursor: Option<String> = None;

    for _ in 0..10 {
        let query_str = format!(
            r#"
                query {{
                    search(query: "{}", type: ISSUE, first: 100, after: {}) {{
                        issueCount
                        nodes {{
                            ... on Issue {{
                                url
                                body
                                comments(first: 50) {{
                                    nodes {{
                                        author {{
                                            login
                                        }}
                                        body
                                }}
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
            query.replace("\"", "\\\""),
            after_cursor
                .as_ref()
                .map_or(String::from("null"), |c| format!("\"{:?}\"", c)),
        );

        let response_body = github_http_post_gql(&query_str)
            .await
            .map_err(|e| anyhow!("Failed to post GraphQL query: {}", e))?;

        let response: GraphQLResponse = serde_json::from_slice(&response_body)
            .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?;

        if let Some(data) = response.data {
            if let Some(search) = data.search {
                if let Some(nodes) = search.nodes {
                    for issue in nodes {
                        let temp_str = String::from("");
                        let comments = issue.comments.map_or(Vec::new(), |comments| {
                            comments.nodes.map_or(Vec::new(), |nodes| {
                                nodes
                                    .iter()
                                    .filter_map(|comment| {
                                        Some(format!(
                                            "{}: {}",
                                            comment.author.as_ref().map_or("", |a| a
                                                .login
                                                .as_ref()
                                                .unwrap_or(&temp_str)),
                                            comment.body.as_ref().unwrap_or(&temp_str)
                                        ))
                                    })
                                    .collect()
                            })
                        });

                        all_issues.push(OuterIssue {
                            url: issue.url.unwrap_or_default(),
                            body: issue.body.clone().unwrap_or_default(),
                            comments,
                            ..Default::default() // Use Default trait implementation to fill in missing fields
                        });
                    }
                }

                if let Some(page_info) = search.pageInfo {
                    if page_info.hasNextPage {
                        after_cursor = page_info.endCursor
                    } else {
                        break;
                    }
                }
            }
        }
    }

    Ok(all_issues)
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OuterIssue {
    pub title: String,
    pub url: String,
    pub author: String,
    pub body: String,
    pub repository: String,
    pub repository_stars: i64,
    pub repository_avatar: String,
    pub issue_labels: Vec<String>,
    pub comments: Vec<String>,
}

pub async fn search_issues_open(query: &str) -> anyhow::Result<Vec<OuterIssue>> {
    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct GraphQLResponse {
        data: Option<Data>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Data {
        search: Option<Search>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Search {
        issueCount: Option<i32>,
        nodes: Option<Vec<Issue>>,
        pageInfo: Option<PageInfo>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: bool,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Issue {
        title: String,
        url: String,
        body: Option<String>,
        author: Option<Author>,
        repository: Option<Repository>,
        labels: Option<LabelNodes>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Author {
        login: Option<String>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Repository {
        url: Option<String>,
        stargazers: Option<Stargazers>,
        owner: Option<Owner>,
        usesCustomOpenGraphImage: Option<bool>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Owner {
        avatarUrl: Option<String>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Stargazers {
        totalCount: Option<i64>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct LabelNodes {
        nodes: Option<Vec<Label>>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Label {
        name: Option<String>,
    }
    let mut all_issues = Vec::new();
    let mut after_cursor: Option<String> = None;

    for _ in 0..10 {
        let query_str = format!(
            r#"
            query {{
                search(query: "{}", type: ISSUE, first: 100, after: {}) {{
                    issueCount
                    nodes {{
                            ... on Issue {{
                                title
                                url
                                body
                                author {{
                                    login
                                }}
                                repository {{
                                    url
                                    stargazers {{
                                        totalCount
                                    }}
                                    owner {{
                                        avatarUrl
                                    }}
                                }}
                                labels(first: 10) {{
                                    nodes {{
                                        name
                                    }}
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
            query.replace("\"", "\\\""),
            after_cursor
                .as_ref()
                .map_or(String::from("null"), |c| format!("\"{}\"", c)),
        );

        let response_body = github_http_post_gql(&query_str)
            .await
            .map_err(|e| anyhow!("Failed to post GraphQL query: {}", e))?;

        let response: GraphQLResponse = serde_json::from_slice(&response_body)
            .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?;

        if let Some(data) = response.data {
            if let Some(search) = data.search {
                if let Some(nodes) = search.nodes {
                    for issue in nodes {
                        let labels = issue.labels.as_ref().map_or(Vec::new(), |labels| {
                            labels.nodes.as_ref().map_or(Vec::new(), |nodes| {
                                nodes
                                    .iter()
                                    .filter_map(|label| label.name.clone())
                                    .collect()
                            })
                        });

                        all_issues.push(OuterIssue {
                            title: issue.title,
                            url: issue.url,
                            author: issue.author.as_ref().map_or(String::new(), |author| {
                                author.login.clone().unwrap_or_default()
                            }),
                            body: issue.body.clone().unwrap_or_default(),
                            repository: issue
                                .repository
                                .as_ref()
                                .map_or(String::new(), |repo| repo.url.clone().unwrap_or_default()),
                            repository_stars: issue.repository.as_ref().map_or(0, |repo| {
                                repo.stargazers
                                    .as_ref()
                                    .map_or(0, |stars| stars.totalCount.unwrap_or(0))
                            }),
                            repository_avatar: issue.repository.as_ref().map_or(
                                String::new(),
                                |repo| {
                                    repo.owner.as_ref().map_or(String::new(), |owner| {
                                        owner.avatarUrl.clone().unwrap_or_default()
                                    })
                                },
                            ),
                            issue_labels: labels,
                            comments: Vec::<String>::new(),
                        });
                    }
                }

                if let Some(pageInfo) = search.pageInfo {
                    if pageInfo.hasNextPage {
                        after_cursor = pageInfo.endCursor;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    Ok(all_issues)
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CloseOuterIssue {
    pub title: String,
    pub url: String,
    pub author: String,
    pub body: String,
    pub repository: String,
    pub repository_stars: i64,
    pub issue_labels: Vec<String>,
    pub comments: Vec<String>, // Concat of author and comment
    pub close_reason: String,
    pub close_pull_request: String,
    pub close_author: String,
}

pub async fn search_issues_closed(query: &str) -> anyhow::Result<Vec<CloseOuterIssue>> {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct GraphQLResponse {
        data: Option<Data>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Data {
        search: Option<Search>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Search {
        issueCount: Option<i32>,
        nodes: Option<Vec<Issue>>,
        pageInfo: Option<PageInfo>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: bool,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Issue {
        url: Option<String>,
        labels: Option<LabelNodes>,
        timelineItems: Option<TimelineItems>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct LabelNodes {
        nodes: Option<Vec<Label>>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Label {
        name: Option<String>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineItems {
        nodes: Option<Vec<ClosedEvent>>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct ClosedEvent {
        stateReason: Option<String>,
        closer: Option<Closer>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Closer {
        title: Option<String>,
        url: Option<String>,
        author: Option<Author>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Author {
        login: Option<String>,
    }

    let mut all_issues = Vec::new();
    let mut after_cursor: Option<String> = None;

    for _ in 0..10 {
        let query_str = format!(
            r#"
            query {{
                search(query: "{}", type: ISSUE, first: 100, after: {}) {{
                    issueCount
                    nodes {{
                        ... on Issue {{
                            url
                            labels(first: 10) {{
                                nodes {{
                                    name
                                }}
                            }}
                            timelineItems(first: 10, itemTypes: [CLOSED_EVENT]) {{
                                nodes {{
                                    ... on ClosedEvent {{
                                        stateReason
                                        closer {{
                                            ... on PullRequest {{
                                                title
                                                url
                                                author {{
                                                    login
                                                }}
                                            }}
                                        }}
                                    }}
                            }}
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
            query.replace("\"", "\\\""),
            after_cursor
                .as_ref()
                .map_or(String::from("null"), |c| format!("\"{:?}\"", c)),
        );

        let response_body = github_http_post_gql(&query_str)
            .await
            .map_err(|e| anyhow!("Failed to post GraphQL query: {}", e))?;

        let response: GraphQLResponse = serde_json::from_slice(&response_body)
            .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?;

        if let Some(data) = response.data {
            if let Some(search) = data.search {
                if let Some(nodes) = search.nodes {
                    for issue in nodes {
                        let labels = issue.labels.as_ref().map_or(Vec::new(), |labels| {
                            labels.nodes.as_ref().map_or(Vec::new(), |nodes| {
                                nodes
                                    .iter()
                                    .filter_map(|label| label.name.clone())
                                    .collect()
                            })
                        });

                        let (close_reason, close_pull_request, close_author) = issue
                            .timelineItems
                            .as_ref()
                            .map_or((String::new(), String::new(), String::new()), |items| {
                                items.nodes.as_ref().map_or(
                                    (String::new(), String::new(), String::new()),
                                    |nodes| {
                                        nodes
                                            .iter()
                                            .filter_map(|event| {
                                                if let Some(closer) = &event.closer {
                                                    Some((
                                                        event
                                                            .stateReason
                                                            .clone()
                                                            .unwrap_or_default(),
                                                        closer.url.clone().unwrap_or_default(),
                                                        closer.author.as_ref().map_or(
                                                            String::new(),
                                                            |author| {
                                                                author
                                                                    .login
                                                                    .clone()
                                                                    .unwrap_or_default()
                                                            },
                                                        ),
                                                    ))
                                                } else {
                                                    Some((
                                                        String::new(),
                                                        String::new(),
                                                        String::new(),
                                                    ))
                                                }
                                            })
                                            .next()
                                            .unwrap_or((
                                                String::new(),
                                                String::new(),
                                                String::new(),
                                            ))
                                    },
                                )
                            });

                        all_issues.push(CloseOuterIssue {
                            title: String::new(),
                            url: issue.url.unwrap_or_default(),
                            author: String::new(),
                            body: String::new(),
                            repository: String::new(),
                            repository_stars: 0,
                            issue_labels: labels,
                            comments: Vec::<String>::new(),
                            close_reason: close_reason,
                            close_pull_request: close_pull_request,
                            close_author: close_author,
                        });
                    }
                }

                if let Some(pageInfo) = search.pageInfo {
                    if pageInfo.hasNextPage {
                        after_cursor = pageInfo.endCursor;
                    } else {
                        break;
                    }
                }
            }
        }
    }
    Ok(all_issues)
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct OuterPull {
    pub title: String,
    pub url: String,
    pub author: String,
    pub connected_issues: Vec<String>,
    pub labels: Vec<String>,
    pub reviews: Vec<String>, // authors whose review state is approved
    pub merged_by: String,
}

pub async fn search_pull_requests(query: &str) -> anyhow::Result<Vec<OuterPull>> {
    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct Author {
        login: Option<String>,
    }

    #[derive(Serialize, Deserialize, Default, Clone, Debug)]
    pub struct Subject {
        url: Option<String>,
    }

    #[derive(Serialize, Deserialize, Default, Clone, Debug)]
    pub struct ConnectedEvent {
        subject: Option<Subject>,
    }

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct Label {
        name: Option<String>,
    }

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct Review {
        author: Option<Author>,
        state: Option<String>,
    }

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct LabelNodes {
        nodes: Option<Vec<Label>>,
    }

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct ReviewNodes {
        nodes: Option<Vec<Review>>,
    }

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct Closer {
        title: Option<String>,
        url: Option<String>,
        author: Option<Author>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct TimelineItems {
        nodes: Option<Vec<ConnectedEvent>>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct PullRequest {
        title: String,
        url: String,
        author: Option<Author>,
        timelineItems: Option<TimelineItems>,
        labels: Option<LabelNodes>,
        reviews: Option<ReviewNodes>,
        mergedBy: Option<Author>,
        mergedAt: Option<i64>,
    }
    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct Search {
        issueCount: Option<i32>,
        nodes: Option<Vec<PullRequest>>,
        pageInfo: Option<PageInfo>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Default, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: bool,
    }

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct QueryResult {
        data: Search,
    }
    let mut all_pulls = Vec::new();
    let mut after_cursor = None;
    for _n in 0..10 {
        let query_str = format!(
            r#"
            query {{
                search(query: "{}", type: ISSUE, first: 1, after: {}) {{
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
                            mergedAt
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
                .map_or(String::from("null"), |c| format!("\"{:?}\"", c))
        );

        let response_body =  github_http_post_gql(&query_str).await?;
        // println!("{}", String::from_utf8_lossy(&response_body));
        // break;
        let response: QueryResult = serde_json::from_slice(&response_body)?;

        let search = response.data;
        if let Some(nodes) = search.nodes {
            for pull in nodes {
                let connected_issues = vec![    ];
                // let connected_issues = pull
                //     .timelineItems
                //     .and_then(|items| items.nodes)
                //     .unwrap_or_default()
                //     .into_iter()
                //     .filter_map(|item| item.subject)
                //     .map(|subject| subject.url.unwrap_or_default())
                //     .collect();

                let labels = pull
                    .labels
                    .and_then(|items| items.nodes)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|label| label.name.unwrap_or_default())
                    .collect();

                let reviews = pull
                    .reviews
                    .and_then(|items| items.nodes)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|review| review.author)
                    .map(|author| author.login.unwrap_or_default())
                    .collect();

                let outer_pull = OuterPull {
                    title: pull.title.clone(),
                    url: pull.url,
                    author: pull
                        .author
                        .map_or(String::new(), |a| a.login.unwrap_or_default()),
                    connected_issues,
                    labels,
                    reviews,
                    merged_by: pull
                        .mergedBy
                        .map_or(String::new(), |a| a.login.unwrap_or_default()),
                };

                all_pulls.push(outer_pull);
            }
        }
        if let Some(pageInfo) = search.pageInfo {
            if pageInfo.hasNextPage {
                after_cursor = pageInfo.endCursor;
            } else {
                break;
            }
        }
    }

    Ok(all_pulls)
}
