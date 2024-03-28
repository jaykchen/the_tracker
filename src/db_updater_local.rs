use anyhow::Result;
use dotenv::dotenv;
use mysql_async::Error;
pub use mysql_async::*;
use mysql_async::{prelude::*, Pool};
use serde_json::json;

pub async fn get_pool() -> Pool {
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("not url db url found");

    let opts = Opts::from_url(&url).unwrap();
    let builder = OptsBuilder::from_opts(opts);
    // The connection pool will have a min of 5 and max of 10 connections.
    let constraints = PoolConstraints::new(5, 10).unwrap();
    let pool_opts = PoolOpts::default().with_constraints(constraints);

    Pool::new(builder.pool_opts(pool_opts))
}

pub async fn project_exists(
    pool: &mysql_async::Pool,
    project_id: &str,
) -> Result<bool, mysql_async::Error> {
    let mut conn = pool.get_conn().await?;
    let result: Option<(i32,)> = conn
        .query_first(format!(
            "SELECT 1 FROM projects WHERE project_id = '{}'",
            project_id
        ))
        .await?;
    Ok(result.is_some())
}

/* pub async fn add_project(
    pool: &mysql_async::Pool,
    project_id: &str,
    project_logo: &str,
    issue_id: &str,
) -> Result<()> {
    let mut conn = pool.get_conn().await?;
    let issue_id_json: Value = json!(issue_id).into();

    let query = r"INSERT INTO projects (project_id, project_logo, issues_list)
                  VALUES (:project_id, :project_logo, :issues_list)";

    conn.exec_drop(
        query,
        params! {
            "project_id" => project_id,
            "project_logo" => project_logo,
            "issues_list" => issue_id_json,
        },
    )
    .await?;

    Ok(())
} */

/* pub async fn update_project(
    pool: &mysql_async::Pool,
    project_id: &str,
    issue_id: &str,
) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    let issue_id_json: Value = json!(issue_id).into();

    let params = params! {
        "issue_id" => &issue_id_json,
        "project_id" => project_id,
    };
    "UPDATE projects
        SET issues_list = JSON_ARRAY_APPEND(issues_list, '$', :issue_id)
        WHERE project_id = :project_id"
        .with(params)
        .run(&mut conn)
        .await?;

    Ok(())
} */

/* pub async fn list_projects(pool: &Pool) -> Result<Vec<(String, String, Vec<String>)>> {
    let mut conn = pool.get_conn().await?;
    let projects: Vec<(String, String, Vec<String>)> = conn
        .query_map(
            "SELECT project_id, project_logo, JSON_UNQUOTE(JSON_EXTRACT(issues_list, '$')) FROM projects ORDER BY project_id",
            |(project_id, project_logo, issues_list): (String, String, String)| {
                (project_id, project_logo, serde_json::from_str(&issues_list).unwrap_or_default())
            },
        )
        .await?;

    Ok(projects)
} */

/* pub async fn issue_exists(
    pool: &mysql_async::Pool,
    issue_id: &str,
) -> Result<bool, mysql_async::Error> {
    let mut conn = pool.get_conn().await?;
    let result: Option<(i32,)> = conn
        .query_first(format!(
            "SELECT 1 FROM issues WHERE issue_id = '{}'",
            issue_id
        ))
        .await?;
    Ok(result.is_some())
} */

/* pub async fn add_issue(
    pool: &mysql_async::Pool,
    issue_id: &str,
    project_id: &str,
    title: &str,
    description: &str,
) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    let query = r"INSERT INTO issues (issue_id, project_id, issue_title, issue_description)
                  VALUES (:issue_id, :project_id, :issue_title, :issue_description)";

    conn.exec_drop(
        query,
        params! {
            "issue_id" => issue_id,
            "project_id" => project_id,
            "issue_title" => title,
            "issue_description" => description,
        },
    )
    .await?;

    Ok(())
} */

pub async fn select_issue(
    pool: &mysql_async::Pool,
    issue_id: &str,
    issue_budget: i64,
) -> Result<(), mysql_async::Error> {
    let mut conn = pool.get_conn().await?;

    let query = r"UPDATE issues 
                  SET issue_budget = :issue_budget, 
                      review_status = 'approve'
                  WHERE issue_id = :issue_id";

    conn.exec_drop(
        query,
        params! {
            "issue_id" => issue_id,
            "issue_budget" => issue_budget,
        },
    )
    .await?;

    Ok(())
}

pub async fn approve_issue(
    pool: &mysql_async::Pool,
    issue_id: &str,
) -> Result<(), mysql_async::Error> {
    let mut conn = pool.get_conn().await?;

    let query = r"UPDATE issues 
                  SET issue_budget_approved = True, 
                      review_status = 'approve'
                  WHERE issue_id = :issue_id";

    let result = conn
        .exec_drop(
            query,
            params! {
                "issue_id" => issue_id,
            },
        )
        .await;

    Ok(())
}

/* pub async fn add_issue_checked(
    pool: &mysql_async::Pool,
    issue_id: &str,
    project_id: &str,
    title: &str,
    description: &str,
    repository_avatar: &str,
) -> Result<(), Error> {
    if project_exists(pool, project_id).await? {
        update_project(pool, project_id, issue_id).await?;
    } else {
        add_project(pool, project_id, repository_avatar, issue_id)
            .await
            .unwrap();
    }

    if issue_exists(pool, issue_id).await? {
    } else {
        add_issue(pool, issue_id, project_id, title, description).await?;
    }
    Ok(())
} */

/* pub async fn update_issue(
    pool: &mysql_async::Pool,
    issue_id: &str,
    issue_assignee: &str,
    issue_linked_pr: &str,
) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    let query = r"UPDATE issues
                  SET issue_assignee = :issue_assignee,
                      issue_linked_pr = :issue_linked_pr
                  WHERE issue_id = :issue_id";

    conn.exec_drop(
        query,
        params! {
            "issue_id" => issue_id,
            "issue_assignee" => issue_assignee,
            "issue_linked_pr" => issue_linked_pr,
        },
    )
    .await?;

    Ok(())
} */

/* pub async fn pull_request_exists(pool: &Pool, pull_id: &str) -> Result<bool, Error> {
    let mut conn = pool.get_conn().await?;
    let result: Option<(i32,)> = conn
        .query_first(format!(
            "SELECT 1 FROM pull_requests WHERE pull_id = '{}'",
            pull_id
        ))
        .await?;
    Ok(result.is_some())
} */

pub async fn populate_projects_table(pool: &mysql_async::Pool) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    let project_ids: Vec<String> = conn
        .query(
            r"
            SELECT DISTINCT project_id FROM issues_master
            ",
        )
        .await?;

    for project_id in project_ids {
        let (repo_stars, project_logo): (i32, String) = conn
            .exec_first(
                r"
                SELECT repo_stars, repo_avatar FROM issues_open
                WHERE project_id = :project_id
                ",
                params! { "project_id" => &project_id },
            )
            .await?
            .unwrap_or((0, String::new())); // Default values if no matching row is found

        // Insert data into the projects table
        let query = r"
            INSERT INTO projects (project_id, repo_stars, project_logo)
            VALUES (:project_id, :repo_stars, :project_logo)
            ";

        conn.exec_drop(
            query,
            params! {
                "project_id" => &project_id,
                "repo_stars" => repo_stars,
                "project_logo" => &project_logo,
            },
        )
        .await?;
    }

    Ok(())
}

pub async fn consolidate_issues(pool: &mysql_async::Pool) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    let select_query = r"
        SELECT 
            issues_open.issue_id, 
            issues_open.project_id, 
            issues_open.issue_title, 
            issues_open.issue_description, 
            issues_open.repo_stars, 
            issues_open.repo_avatar, 
            issues_closed.issue_assignees, 
            issues_closed.issue_linked_pr, 
            issues_comments.issue_status
        FROM issues_open
        LEFT JOIN issues_closed ON issues_open.issue_id = issues_closed.issue_id
        LEFT JOIN issues_comments ON issues_open.issue_id = issues_comments.issue_id";

    let result: Vec<(
        String,
        String,
        String,
        String,
        i32,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = conn.query(select_query).await?;

    let mut transaction = conn
        .start_transaction(mysql_async::TxOpts::default())
        .await?;
    for row in result {
        transaction.exec_drop(
            r"
                INSERT INTO issues_master (issue_id, project_id, issue_title, issue_description, issue_assignees, issue_linked_pr, issue_status)
                VALUES (:issue_id, :project_id, :issue_title, :issue_description, :issue_assignees, :issue_linked_pr, :issue_status)",
            params! {
                "issue_id" => &row.0,
                "project_id" => &row.1,
                "issue_title" => &row.2,
                "issue_description" => &row.3,
                "issue_assignees" => row.6.as_deref(),
                "issue_linked_pr" => row.7.as_deref(),
                "issue_status" => row.8.as_deref(),
            },
        )
        .await?;
    }

    transaction.commit().await?;

    Ok(())
}

pub async fn add_issues_open(
    pool: &Pool,
    issue_id: &str,
    project_id: &str,
    issue_title: &str,
    issue_description: &str,
    repo_stars: i32,
    repo_avatar: &str,
) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    let query = r"INSERT INTO issues_open (issue_id, project_id, issue_title, issue_description, repo_stars, repo_avatar)
                  VALUES (:issue_id, :project_id, :issue_title, :issue_description, :repo_stars, :repo_avatar)";

    conn.exec_drop(
        query,
        params! {
            "issue_id" => issue_id,
            "project_id" => project_id,
            "issue_title" => issue_title,
            "issue_description" => issue_description,
            "repo_stars" => repo_stars,
            "repo_avatar" => repo_avatar,
        },
    )
    .await?;

    Ok(())
}

pub async fn add_issues_closed(
    pool: &Pool,
    issue_id: &str,
    issue_assignees: &Vec<String>,
    issue_linked_pr: &str,
) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    let issue_assignees_json: Value = json!(issue_assignees).into();

    let query = r"INSERT INTO issues_closed (issue_id,  issue_assignees, issue_linked_pr)
                  VALUES (:issue_id, :issue_assignees, :issue_linked_pr)";

    conn.exec_drop(
        query,
        params! {
            "issue_id" => issue_id,
            "issue_assignees" => &issue_assignees_json,
            "issue_linked_pr" => issue_linked_pr,
        },
    )
    .await?;

    Ok(())
}

pub async fn add_issues_comments(
    pool: &Pool,
    issue_id: &str,
    comments: &Vec<String>,
) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    // let issue_status = todo!(comments);
    let issue_status = comments[0].clone();

    let query = r"INSERT INTO issues_comments (issue_id, issue_status)
                  VALUES (:issue_id, :issue_status)";

    conn.exec_drop(
        query,
        params! {
            "issue_id" => issue_id,
            "issue_status" => issue_status,
        },
    )
    .await?;

    Ok(())
}
pub async fn add_pull_request(
    pool: &Pool,
    pull_id: &str,
    title: &str,
    author: &str,
    project_id: &str,
    merged_by: &str,
    connected_issues: &Vec<String>,
    pull_status: &str,
) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    let connected_issues_json: Value = json!(connected_issues).into();

    let query = r"INSERT INTO pull_requests (pull_id, title, author, project_id, merged_by, connected_issues, pull_status)
                  VALUES (:pull_id, :title, :author, :project_id, :merged_by, :connected_issues, :pull_status)";

    match conn
        .exec_drop(
            query,
            params! {
                "pull_id" => pull_id,
                "title" => title,
                "author" => author,
                "project_id" => project_id,
                "connected_issues" => &connected_issues_json,
                "merged_by" => merged_by,
                "pull_status" => pull_status,
            },
        )
        .await
    {
        Ok(()) => println!("Pull request added successfully"),
        Err(e) => println!("Error adding pull request: {:?}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_issues_open() {
        let pool: Pool = get_pool().await;

        let issue_id = "https://github.com/test/test/issues/4";
        let project_id = "https://github.com/test/test14";
        let title = "Test Issue Checked";
        let description = "This is a test issue for the checked function.";
        let repository_avatar = "https://avatars.githubusercontent.com/u/test?v=4";
        let repo_stars = 123;

        let result = add_issues_open(
            &pool,
            issue_id,
            project_id,
            title,
            description,
            repo_stars,
            repository_avatar,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_issues_closed() {
        let pool: Pool = get_pool().await;

        let issue_id = "https://github.com/test/test/issues/5";
        let issue_assignees = vec!["assignee1".to_string(), "assignee2".to_string()];
        let issue_linked_pr = "https://github.com/test/test/pull/1";

        let result = add_issues_closed(&pool, issue_id, &issue_assignees, issue_linked_pr).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_issues_comments() {
        let pool: Pool = get_pool().await;

        let issue_id = "https://github.com/test/test/issues/4";
        let comments = vec!["This is a test comment.".to_string()];

        let result = add_issues_comments(&pool, issue_id, &comments).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_pull_request() {
        let pool: Pool = get_pool().await;

        let pull_id = "https://github.com/test/test/pull/1";
        let title = "Test Pull Request";
        let author = "test_author";
        let project_id = "https://github.com/test/test";
        let merged_by = "test_merger";
        let connected_issues = vec!["https://github.com/test/test/issues/4".to_string()];
        let pull_status = "merged";

        let result = add_pull_request(
            &pool,
            pull_id,
            title,
            author,
            project_id,
            merged_by,
            &connected_issues,
            pull_status,
        )
        .await;

        assert!(result.is_ok());
    }
}
