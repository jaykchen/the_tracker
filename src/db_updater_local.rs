use anyhow::Result;
use dotenv::dotenv;
use mysql_async::Error;
pub use mysql_async::*;
use mysql_async::{prelude::*, Pool};
use serde_json::json;

async fn get_pool() -> Pool {
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

pub async fn add_project(
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
}

pub async fn update_project(
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
}

pub async fn list_projects(pool: &Pool) -> Result<Vec<(String, String, Vec<String>)>> {
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
}

pub async fn issue_exists(
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
}

pub async fn add_issue(
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
}

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

pub async fn add_issue_checked(
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
}

pub async fn update_issue(
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
}

pub async fn pull_request_exists(pool: &Pool, pull_id: &str) -> Result<bool, Error> {
    let mut conn = pool.get_conn().await?;
    let result: Option<(i32,)> = conn
        .query_first(format!(
            "SELECT 1 FROM pull_requests WHERE pull_id = '{}'",
            pull_id
        ))
        .await?;
    Ok(result.is_some())
}

pub async fn add_pull_request(
    pool: &Pool,
    pull_id: &str,
    title: &str,
    author: &str,
    repository: &str,
    merged_by: &str,
) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    let query = r"INSERT INTO pull_requests (pull_id, title, author, repository, merged_by)
                  VALUES (:pull_id, :title, :author, :repository, :merged_by)";

    conn.exec_drop(
        query,
        params! {
            "pull_id" => pull_id,
            "title" => title,
            "author" => author,
            "repository" => repository,
            "merged_by" => merged_by,
        },
    )
    .await?;

    Ok(())
}

pub async fn update_pull_request(
    pool: &Pool,
    pull_id: &str,
    merged_by: &str,
    cross_referenced_issues: &Vec<String>,
    connected_issues: &Vec<String>,
) -> Result<(), Error> {
    let mut conn = pool.get_conn().await?;

    let cross_referenced_issues_json: Value = json!(cross_referenced_issues).into();
    let connected_issues_json: Value = json!(connected_issues).into();

    let query = r"UPDATE pull_requests 
                  SET merged_by = :merged_by, 
                      cross_referenced_issues = :cross_referenced_issues, 
                      connected_issues = :connected_issues
                  WHERE pull_id = :pull_id";

    let result = conn
        .exec_drop(
            query,
            params! {
                "pull_id" => pull_id,
                "merged_by" => merged_by,
                "cross_referenced_issues" => cross_referenced_issues_json,
                "connected_issues" => connected_issues_json,
            },
        )
        .await;

    result
}

/* pub async fn add_comment(
    pool: &PgPool,
    comment_id: &str,
    issue_id: &str,
    creator: &str,
    content: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO comments (comment_id, issue_id, creator, content)
        VALUES ($1, $2, $3, $4)
        "#,
        comment_id,
        issue_id,
        creator,
        content,
    )
    .execute(pool)
    .await?;
    Ok(())
}
 */

#[cfg(test)]
mod tests {
    use super::*;
    use mysql_async::prelude::Queryable;
    use mysql_async::Pool;

    #[tokio::test]
    async fn test_list_projects() {
        let pool = get_pool().await;
        let projects = list_projects(&pool).await.unwrap();

        println!("projects: {:?}", projects);

        assert_eq!(projects[1].2, vec!["issue3", "issue4"]);
    }

    #[tokio::test]
    async fn test_update_pull_request() {
        let pool = get_pool().await;

        let pull_id = "https://github.com/test/test/pull/4";
        let merged_by = "test_updated";
        let cross_referenced_issues = vec!["https://github.com/test/test/issues/5".to_string()];
        let connected_issues = vec!["https://github.com/test/test/issues/6".to_string()];

        // Update a pull request
        let result = update_pull_request(
            &pool,
            pull_id,
            merged_by,
            &cross_referenced_issues,
            &connected_issues,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_pull_request() {
        let pool = get_pool().await;

        let pull_id = "https://github.com/test/test/pull/4";
        let title = "Test Pull Request 2";
        let author = "test2";
        let repository = "https://github.com/test/test2";
        let merged_by = "test2";

        // Add a pull request
        let result = add_pull_request(&pool, pull_id, title, author, repository, merged_by).await;
        assert!(result.is_ok());

        // Check if the pull request exists
        let exists = pull_request_exists(&pool, pull_id).await.unwrap();
        assert!(exists);
    }
    // #[tokio::test]
    // async fn test_update_issue() {
    //     let pool = get_pool().await;
    //

    //     let issue_id = "https://github.com/test/test/issues/3";
    //     let project_id = "https://github.com/test/test13";
    //     let title = "Test Issue";
    //     let description = "This is a test issue.";

    //     // Add an issue first
    //     let add_result = add_issue(&pool, issue_id, project_id, title, description).await;
    //     println!("add_issue result: {:?}", add_result);

    //     // Update the issue
    //     let new_title = "Updated Test Issue";
    //     let new_description = "This is an updated test issue.";
    //     let update_result = update_issue(&pool, issue_id, new_title, new_description).await;
    //     println!("update_issue result: {:?}", update_result);
    // }

    // #[tokio::test]
    // async fn test_add_issue_checked() {
    //     let pool = get_pool().await;
    //

    //     let issue_id = "https://github.com/test/test/issues/4";
    //     let project_id = "https://github.com/test/test14";
    //     let title = "Test Issue Checked";
    //     let description = "This is a test issue for the checked function.";
    //     let repository_avatar = "https://avatars.githubusercontent.com/u/test?v=4";

    //     // Add an issue with checking
    //     let result = add_issue_checked(
    //         &pool,
    //         issue_id,
    //         project_id,
    //         title,
    //         description,
    //         repository_avatar,
    //     )
    //     .await;
    //     println!("add_issue_checked result: {:?}", result);
    // }

    #[tokio::test]
    async fn test_add_project() {
        let pool = get_pool().await;

        let project_id = "https://github.com/test/test15";

        let issue_id = "test_issue_id";
        let res = add_project(&pool, project_id, "test_logo", issue_id).await;
        println!("res: {:?}", res);
        // The project should now exist
        assert_eq!(project_exists(&pool, project_id).await.unwrap(), true);
    }

    // #[tokio::test]
    // async fn test_update_project() {
    //     let pool = get_pool().await;
    //

    //     let project_id = "https://github.com/test/test13";
    //     let project_logo = "https://avatars.githubusercontent.com/u/test?v=4";

    //     let new_issue_id = "https://github.com/test/test/issues/3"; // ensure new_issue_id is a valid JSON array

    //     let result = update_project(&pool, project_id, new_issue_id).await;
    //     println!("update_project result: {:?}", result);

    //     assert!(
    //         true,
    //         "Project's issues_list should contain the new issue_id after being updated"
    //     );
    // }
}
