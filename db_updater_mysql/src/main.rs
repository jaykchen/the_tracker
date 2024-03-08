// AddProject {
//     name: String,
//     project_logo: String,
// },
// ListProjects,
// AddIssue {
//     project_id: i32,
//     title: String,
//     description: String,
//     budget: f64,
// },
// ListIssues {
//     project_id: i32,
// },
use mysql_async::{prelude::*, Error, Opts, Pool, QueryResult, Result};

use std::{env, io};

fn get_url() -> String {
    if let Ok(url) = env::var("DATABASE_URL") {
        let opts = Opts::from_url(&url).expect("DATABASE_URL invalid");
        if opts
            .db_name()
            .expect("a database name is required")
            .is_empty()
        {
            panic!("database name is empty");
        }
        url
    } else {
        "mysql://root:password@127.0.0.1:3307/mysql".into()
    }
}

 here are my errors
}

/* async fn add_project(pool: &PgPool, project_id: &str, project_logo: &str) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO projects (project_id, project_logo)
        VALUES ($1, $2)
        "#,
        project_id,
        project_logo
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn add_project_test_1(pool: &PgPool) -> anyhow::Result<()> {
    let project_id = "jaykchen/b-test";
    let project_logo = "https://avatars.githubusercontent.com/u/112579101?v=4";

    let _ = add_project(pool, project_id, project_logo).await?;

    Ok(())
}

async fn list_projects(pool: &PgPool) -> anyhow::Result<()> {
    let recs = sqlx::query!(
        r#"
        SELECT project_id, project_logo
        FROM projects
        ORDER BY project_id
        "#
    )
    .fetch_all(pool)
    .await?;

    for rec in recs {
        println!("{}: {}", rec.project_id, rec.project_logo);
    }

    Ok(())
}

async fn add_issue(
    pool: &PgPool,
    issue_id: &str,
    project_id: &str,
    title: &str,
    description: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO issues (issue_id, project_id, issue_title, issue_description)
        VALUES ($1, $2, $3, $4)
        "#,
        issue_id,
        project_id,
        title,
        description,
    )
    .execute(pool)
    .await?;
    Ok(())
}
async fn add_issue_test_1(pool: &PgPool) -> anyhow::Result<()> {
    let issue_id = "https://github.com/jaykchen/issue-labeler/issues/24";
    let project_id = "jaykchen/issue-labeler";
    let title = "WASI-NN with GPU on Jetson Orin Nano";
    let description = "demo";

    sqlx::query!(
        r#"
        INSERT INTO issues (issue_id, project_id, issue_title, issue_description)
        VALUES ($1, $2, $3, $4)
        "#,
        issue_id,
        project_id,
        title,
        description,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn list_issues(pool: &PgPool, project_id: &str) -> anyhow::Result<()> {
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

async fn add_comment(
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
        content
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn add_comment_test_1(pool: &PgPool) -> anyhow::Result<()> {
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

async fn list_comments(pool: &PgPool, issue_id: &str) -> anyhow::Result<()> {
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
*/
