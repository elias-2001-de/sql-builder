mod common;

use common::{Posts, Users};
use sql_builder::{QueryBuilder, Table, WhereClause};

// ── FROM subquery ─────────────────────────────────────────────────────────────

#[derive(Table)]
#[table_name = "active_authors"]
struct ActiveAuthors {
    #[primary_key]
    #[column_name = "UserId"]
    user_id: i64,
    #[column_name = "UserName"]
    user_name: String,
}

#[test]
fn from_subquery_select_all() {
    let inner = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_clause(WhereClause::new().eq::<common::users::UserName, _>("Alice"))
        .build();

    let sql = QueryBuilder::new()
        .from_subquery::<ActiveAuthors>(inner)
        .select_all()
        .build();

    assert_eq!(
        sql,
        "SELECT * FROM (SELECT * FROM users WHERE (UserName = 'Alice')) AS active_authors"
    );
}

#[test]
fn from_subquery_with_where() {
    let inner = QueryBuilder::new()
        .from::<Users>()
        .select::<(common::users::UserId,)>()
        .build();

    let sql = QueryBuilder::new()
        .from_subquery::<ActiveAuthors>(inner)
        .select::<(activeauthors::UserId,)>()
        .where_clause(WhereClause::new().eq::<activeauthors::UserName, _>("Bob"))
        .build();

    assert_eq!(
        sql,
        "SELECT active_authors.UserId FROM (SELECT users.UserId FROM users) AS active_authors WHERE (UserName = 'Bob')"
    );
}

// ── WHERE IN subquery ─────────────────────────────────────────────────────────

#[test]
fn where_in_subquery() {
    let inner = QueryBuilder::new()
        .from::<Users>()
        .select::<(common::users::UserId,)>()
        .where_clause(WhereClause::new().eq::<common::users::UserName, _>("Alice"))
        .into_subquery();

    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(WhereClause::new().in_subquery::<common::posts::AuthorId>(inner))
        .build();

    assert_eq!(
        sql,
        "SELECT * FROM posts WHERE (AuthorId IN (SELECT users.UserId FROM users WHERE (UserName = 'Alice')))"
    );
}

#[test]
fn where_not_in_subquery() {
    let inner = QueryBuilder::new()
        .from::<Users>()
        .select::<(common::users::UserId,)>()
        .into_subquery();

    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(WhereClause::new().not_in_subquery::<common::posts::AuthorId>(inner))
        .build();

    assert_eq!(
        sql,
        "SELECT * FROM posts WHERE (AuthorId NOT IN (SELECT users.UserId FROM users))"
    );
}

// ── WHERE EXISTS / NOT EXISTS ─────────────────────────────────────────────────

#[test]
fn where_exists_subquery() {
    let inner = QueryBuilder::new()
        .from::<Users>()
        .select::<(common::users::UserId,)>()
        .where_clause(WhereClause::new().eq::<common::users::UserName, _>("Alice"))
        .build();

    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(WhereClause::<Posts, _>::new().exists(inner))
        .build();

    assert_eq!(
        sql,
        "SELECT * FROM posts WHERE (EXISTS (SELECT users.UserId FROM users WHERE (UserName = 'Alice')))"
    );
}

#[test]
fn where_not_exists_subquery() {
    let inner = QueryBuilder::new()
        .from::<Users>()
        .select::<(common::users::UserId,)>()
        .build();

    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(WhereClause::<Posts, _>::new().not_exists(inner))
        .build();

    assert_eq!(
        sql,
        "SELECT * FROM posts WHERE (NOT EXISTS (SELECT users.UserId FROM users))"
    );
}

// ── Chained subquery predicates ───────────────────────────────────────────────

#[test]
fn chained_in_subquery_and_eq() {
    let inner = QueryBuilder::new()
        .from::<Users>()
        .select::<(common::users::UserId,)>()
        .into_subquery();

    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(
            WhereClause::new()
                .in_subquery::<common::posts::AuthorId>(inner)
                .and()
                .eq::<common::posts::Title, _>("Hello"),
        )
        .build();

    assert_eq!(
        sql,
        "SELECT * FROM posts WHERE (AuthorId IN (SELECT users.UserId FROM users) AND Title = 'Hello')"
    );
}
