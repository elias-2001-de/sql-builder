mod common;
use common::*;
use sql_builder::*;

#[test]
fn inner_join_via_fk() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::Title, posts::AuthorId)>()
        .join::<Users, posts::AuthorId>()
        .build();
    assert_eq!(
        sql,
        "SELECT posts.Title, posts.AuthorId FROM posts \
         INNER JOIN users ON posts.AuthorId = users.UserId"
    );
}

#[test]
fn left_join_nullable_fk() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select_all()
        .left_join::<Comments, comments::ParentId>()
        .build();
    assert_eq!(
        sql,
        "SELECT * FROM comments \
         LEFT JOIN comments ON comments.ParentId = comments.CommentId"
    );
}

#[test]
fn multiple_joins() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select_all()
        .join::<Posts, comments::PostId>()
        .join::<Users, comments::AuthorId>()
        .build();
    assert_eq!(
        sql,
        "SELECT * FROM comments \
         INNER JOIN posts ON comments.PostId = posts.PostId \
         INNER JOIN users ON comments.AuthorId = users.UserId"
    );
}

#[test]
fn join_with_where_clause() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::Title, posts::AuthorId)>()
        .join::<Users, posts::AuthorId>()
        .where_clause(WhereClause::<Posts, _>::new().gt::<posts::PostId, _>(100_i64))
        .build();
    assert_eq!(
        sql,
        "SELECT posts.Title, posts.AuthorId FROM posts \
         INNER JOIN users ON posts.AuthorId = users.UserId \
         WHERE (PostId > 100)"
    );
}

#[test]
fn join_with_order_and_limit() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .join::<Users, posts::AuthorId>()
        .order_by::<posts::PostId>(Direction::Desc)
        .limit(5)
        .build();
    assert_eq!(
        sql,
        "SELECT * FROM posts \
         INNER JOIN users ON posts.AuthorId = users.UserId \
         ORDER BY PostId DESC LIMIT 5"
    );
}

#[test]
fn self_join_with_is_null_where() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select::<(comments::CommentId, comments::Body)>()
        .left_join::<Comments, comments::ParentId>()
        .where_clause(WhereClause::<Comments, _>::new().is_null::<comments::ParentId>())
        .build();
    assert_eq!(
        sql,
        "SELECT comments.CommentId, comments.Body FROM comments \
         LEFT JOIN comments ON comments.ParentId = comments.CommentId \
         WHERE (ParentId IS NULL)"
    );
}
