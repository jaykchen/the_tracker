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
        edges: Option<Vec<Edge>>,
        pageInfo: PageInfo,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: bool,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Edge {
        node: Option<Issue>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Issue {
        url: Option<String>,
        body: Option<String>,
        comments: Option<Comments>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Comments {
        edges: Option<Vec<CommentEdge>>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct CommentEdge {
        node: Option<Comment>,
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
                        edges {{
                            node {{
                                ... on Issue {{
                                    url
                                    body
                                    comments(first: 50) {{
                                        edges {{
                                            node {{
                                                author {{
                                                    login
                                                }}
                                                body
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
                .map_or(String::from("null"), |c| format!("\"{}\"", c)),
        );

        let response_body = github_http_post_gql(&query_str)
            .await
            .map_err(|e| anyhow!("Failed to post GraphQL query: {}", e))?;

        let response: GraphQLResponse = serde_json::from_slice(&response_body)
            .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?;

        if let Some(data) = response.data {
            if let Some(search) = data.search {
                for edge in search.edges.unwrap_or_default() {
                    if let Some(issue) = edge.node {
                        let temp_str = String::from("");
                        let comments = issue.comments.map_or(Vec::new(), |comments| {
                            comments.edges.map_or(Vec::new(), |edges| {
                                edges
                                    .iter()
                                    .filter_map(|edge| {
                                        edge.node.as_ref().map(|comment| {
                                            format!(
                                                "{}: {}",
                                                comment.author.as_ref().map_or("", |a| a
                                                    .login
                                                    .as_ref()
                                                    .unwrap_or(&temp_str)),
                                                comment.body.as_ref().unwrap_or(&temp_str)
                                            )
                                        })
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

                if search.pageInfo.hasNextPage {
                    after_cursor = search.pageInfo.endCursor
                } else {
                    break;
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
        edges: Option<Vec<Edge>>,
        pageInfo: PageInfo,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: bool,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Edge {
        node: Option<Issue>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Issue {
        title: Option<String>,
        url: Option<String>,
        body: Option<String>,
        author: Option<Author>,
        repository: Option<Repository>,
        labels: Option<Labels>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Author {
        login: Option<String>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Owner {
        avatarUrl: Option<String>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Repository {
        url: Option<String>,
        stargazers: Option<Stargazers>,
        owner: Option<Owner>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Stargazers {
        totalCount: Option<i64>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Labels {
        edges: Option<Vec<LabelEdge>>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct LabelEdge {
        node: Option<Label>,
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
                    edges {{
                        node {{
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
                                    edges {{
                                        node {{
                                            name
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
                .map_or(String::from("null"), |c| format!("\"{}\"", c)),
        );

        let response_body = github_http_post_gql(&query_str)
            .await
            .map_err(|e| anyhow!("Failed to post GraphQL query: {}", e))?;

        let response: GraphQLResponse = serde_json::from_slice(&response_body)
            .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?;

        if let Some(data) = response.data {
            if let Some(search) = data.search {
                for edge in search.edges.unwrap_or_default() {
                    if let Some(issue) = edge.node {
                        let labels = issue.labels.map_or(Vec::new(), |labels| {
                            labels.edges.map_or(Vec::new(), |edges| {
                                edges
                                    .iter()
                                    .filter_map(|edge| {
                                        edge.node
                                            .as_ref()
                                            .map(|label| label.name.clone().unwrap_or_default())
                                    })
                                    .collect()
                            })
                        });
                        let temp_str = String::from("");

                        all_issues.push(OuterIssue {
                            title: issue.title.unwrap_or_default(),
                            url: issue.url.unwrap_or_default(),
                            author: issue
                                .author
                                .clone()
                                .map_or(String::new(), |author| author.login.unwrap_or_default()),
                            body: issue.body.clone().unwrap_or_default(),
                            repository: issue
                                .repository
                                .clone() // Clone here
                                .map_or(String::new(), |repo| repo.url.unwrap_or_default()),
                            repository_stars: issue.repository.clone().map_or(0, |repo| {
                                repo.stargazers
                                    .map_or(0, |stars| stars.totalCount.unwrap_or(0))
                            }),
                            repository_avatar: issue.repository.map_or(String::new(), |repo| {
                                repo.owner.map_or(String::new(), |owner| {
                                    owner.avatarUrl.unwrap_or_default()
                                })
                            }),
                            issue_labels: labels,
                            comments: Vec::<String>::new(),
                        });
                    }
                }
                if search.pageInfo.hasNextPage {
                    after_cursor = search.pageInfo.endCursor
                } else {
                    break;
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
        edges: Option<Vec<Edge>>,
        pageInfo: PageInfo,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct PageInfo {
        endCursor: Option<String>,
        hasNextPage: bool,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Edge {
        node: Option<Issue>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Issue {
        url: Option<String>,
        labels: Option<Labels>,
        timelineItems: Option<TimelineItems>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Author {
        login: Option<String>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Labels {
        edges: Option<Vec<LabelEdge>>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct LabelEdge {
        node: Option<Label>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Label {
        name: Option<String>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineItems {
        edges: Option<Vec<TimelineEdge>>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct TimelineEdge {
        node: Option<ClosedEvent>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct ClosedEvent {
        stateReason: Option<String>,
        closer: Option<Closer>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Closer {
        title: Option<String>,
        url: Option<String>,
        author: Option<Author>,
    }

    let first_comments = 10;
    let first_timeline_items = 10;
    let mut all_issues = Vec::new();
    let mut after_cursor: Option<String> = None;

    for _ in 0..10 {
        let query_str = format!(
            r#"
            query {{
                search(query: "{}", type: ISSUE, first: 100, after: {}) {{
                    issueCount
                    edges {{
                        node {{
                            ... on Issue {{
                                url
                                labels(first: 10) {{
                                    edges {{
                                        node {{
                                            name
                                        }}
                                    }}
                                }}
                                timelineItems(first: 10, itemTypes: [CLOSED_EVENT]) {{
                                    edges {{
                                        node {{
                                            ... on ClosedEvent {{
                                                stateReason
                                                closer {{
                                                    __typename
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
                for edge in search.edges.unwrap_or_default() {
                    if let Some(issue) = edge.node {
                        let labels = issue.labels.map_or(Vec::new(), |labels| {
                            labels.edges.map_or(Vec::new(), |edges| {
                                edges
                                    .iter()
                                    .filter_map(|edge| {
                                        edge.node
                                            .as_ref()
                                            .map(|label| label.name.clone().unwrap_or_default())
                                    })
                                    .collect()
                            })
                        });
                        let temp_str = String::from("");

                        let (close_reason, close_pull_request, close_author) = issue
                            .timelineItems
                            .map_or((String::new(), String::new(), String::new()), |items| {
                                items.edges.map_or(
                                    (String::new(), String::new(), String::new()),
                                    |edges| {
                                        edges
                                            .iter()
                                            .filter_map(|edge| {
                                                edge.node.as_ref().map(|event| {
                                                    if let Some(closer) = &event.closer {
                                                        (
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
                                                        )
                                                    } else {
                                                        (
                                                            String::new(),
                                                            String::new(),
                                                            String::new(),
                                                        )
                                                    }
                                                })
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
                if search.pageInfo.hasNextPage {
                    after_cursor = search.pageInfo.endCursor
                } else {
                    break;
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

    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct Subject {
        url: Option<String>,
    }

    #[derive(Serialize, Deserialize, Default, Debug)]
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
    pub struct ConnectedEventNodes {
        nodes: Option<Vec<ConnectedEvent>>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct PullRequest {
        title: String,
        url: String,
        author: Option<Author>,
        timelineItems: Option<ConnectedEventNodes>,
        labels: Option<LabelNodes>,
        reviews: Option<ReviewNodes>,
        mergedBy: Option<Author>,
        mergedAt: Option<String>,
    }

    #[allow(non_snake_case)]
    #[derive(Serialize, Deserialize, Default, Debug)]
    pub struct Search {
        issueCount: Option<i64>,
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

        let response_body = github_http_post_gql(&query_str).await?;
        let response: QueryResult = serde_json::from_slice(&response_body)?;

        let search = response.data;
        if let Some(nodes) = search.nodes {
            for pull in nodes {
                let connected_issues = pull
                    .timelineItems
                    .and_then(|items| items.nodes)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|item| item.subject)
                    .map(|subject| subject.url.unwrap_or_default())
                    .collect();

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
                        .unwrap_or_default().login.unwrap_or_default(),
                    connected_issues,
                    labels,
                    reviews,
                    merged_by: pull
                        .mergedBy
                        .unwrap_or_default().login.unwrap_or_default(),
                };

                all_pulls.push(outer_pull);
            }
        }
        if let Some(pageInfo) = search.pageInfo {
            if pageInfo.hasNextPage {
                after_cursor = Some(pageInfo.endCursor);
            } else {
                break;
            }
        }
    }
    Ok(all_pulls)
}
