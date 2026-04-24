mod common;
use common::*;
use sql_builder::InsertBuilder;

#[test]
fn insert_single_column() {
    let sql = InsertBuilder::new()
        .into_table::<Users>()
        .value::<users::UserName, _>("Alice")
        .build();
    assert_eq!(sql, "INSERT INTO users (UserName) VALUES ('Alice')");
}

#[test]
fn insert_multiple_columns() {
    let sql = InsertBuilder::new()
        .into_table::<Users>()
        .value::<users::UserName, _>("Alice")
        .value::<users::UserId, _>(42_i64)
        .build();
    assert_eq!(sql, "INSERT INTO users (UserName, UserId) VALUES ('Alice', 42)");
}

#[test]
fn insert_bool_value() {
    let sql = InsertBuilder::new()
        .into_table::<Posts>()
        .value::<posts::Title, _>("Hello")
        .value::<posts::AuthorId, _>(1_i64)
        .build();
    assert_eq!(sql, "INSERT INTO posts (Title, AuthorId) VALUES ('Hello', 1)");
}

#[test]
fn insert_string_with_single_quote_escaped() {
    let sql = InsertBuilder::new()
        .into_table::<Users>()
        .value::<users::UserName, _>("O'Brien")
        .build();
    assert_eq!(sql, "INSERT INTO users (UserName) VALUES ('O''Brien')");
}
