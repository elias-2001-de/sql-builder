#![allow(dead_code)]

use sql_builder::Table;

#[derive(Table)]
#[table_name = "users"]
pub struct Users {
    #[primary_key]
    #[column_name = "UserId"]
    pub user_id: i64,
    #[column_name = "UserName"]
    pub user_name: String,
    #[column_name = "Email"]
    pub email: Option<String>,
}

#[derive(Table)]
#[table_name = "posts"]
pub struct Posts {
    #[primary_key]
    #[column_name = "PostId"]
    pub post_id: i64,
    #[column_name = "Title"]
    pub title: String,
    #[foreign_key(Users)]
    #[column_name = "AuthorId"]
    pub author_id: i64,
    #[column_name = "Draft"]
    pub draft: Option<bool>,
}

#[derive(Table)]
#[table_name = "comments"]
pub struct Comments {
    #[primary_key]
    #[column_name = "CommentId"]
    pub comment_id: i64,
    #[column_name = "Body"]
    pub body: String,
    #[foreign_key(Posts)]
    #[column_name = "PostId"]
    pub post_id: i64,
    #[foreign_key(Users)]
    #[column_name = "AuthorId"]
    pub author_id: i64,
    #[foreign_key(Comments)]
    #[column_name = "ParentId"]
    pub parent_id: Option<i64>,
}
