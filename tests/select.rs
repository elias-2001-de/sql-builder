mod common;
use common::*;
use sql_builder::*;

#[test]
fn select_all_produces_star() {
    let runner = StringRunner::new();
    QueryBuilder::new().from::<Users>().select_all().execute_all(&runner).unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT * FROM users");
}

#[test]
fn select_single_column() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Users>()
        .select::<(users::UserId,)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT users.UserId FROM users");
}

#[test]
fn select_multiple_columns() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Users>()
        .select::<(users::UserId, users::UserName)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT users.UserId, users.UserName FROM users");
}

#[test]
fn select_three_columns() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Comments>()
        .select::<(comments::CommentId, comments::Body, comments::PostId)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(
        runner.query().unwrap(),
        "SELECT comments.CommentId, comments.Body, comments.PostId FROM comments"
    );
}
