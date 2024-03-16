// AddProject {
//     name: String,

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

// CREATE DATABASE projects;
// CREATE USER 'jaykchen'@'localhost' IDENTIFIED BY 'Sunday228';
// GRANT ALL PRIVILEGES ON projects.* TO 'jaykchen'@'localhost';
// FLUSH PRIVILEGES;
// EXIT;
