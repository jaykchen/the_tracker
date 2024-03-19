/* use sqlx::postgres::PgPool;

pub async fn project_exists(pool: &PgPool, project_id: &str) -> anyhow::Result<bool> {
    let exists = sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM projects WHERE project_id = $1) AS "exists!"
        "#,
        project_id
    )
    .fetch_one(pool)
    .await?
    .exists;

    Ok(exists)
}
pub async fn add_project(
    pool: &PgPool,
    project_id: &str,
    project_logo: &str,
    issue_id: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO projects (project_id, project_logo, issues_list)
        VALUES ($1, $2, ARRAY[$3]::text[])
        "#,
        project_id,
        project_logo,
        issue_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
pub async fn add_project_checked(
    pool: &PgPool,
    project_id: &str,
    project_logo: &str,
    issue_id: &str,
) -> anyhow::Result<()> {
    if project_exists(pool, project_id).await? {
        update_project(pool, project_id, issue_id).await?;
    } else {
        add_project(pool, project_id, project_logo, issue_id).await?;
    }

    Ok(())
}

pub async fn update_project(pool: &PgPool, project_id: &str, issue_id: &str) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE projects
        SET issues_list = array_append(issues_list, $1)
        WHERE project_id = $2
        "#,
        issue_id,
        project_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn add_project_test_1(pool: &PgPool) -> anyhow::Result<()> {
    let project_id = "https://github.com/jaykchen/issue-labeler";
    let issue_id = "https://github.com/jaykchen/issue-labeler/issues/24";
    let project_logo = "https://avatars.githubusercontent.com/u/112579101?v=4";

    let _ = add_project(pool, project_id, project_logo, issue_id).await?;

    Ok(())
}

pub async fn list_projects(pool: &PgPool) -> anyhow::Result<Vec<(String, String, Vec<String>)>> {
    let recs = sqlx::query!(
        r#"
        SELECT project_id, project_logo, issues_list
        FROM projects
        ORDER BY project_id
        "#
    )
    .fetch_all(pool)
    .await?;

    let projects = recs
        .iter()
        .map(|r| {
            (
                r.project_id.clone(),
                r.project_logo.clone(),
                r.issues_list.clone().unwrap_or_default(), // Handle Option<Vec<String>>
            )
        })
        .collect();

    Ok(projects)
}
pub async fn issue_exists(pool: &PgPool, issue_id: &str) -> anyhow::Result<bool> {
    let exists = sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM issues WHERE issue_id = $1) AS "exists!"
        "#,
        issue_id
    )
    .fetch_one(pool)
    .await?
    .exists;

    Ok(exists)
}

pub async fn add_issue_checked(
    pool: &PgPool,
    issue_id: &str,
    project_id: &str,
    title: &str,
    description: &str,
    repository_avatar: &str,
) -> anyhow::Result<()> {
    if project_exists(pool, project_id).await? {
        update_project(pool, project_id, issue_id).await?;
    } else {
        add_project(pool, project_id, repository_avatar, issue_id).await?;
    }

    if issue_exists(pool, issue_id).await? {
    } else {
        add_issue(pool, issue_id, project_id, title, description).await?;
    }
    Ok(())
}

pub async fn add_issue(
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

pub async fn add_issue_test_1(pool: &PgPool) -> anyhow::Result<()> {
    let issue_id = "https://github.com/jaykchen/issue-labeler/issues/24";
    let project_id = "https://github.com/jaykchen/issue-labeler";
    let title = "WASI-NN with GPU on Jetson Orin Nano";
    let description = "demo";

    let _ = add_issue(pool, issue_id, project_id, title, description).await?;
    Ok(())
}

pub async fn list_issues(pool: &PgPool, project_id: &str) -> anyhow::Result<()> {
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
*/
/* pub async fn list_comments(pool: &PgPool, issue_id: &str) -> anyhow::Result<()> {
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
/* pub async fn list_pull_requests(
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
 */