mod common;
use common::*;
use sql_builder::{InsertBuilder, StringRunner};

#[test]
fn insert_single_column() {
    let runner = StringRunner::new();
    InsertBuilder::new()
        .into_table::<Users>()
        .value::<users::UserName, _>("Alice")
        .execute(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "INSERT INTO users (UserName) VALUES ('Alice')");
}

#[test]
fn insert_multiple_columns() {
    let runner = StringRunner::new();
    InsertBuilder::new()
        .into_table::<Users>()
        .value::<users::UserName, _>("Alice")
        .value::<users::UserId, _>(42_i64)
        .execute(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "INSERT INTO users (UserName, UserId) VALUES ('Alice', 42)");
}

#[test]
fn insert_bool_value() {
    let runner = StringRunner::new();
    InsertBuilder::new()
        .into_table::<Posts>()
        .value::<posts::Title, _>("Hello")
        .value::<posts::AuthorId, _>(1_i64)
        .execute(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "INSERT INTO posts (Title, AuthorId) VALUES ('Hello', 1)");
}

#[test]
fn insert_string_with_single_quote_escaped() {
    let runner = StringRunner::new();
    InsertBuilder::new()
        .into_table::<Users>()
        .value::<users::UserName, _>("O'Brien")
        .execute(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "INSERT INTO users (UserName) VALUES ('O''Brien')");
}
