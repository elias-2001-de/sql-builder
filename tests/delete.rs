mod common;
use common::*;
use sql_builder::*;
use sql_builder::DeleteBuilder;

#[test]
fn delete_with_where() {
    let sql = DeleteBuilder::new()
        .from::<Users>()
        .where_clause(WhereClause::<Users, _>::new().eq::<users::UserId, _>(7_i64))
        .build();
    assert_eq!(sql, "DELETE FROM users WHERE (UserId = 7)");
}

#[test]
fn delete_without_where() {
    let sql = DeleteBuilder::new().from::<Users>().build();
    assert_eq!(sql, "DELETE FROM users");
}

#[test]
fn delete_with_compound_where() {
    let sql = DeleteBuilder::new()
        .from::<Posts>()
        .where_clause(
            WhereClause::<Posts, _>::new()
                .is_not_null::<posts::Draft>()
                .and()
                .lt::<posts::PostId, _>(100_i64),
        )
        .build();
    assert_eq!(sql, "DELETE FROM posts WHERE (Draft IS NOT NULL AND PostId < 100)");
}

#[test]
fn delete_multiple_where_clauses_joined_with_and() {
    let sql = DeleteBuilder::new()
        .from::<Users>()
        .where_clause(WhereClause::<Users, _>::new().gt::<users::UserId, _>(50_i64))
        .where_clause(WhereClause::<Users, _>::new().is_null::<users::Email>())
        .build();
    assert_eq!(sql, "DELETE FROM users WHERE (UserId > 50) AND (Email IS NULL)");
}
