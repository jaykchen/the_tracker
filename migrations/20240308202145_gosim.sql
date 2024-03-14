CREATE TYPE review_status AS ENUM ('queue', 'approve', 'decline');

CREATE TABLE projects (
    project_id VARCHAR(255) PRIMARY KEY,
    project_logo VARCHAR(255) NOT NULL
);

CREATE TABLE issues (
    issue_id VARCHAR(255) PRIMARY KEY,
        project_id VARCHAR(255) NOT NULL,
    issue_title VARCHAR(255) NOT NULL,
    issue_description TEXT NOT NULL,
    issue_budget DECIMAL(10, 2),
    issue_assignee VARCHAR(255),
    issue_linked_pr VARCHAR(255),
    issue_status VARCHAR(50),
    review_status review_status,
    issue_budget_approved BOOLEAN,
    FOREIGN KEY (project_id) REFERENCES projects(project_id)
);

CREATE TABLE comments (
    comment_id VARCHAR(255) PRIMARY KEY,
    issue_id VARCHAR(255) NOT NULL,
    creator VARCHAR(50) NOT NULL,
    time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    content TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(issue_id)
);

CREATE TABLE pull_requests (
    pull_id VARCHAR(255) PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    author VARCHAR(50) NOT NULL,
    repository VARCHAR(255) NOT NULL,
    merged_by VARCHAR(50) NOT NULL,
    cross_referenced_issues VARCHAR(255)[] -- Array of issue URLs
);
