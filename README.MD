## The Project Tracker

Give me suggestion on how to manage the execution flow of this program, according Predefined schedule.

There are a few key steps:
Item 1, Search_issue_open,  It means that at the beginning of the program run The function to search Github For issues that matches Project conditions. Initial issues information is saved in database;
 Item 2, search_overall_pull_requests function search on Github For those that fit The project conditions. Pull request information is saved to database. And some of the information is related to the issues. So the issues related to the requests are updated;
  Item 3, search_issue_close function will search the Github. on issues That fit the project conditions. This is now close to the end of the project. Issues closed, with pull_request merged that addressed the issue will be tagged as closed. When found data Is so
  Item 4, search_issues_close function shall search  issues On Github repo by repo out of the repos logged in database, Search all issues on the repo that fits the project conditions. This is to cross check data about issues and their connected pull_request are correct.

  Additionally, there are two Layers of logic. The first layer is to Get data from Github to populate the database like described above. The second layer is human administrator will monitor over the projects and approve or reject Projects, which is identified per issue.

   I need a logic to control the program flow. Give me a flow design in Ascii text diagram.

   there is constant loop, 
   Program will search Github daily to update the database, Human administ Will monitor over To database. And alter status Information of issues/which corresponds to projects on the database daily. Next, Search_overall_pull_request function Get pull requests information. Which will match the issues already saved in the previous step, Next search_issues_closed Function will search issues for New comments, and progream will use AI to Identify Problematic situation on the issue. Program will flag the issue when extraordinary situation arise. Lastly after human administrator intervention and the search_issues_per_repo repo by repo. The information about issues and their connected pool requests are cross checked and the final result can be obtained


+---------------------+
        | Start               |
        +---------------------+
                 |
                 v
        +---------------------+     +------------------------------+
        | Hourly Loop Trigger |---->| (Action) Hourly Collection   |
        +---------------------+     +------------------------------+
                 |                              |
                 |                              v
                 |                 +---------------------------------------+
                 |                 | search_issues_w_update_comments       |
                 |                 | identify issues with potential problems by filtering comments with AI, flag them in db if found       |
                 |                 +---------------------------------------+
                 |                              |
                 |                              v
                 |                 +---------------------------+
                 |<----------------| Wait until next hour      |
                 |                 +---------------------------+
                 |
                 v
        +---------------------+     +------------------------------+
        | Daily Loop Trigger  |---->| (Action) Daily Collection    |
        +---------------------+     +------------------------------+
                 |                              |
                 |                              v
                 |                 +---------------------------------------+
                 |                 | search_issues_open                    |
                 |                 | record new issues meeting all project conditions                    |
                 |                 +---------------------------------------+
                 |                              |
                 |                              v
                 |                 +---------------------------------------+
                 |                 | overall_search_pull_requests          |
                 |                 | record pull_requests meeting all project conditions          |
                 |                 | to match pull_requests with issues in db           |
                 |                 +---------------------------------------+
                 |                              |
                 |                              v
                 |                 +---------------------------------------+
                 |                 | search_issues_closed                  |
                 |                 | record issues with closed status                   |
                 |                 | flag them in db, ready for admin review                   |
                 |                 +---------------------------------------+
                 |                              |
                 |                              v
                 |                 +---------------------------------------+
                 |                 | get_per_repo_pull_requests            |
                 |                 | double check previously chosen issues have proper pull_requests to address them            |
                 |                 | ready for final admin approval            |
                 |                 +---------------------------------------+
                 |                              |
                 |                 +------------------------------+
                 |<----------------| (Manual) Human Intervention |
                 |                 | react to issues flaged  |
                 |                 | assign issue_budget when admin sees an issue valid            |
                 |                 | approve to release issue_budget when issue and pull_request match, all conditions comply            |
                 |                 +------------------------------+
                 |                              |
                 |                              v
                 |                 +-------------------------+
                 |<----------------| End of Daily Loop       |
                 |                 +-------------------------+
                 |                              |
                 |                              v
                 |                 +-------------------------+
                 |<----------------| Wait until next cycle   |
                 |                 +-------------------------+
                 |                              |
                 |                              v
                 |<----------------| Repeat the Daily Loop   |
                                   +-------------------------+

**Explanatory Notes for Functions Based on Flowchart:**

- **search_issues_w_update_comments:**
  - **Location in Chart:** Layer 1: Hourly Collection.
  - **Purpose:** This function is responsible for finding issues that have been updated with comments within the last hour.
  - **Query File:** `search_issues_w_update_comments.gql`.
  - **Execution:** Runs every hour to ensure up-to-date comment tracking.

- **search_issues_open:**
  - **Location in Chart:** Layer 1: Daily Collection (search_issues_open).
  - **Purpose:** Gathers all issues that are currently open.
  - **Query File:** `search_issues_open.gql`.
  - **Execution:** Invoked once a day to compile active issues for daily reporting.

- **overall_search_pull_requests:**
  - **Location in Chart:** Layer 1: Daily Collection (overall_search_pull_requests).
  - **Purpose:** Searches for pull requests across the entire dataset.
  - **Query File:** `search_pull_requests_overall.gql`.
  - **Execution:** Carried out daily to provide a holistic view of pull requests.

- **search_issues_closed:**
  - **Location in Chart:** Layer 1: Daily Collection (search_issues_closed).
  - **Purpose:** Finds all issues that were closed within the last day.
  - **Query File:** `search_issues_closed.gql`.
  - **Execution:** Conducted daily to track the volume and status of resolved issues.

- **get_per_repo_pull_requests:**
  - **Location in Chart:** Layer 1: Daily Collection (get_per_repo_pull_requests).
  - **Purpose:** Retrieves pull request data for each individual repository.
  - **Query File:** `search_pull_requests_per_repo.gql`.
  - **Execution:** Performed daily to analyze pull request activities per repository.