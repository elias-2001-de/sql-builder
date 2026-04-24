mod common;
use common::*;
use sql_builder::*;

#[test]
fn count_star() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select::<(Count,)>()
        .build();
    assert_eq!(sql, "SELECT COUNT(*) FROM users");
}

#[test]
fn max_column() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(Max<posts::PostId>,)>()
        .build();
    assert_eq!(sql, "SELECT MAX(posts.PostId) FROM posts");
}

#[test]
fn min_column() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(Min<posts::PostId>,)>()
        .build();
    assert_eq!(sql, "SELECT MIN(posts.PostId) FROM posts");
}

#[test]
fn sum_column() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select::<(Sum<comments::AuthorId>,)>()
        .build();
    assert_eq!(sql, "SELECT SUM(comments.AuthorId) FROM comments");
}

#[test]
fn mixed_column_and_aggregate() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::AuthorId, Count)>()
        .build();
    assert_eq!(sql, "SELECT posts.AuthorId, COUNT(*) FROM posts");
}

#[test]
fn multiple_aggregates() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(Count, Max<posts::PostId>, Min<posts::PostId>)>()
        .build();
    assert_eq!(sql, "SELECT COUNT(*), MAX(posts.PostId), MIN(posts.PostId) FROM posts");
}

#[test]
fn aggregate_with_where() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select::<(Count,)>()
        .where_clause(WhereClause::<Users, _>::new().is_not_null::<users::Email>())
        .build();
    assert_eq!(sql, "SELECT COUNT(*) FROM users WHERE (Email IS NOT NULL)");
}
