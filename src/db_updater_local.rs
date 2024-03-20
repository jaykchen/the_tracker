use anyhow::Result;
use dotenv::dotenv;
use mysql_async::Error;
pub use mysql_async::*;
use mysql_async::{prelude::*, Pool};





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
    let issue_id_json: Value = serde_json::json!(issue_id).into();

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

    let issue_id_json: Value = serde_json::json!(issue_id).into();

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

/* pub async fn add_project_checked(
    pool: &Pool,
    project_id: &str,
    project_logo: &str,
    issue_id: &str,
) -> Result<()> {
    if project_exists(pool, project_id).await? {
        update_project(pool, project_id, issue_id).await?;
    } else {
        add_project(pool, project_id, project_logo, issue_id).await?;
    }

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
 */

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

/* pub async fn list_issues(pool: &PgPool, project_id: &str) -> anyhow::Result<()> {
    let recs = sqlx::query!(
        r#"
        SELECT issue_id, issue_title, issue_description, issue_budget
        FROM issues
        WHERE project_id = $1
        ORDER BY issue_id
        "#,
        project_id
    )
    .fetch_all(pool)
    .await?;

    for rec in recs {
        println!(
            "- [{}] {}: {} (${:?})",
            rec.issue_id, rec.issue_title, rec.issue_description, rec.issue_budget
        );
    }

    Ok(())
}

pub async fn get_issue(pool: &PgPool, issue_id: &str) -> anyhow::Result<()> {
    let recs = sqlx::query!(
        r#"
        SELECT issue_id, issue_title, issue_description, issue_budget
        FROM issues
        WHERE issue_id = $1
        ORDER BY issue_id
        "#,
        issue_id
    )
    .fetch_all(pool)
    .await?;

    for rec in recs {
        println!(
            "- [{}] {}: {} (${:?})",
            rec.issue_id, rec.issue_title, rec.issue_description, rec.issue_budget
        );
    }

    Ok(())
}

pub async fn add_comment(
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
/* pub async fn add_comment_checked(
    pool: &PgPool,
    comment_id: &str,
    issue_id: &str,
    creator: &str,
    content: &str,
) -> anyhow::Result<()> {
    if issue_exists(pool, issue_id).await? {
    } else {
        let _ = add_issue_checked(pool, issue_id, "title", "description").await?;
    }

    sqlx::query!(
        r#"
            INSERT INTO comments (comment_id, issue_id, creator, content)
            VALUES ($1, $2, $3, $4)
            "#,
        comment_id,
        issue_id,
        creator,
        content
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn add_comment_test_1(pool: &PgPool) -> anyhow::Result<()> {
    let comment_id = "https://github.com/jaykchen/issue-labeler/issues/24#issuecomment-1979927212";
    let issue_id = "https://github.com/jaykchen/issue-labeler/issues/24";
    let creator = "jaykchen";
    let content = "This is a placeholder comment on this issue.";
    sqlx::query!(
        r#"
        INSERT INTO comments (comment_id, issue_id, creator, content)
        VALUES ($1, $2, $3, $4)
        "#,
        comment_id,
        issue_id,
        creator,
        content
    )
    .execute(pool)
    .await?;
    Ok(())
}
 pub async fn list_comments(pool: &PgPool, issue_id: &str) -> anyhow::Result<()> {
    let recs = sqlx::query!(
        r#"
        SELECT comment_id, content
        FROM comments
        WHERE issue_id = $1
        ORDER BY comment_id
        "#,
        issue_id
    )
    .fetch_all(pool)
    .await?;

    for rec in recs {
        println!("- [{}] {}", rec.comment_id, rec.content);
    }

    Ok(())
}

pub async fn list_pull_requests(
    pool: &sqlx::PgPool,
) -> anyhow::Result<
    Vec<(
        String,
        String,
        String,
        String,
        String,
        Vec<String>,
        Vec<String>,
    )>,
> {
    let pull_requests = sqlx::query!(
        r#"
        SELECT pull_id, title, author, repository, merged_by, cross_referenced_issues, connected_issues
        FROM pull_requests
        "#
    )
    .fetch_all(pool)
    .await?
    .iter()
    .map(|r| {
        (
            r.pull_id.clone(),
            r.title.clone(),
            r.author.clone(),
            r.repository.clone(),
            r.merged_by.clone(),
            r.cross_referenced_issues.clone().unwrap_or_default(), // Handle Option<Vec<String>>
            r.connected_issues.clone().unwrap_or_default(), // Handle Option<Vec<String>>
        )
    })
    .collect();

    Ok(pull_requests)
}
pub async fn pull_request_exists(pool: &sqlx::PgPool, pull_id: &str) -> anyhow::Result<bool> {
    let exists = sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM pull_requests WHERE pull_id = $1)
        "#,
        pull_id
    )
    .fetch_one(pool)
    .await?
    .exists
    .unwrap_or(false);

    Ok(exists)
}
pub async fn add_pull_request_checked(
    pool: &sqlx::PgPool,
    pull_id: &str,
    title: &str,
    author: &str,
    repository: &str,
    merged_by: &str,
    cross_referenced_issues: &Vec<String>,
    connected_issues: &Vec<String>,
) -> anyhow::Result<()> {
    let exists = pull_request_exists(pool, pull_id).await?;

    if !exists {
        add_pull_request(
            pool,
            pull_id,
            title,
            author,
            repository,
            merged_by,
            cross_referenced_issues,
            connected_issues,
        )
        .await?;
    }

    Ok(())
}

pub async fn add_pull_request(
    pool: &PgPool,
    pull_id: &str,
    title: &str,
    author: &str,
    repository: &str,
    merged_by: &str,
    cross_referenced_issues: &Vec<String>,
    connected_issues: &Vec<String>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO pull_requests (pull_id, title, author, repository, merged_by, cross_referenced_issues, connected_issues)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        pull_id,
        title,
        author,
        repository,
        merged_by,
        cross_referenced_issues,
        connected_issues,
    )
    .execute(pool)
    .await?;
    Ok(())
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use mysql_async::prelude::Queryable;
    use mysql_async::Pool;
    

    #[async_trait]
    trait TestDbSetup {
        async fn setup_db(&self);
    }

    #[async_trait]
    impl TestDbSetup for Pool {
        async fn setup_db(&self) {
            let mut conn = self.get_conn().await.unwrap();
            conn.query_drop("CREATE DATABASE IF NOT EXISTS test_db")
                .await
                .unwrap();
            conn.query_drop("USE test_db").await.unwrap();
            conn.query_drop(
                "CREATE TABLE IF NOT EXISTS projects (
                    project_id VARCHAR(255) PRIMARY KEY,
                    project_logo VARCHAR(255) NOT NULL,
                    issues_list JSON
                )",
            )
            .await
            .unwrap();
        }
    }

    // #[tokio::test]
    // async fn test_update_issue() {
    //     let pool = get_pool().await;
    //     pool.setup_db().await;

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

    #[tokio::test]
    async fn test_add_issue_checked() {
        let pool = get_pool().await;
        pool.setup_db().await;

        let issue_id = "https://github.com/test/test/issues/4";
        let project_id = "https://github.com/test/test14";
        let title = "Test Issue Checked";
        let description = "This is a test issue for the checked function.";
        let repository_avatar = "https://avatars.githubusercontent.com/u/test?v=4";

        // Add an issue with checking
        let result = add_issue_checked(
            &pool,
            issue_id,
            project_id,
            title,
            description,
            repository_avatar,
        )
        .await;
        println!("add_issue_checked result: {:?}", result);
    }

    //     #[tokio::test]
    //     async fn test_project_exists() {
    //         let pool = get_pool().await;
    //         pool.setup_db().await;
    //         let project_id = "https://github.com/test/test13";

    //         // // Add a project
    //         // add_project(&pool, project_id, "test_logo", "test_issue_id")
    //         //     .await
    //         //     .unwrap();

    //         // Now the project should exist
    //         assert_eq!(project_exists(&pool, project_id).await.unwrap(), true);
    //     }

    //     #[tokio::test]
    //     async fn test_add_project() {
    //         let pool = get_pool().await;
    //         pool.setup_db().await;
    //         let project_id = "https://github.com/test/test15";

    //         let issue_id= "test_issue_id";
    //      let res =   add_project(&pool, project_id, "test_logo", issue_id)
    //             .await;
    // println!("res: {:?}", res);
    //         // The project should now exist
    //         assert_eq!(project_exists(&pool, project_id).await.unwrap(), true);
    //     }

    // #[tokio::test]
    // async fn test_update_project() {
    //     let pool = get_pool().await;
    //     pool.setup_db().await;

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
