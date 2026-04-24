mod common;
use common::*;
use sql_builder::*;

#[test]
fn select_all_produces_star() {
    let sql = QueryBuilder::new().from::<Users>().select_all().build();
    assert_eq!(sql, "SELECT * FROM users");
}

#[test]
fn select_single_column() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select::<(users::UserId,)>()
        .build();
    assert_eq!(sql, "SELECT users.UserId FROM users");
}

#[test]
fn select_multiple_columns() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select::<(users::UserId, users::UserName)>()
        .build();
    assert_eq!(sql, "SELECT users.UserId, users.UserName FROM users");
}

#[test]
fn select_three_columns() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select::<(comments::CommentId, comments::Body, comments::PostId)>()
        .build();
    assert_eq!(
        sql,
        "SELECT comments.CommentId, comments.Body, comments.PostId FROM comments"
    );
}
