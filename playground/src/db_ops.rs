use crate::db_updater_local::*;
use sqlx::postgres::PgPool;

pub async fn approve_project_per_issue(
    pool: &PgPool,
    issue_id: &str,
    issue_budget: i32,
    issue_budget_approved: bool, // Assuming this is the correct type for your "approved" column
) -> anyhow::Result<()> {
    let rec = sqlx::query!(
        r#"
        UPDATE issues
        SET issue_budget = $2, issue_budget_approved = $3
        WHERE issue_id = $1
        "#,
        issue_id,
        issue_budget,
        issue_budget_approved
    )
    .execute(pool)
    .await?;

    if rec.rows_affected() == 0 {
        // Handle the case where no rows were updated, which could be due to a non-existent issue_id
        // This is optional and depends on your application's requirements
    }

    Ok(())
}

pub async fn pr_pulled_per_issue(
    pool: &PgPool,
    issue_id: &str,
    issue_assignee: &str,
    issue_linked_pr: &str,
    issue_status: &str,
    review_status: &str, // Ensure this matches your database schema type and Rust's type handling
) -> anyhow::Result<()> {
    let rec = sqlx::query!(
        r#"
        UPDATE issues
        SET issue_assignee = $2,
            issue_linked_pr = $3,
            issue_status = $4,
            review_status = $5
        WHERE issue_id = $1
        "#,
        issue_id,
        issue_assignee,
        issue_linked_pr,
        issue_status,
        review_status
    )
    .execute(pool)
    .await?;

    if rec.rows_affected() == 0 {
        // Handle the case where no rows were updated, which could be due to a non-existent issue_id
        // This is optional and depends on your application's requirements
    }

    Ok(())
}

//
pub async fn update_comments(pool: &PgPool, issue_id: &str) -> anyhow::Result<()> {
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
