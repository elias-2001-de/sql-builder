mod common;
use common::*;
use sql_builder::*;

#[test]
fn count_star() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Users>()
        .select::<(Count,)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT COUNT(*) FROM users");
}

#[test]
fn max_column() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Posts>()
        .select::<(Max<posts::PostId>,)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT MAX(posts.PostId) FROM posts");
}

#[test]
fn min_column() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Posts>()
        .select::<(Min<posts::PostId>,)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT MIN(posts.PostId) FROM posts");
}

#[test]
fn sum_column() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Comments>()
        .select::<(Sum<comments::AuthorId>,)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT SUM(comments.AuthorId) FROM comments");
}

#[test]
fn mixed_column_and_aggregate() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::AuthorId, Count)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT posts.AuthorId, COUNT(*) FROM posts");
}

#[test]
fn multiple_aggregates() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Posts>()
        .select::<(Count, Max<posts::PostId>, Min<posts::PostId>)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT COUNT(*), MAX(posts.PostId), MIN(posts.PostId) FROM posts");
}

#[test]
fn aggregate_with_where() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Users>()
        .select::<(Count,)>()
        .where_clause(WhereClause::<Users, _>::new().is_not_null::<users::Email>())
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT COUNT(*) FROM users WHERE (Email IS NOT NULL)");
}
