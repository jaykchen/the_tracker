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
+---------------------+     +-----------------------------+
| Hourly Loop Begins  |---->| Layer 1: Hourly Collection  |
+---------------------+     +-----------------------------+
          |                           |
          |                           v
          |                 +---------------------------+
          |                 | search_issues_w_update_comments        |
          |                 | (Fetch & save issues with updated comments hourly) |  note: a search term that gets issues  
                                                                                    ntoe:  with intened labels, comments updated  within date range,  
          |                 | Flag issues with AI  |
          |                 +---------------------------+
          |                           |
          |                           v
          |                 +---------------------------+
          |<----------------| Wait until next hour      |
          |                 | (Repeat Hourly Loop)      |
          |                 +---------------------------+
          |
          v
+---------------------+     +-----------------------------+
| Daily Loop Begins   |---->| Layer 1: Daily Collection   |
+---------------------+     +-----------------------------+
          |                  
          |                 +---------------------------+
          |                 | search_issues_open        |
          |                 | (Fetch & save initial     |     note: a search term that gets issues with target labels in date range
          |                 |  open issues )      |

                                        |
                                        v
          |                 +---------------------------+
          |                 | search_pull_requests_overall |
          |                 | (Fetch pull requests,     |      note: pull_request with target labels, hoping to match issues already logged
          |                 |  update issues daily)     |
          |                 +---------------------------+
          |                           |
          |                           v
          |                 +---------------------------+
          |<----------------| search_issues_closed      |
          |                 | (Update closed issues     |    note: almost same conditions for issues, identifying those with closed status
          |                 |  daily)                   |
          |                 +---------------------------+
          |                           |
          |                           v
          |                 +---------------------------+
          |<----------------| search_pull_requests_per_repo|    note: a more granular search to walk the projects saved in the database, 
          |                 | (Cross-check data daily)  |    note: cross matching issues with pull_requests, hoping to get connections missed in previous steps
          |                 +---------------------------+
          |                           |
          |                 +-----------------------------+
          |---------------->| Layer 2: Human Intervention |
                            +-----------------------------+
                                      |
                                      v
                            +---------------------------+
                            | Admin monitors & alters   |
                            | status of issues/projects |   note: changed status of issues will cause program to handle differently
                            | daily                     |
                            +---------------------------+
                                      |
                                      v
                             +-------------------------+
                             | Daily Loop Ends         |
                             +-------------------------+
                                      |
                                      v
                             +-------------------------+
                             | Wait until next cycle   |
                             +-------------------------+
                                      |
                                      v
                               (Repeat the Daily Loop)