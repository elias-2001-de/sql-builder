mod common;
use common::*;
use sql_builder::*;

#[test]
fn where_clause_eq() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_clause(WhereClause::<Users, _>::new().eq::<users::UserId, _>(1_i64))
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE (UserId = 1)");
}

#[test]
fn where_clause_gt_and_lt() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(
            WhereClause::<Posts, _>::new()
                .gt::<posts::PostId, _>(10_i64)
                .and()
                .lt::<posts::PostId, _>(50_i64),
        )
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE (PostId > 10 AND PostId < 50)");
}

#[test]
fn where_clause_not_eq_and_lt_eq() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_clause(
            WhereClause::<Users, _>::new()
                .not_eq::<users::UserId, _>(0_i64)
                .and()
                .lt_eq::<users::UserId, _>(99_i64),
        )
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE (UserId <> 0 AND UserId <= 99)");
}

#[test]
fn where_clause_gt_eq() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(WhereClause::<Posts, _>::new().gt_eq::<posts::PostId, _>(100_i64))
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE (PostId >= 100)");
}

#[test]
fn where_clause_like() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_clause(WhereClause::<Users, _>::new().like::<users::UserName, _>("%alice%"))
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE (UserName LIKE '%alice%')");
}

#[test]
fn where_clause_is_null() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_clause(WhereClause::<Users, _>::new().is_null::<users::Email>())
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE (Email IS NULL)");
}

#[test]
fn where_clause_is_not_null() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(WhereClause::<Posts, _>::new().is_not_null::<posts::Draft>())
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE (Draft IS NOT NULL)");
}

#[test]
fn where_clause_is_not_null_on_nullable_fk() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select_all()
        .where_clause(WhereClause::<Comments, _>::new().is_not_null::<comments::ParentId>())
        .build();
    assert_eq!(sql, "SELECT * FROM comments WHERE (ParentId IS NOT NULL)");
}

#[test]
fn where_clause_and_chain() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_clause(
            WhereClause::<Users, _>::new()
                .eq::<users::UserId, _>(1_i64)
                .and()
                .like::<users::UserName, _>("%alice%"),
        )
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE (UserId = 1 AND UserName LIKE '%alice%')");
}

#[test]
fn where_clause_or_chain() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_clause(
            WhereClause::<Users, _>::new()
                .lt::<users::UserId, _>(10_i64)
                .or()
                .gt_eq::<users::UserId, _>(100_i64),
        )
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE (UserId < 10 OR UserId >= 100)");
}

#[test]
fn where_clause_between() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_clause(WhereClause::<Users, _>::new().between::<users::UserId, _>(1_i64, 50_i64))
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE (UserId BETWEEN 1 AND 50)");
}

#[test]
fn where_clause_in_values() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_clause(WhereClause::<Users, _>::new().in_values::<users::UserId, _>([1_i64, 2, 3]))
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE (UserId IN (1, 2, 3))");
}

#[test]
fn where_clause_multiple_calls_and_joined() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(WhereClause::<Posts, _>::new().gt::<posts::PostId, _>(5_i64))
        .where_clause(WhereClause::<Posts, _>::new().is_null::<posts::Draft>())
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE (PostId > 5) AND (Draft IS NULL)");
}

#[test]
fn where_clause_long_chain() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_clause(
            WhereClause::<Users, _>::new()
                .gt_eq::<users::UserId, _>(1_i64)
                .and()
                .like::<users::UserName, _>("B%")
                .and()
                .is_not_null::<users::Email>(),
        )
        .build();
    assert_eq!(
        sql,
        "SELECT * FROM users WHERE (UserId >= 1 AND UserName LIKE 'B%' AND Email IS NOT NULL)"
    );
}
