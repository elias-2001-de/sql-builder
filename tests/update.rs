mod common;
use common::*;
use sql_builder::*;
use sql_builder::UpdateBuilder;

#[test]
fn update_single_column_with_where() {
    let runner = StringRunner::new();
    UpdateBuilder::new()
        .table::<Users>()
        .set::<users::UserName, _>("Bob")
        .where_clause(WhereClause::<Users, _>::new().eq::<users::UserId, _>(1_i64))
        .execute(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "UPDATE users SET UserName = 'Bob' WHERE (UserId = 1)");
}

#[test]
fn update_multiple_columns_with_where() {
    let runner = StringRunner::new();
    UpdateBuilder::new()
        .table::<Users>()
        .set::<users::UserName, _>("Carol")
        .set::<users::UserId, _>(99_i64)
        .where_clause(WhereClause::<Users, _>::new().lt::<users::UserId, _>(10_i64))
        .execute(&runner)
        .unwrap();
    assert_eq!(
        runner.query().unwrap(),
        "UPDATE users SET UserName = 'Carol', UserId = 99 WHERE (UserId < 10)"
    );
}

#[test]
fn update_without_where() {
    let runner = StringRunner::new();
    UpdateBuilder::new()
        .table::<Posts>()
        .set::<posts::Title, _>("Untitled")
        .execute(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "UPDATE posts SET Title = 'Untitled'");
}

#[test]
fn update_set_null_nullable_column() {
    let runner = StringRunner::new();
    UpdateBuilder::new()
        .table::<Users>()
        .set::<users::UserName, _>("Dave")
        .set_null::<users::Email>()
        .where_clause(WhereClause::<Users, _>::new().eq::<users::UserId, _>(5_i64))
        .execute(&runner)
        .unwrap();
    assert_eq!(
        runner.query().unwrap(),
        "UPDATE users SET UserName = 'Dave', Email = NULL WHERE (UserId = 5)"
    );
}
