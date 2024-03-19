CREATE TYPE review_status AS ENUM ('queue', 'approve', 'decline');

CREATE TABLE projects (
    project_id VARCHAR PRIMARY KEY,  -- url of a project repo
    project_logo VARCHAR NOT NULL,
    issues_list TEXT[]
);

CREATE TABLE issues (
    issue_id VARCHAR PRIMARY KEY,  -- url of an issue
        project_id VARCHAR NOT NULL,
    issue_title VARCHAR NOT NULL,
    issue_description TEXT NOT NULL,  -- description of the issue, could be truncated body text
    issue_budget INT,
    issue_assignee VARCHAR,
    issue_linked_pr VARCHAR,    -- url of the pull_request that closed the issue, if any, or the pull_request that is linked to the issue
    issue_status VARCHAR,    -- open, closed, in progress, or some signal of the status identified by the bot summarizing the issue's comments
    review_status review_status,
    issue_budget_approved BOOLEAN
);

CREATE TABLE comments (
    comment_id VARCHAR PRIMARY KEY,
    issue_id VARCHAR NOT NULL,
    creator VARCHAR NOT NULL,
    time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    content TEXT NOT NULL
);

CREATE TABLE pull_requests (
    pull_id VARCHAR PRIMARY KEY,  -- url of pull_request
    title VARCHAR NOT NULL,
    author VARCHAR NOT NULL,
    repository VARCHAR NOT NULL,
    merged_by VARCHAR NOT NULL,
    cross_referenced_issues TEXT[], 
    connected_issues TEXT[] 
);
