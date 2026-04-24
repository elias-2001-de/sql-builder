mod common;
use common::*;
use sql_builder::*;

#[test]
fn group_by_single_column() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::AuthorId, Count)>()
        .group_by::<posts::AuthorId>()
        .build();
    assert_eq!(sql, "SELECT posts.AuthorId, COUNT(*) FROM posts GROUP BY AuthorId");
}

#[test]
fn group_by_multiple_columns() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select::<(comments::PostId, comments::AuthorId, Count)>()
        .group_by::<comments::PostId>()
        .group_by::<comments::AuthorId>()
        .build();
    assert_eq!(
        sql,
        "SELECT comments.PostId, comments.AuthorId, COUNT(*) FROM comments GROUP BY PostId, AuthorId"
    );
}

#[test]
fn group_by_with_having() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::AuthorId, Count)>()
        .group_by::<posts::AuthorId>()
        .having(WhereClause::<Posts, _>::new().gt::<posts::PostId, _>(2_i64))
        .build();
    assert_eq!(
        sql,
        "SELECT posts.AuthorId, COUNT(*) FROM posts GROUP BY AuthorId HAVING (PostId > 2)"
    );
}

#[test]
fn group_by_with_where_and_having() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::AuthorId, Count)>()
        .where_clause(WhereClause::<Posts, _>::new().is_not_null::<posts::Draft>())
        .group_by::<posts::AuthorId>()
        .having(WhereClause::<Posts, _>::new().gt::<posts::PostId, _>(1_i64))
        .build();
    assert_eq!(
        sql,
        "SELECT posts.AuthorId, COUNT(*) FROM posts WHERE (Draft IS NOT NULL) GROUP BY AuthorId HAVING (PostId > 1)"
    );
}

#[test]
fn group_by_with_order_and_limit() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::AuthorId, Count)>()
        .group_by::<posts::AuthorId>()
        .order_by::<posts::AuthorId>(Direction::Desc)
        .limit(10)
        .build();
    assert_eq!(
        sql,
        "SELECT posts.AuthorId, COUNT(*) FROM posts GROUP BY AuthorId ORDER BY AuthorId DESC LIMIT 10"
    );
}

#[test]
fn having_with_compound_condition() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select::<(users::UserId, Count)>()
        .group_by::<users::UserId>()
        .having(
            WhereClause::<Users, _>::new()
                .gt::<users::UserId, _>(0_i64)
                .and()
                .lt::<users::UserId, _>(100_i64),
        )
        .build();
    assert_eq!(
        sql,
        "SELECT users.UserId, COUNT(*) FROM users GROUP BY UserId HAVING (UserId > 0 AND UserId < 100)"
    );
}
