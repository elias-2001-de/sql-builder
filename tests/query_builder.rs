use sql_builder::*;

// ── Test schema ───────────────────────────────────────────────────────────────

table! {
    users: Users => "users" {
        UserId*:  i64,
        UserName: String,
        Email?:   String
    }
}
impl_has_pk!(Users, users::UserId);

table! {
    posts: Posts => "posts" {
        PostId*:   i64,
        Title:     String,
        AuthorId-> Users: i64,
        Draft?:    bool
    }
}
impl_has_pk!(Posts, posts::PostId);

table! {
    comments: Comments => "comments" {
        CommentId*: i64,
        Body:       String,
        PostId->    Posts: i64,
        AuthorId->  Users: i64,
        ParentId?-> Comments: i64
    }
}
impl_has_pk!(Comments, comments::CommentId);

// ── SELECT ────────────────────────────────────────────────────────────────────

#[test]
fn select_all_produces_star() {
    let sql = QueryBuilder::new().from::<Users>().select_all().build();
    assert_eq!(sql, "SELECT * FROM users");
}

#[test]
fn select_single_column() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select::<(users::UserId,)>()
        .build();
    assert_eq!(sql, "SELECT users.UserId FROM users");
}

#[test]
fn select_multiple_columns() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select::<(users::UserId, users::UserName)>()
        .build();
    assert_eq!(sql, "SELECT users.UserId, users.UserName FROM users");
}

// ── WHERE ─────────────────────────────────────────────────────────────────────

#[test]
fn where_eq() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_col(eq::<Users, users::UserId>("1"))
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE UserId = 1");
}

#[test]
fn where_gt() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_col(gt::<Posts, posts::PostId>("10"))
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE PostId > 10");
}

#[test]
fn where_is_null_on_nullable_column() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_col(is_null::<Users, users::Email>())
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE Email IS NULL");
}

#[test]
fn where_is_not_null() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_col(is_not_null::<Posts, posts::Draft>())
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE Draft IS NOT NULL");
}

#[test]
fn where_multiple_conditions_joined_with_and() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_col(gt::<Posts, posts::PostId>("5"))
        .where_col(is_null::<Posts, posts::Draft>())
        .build();
    assert_eq!(
        sql,
        "SELECT * FROM posts WHERE PostId > 5 AND Draft IS NULL"
    );
}

#[test]
fn typed_eq_on_pk() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_col(typed_eq::<Posts, posts::PostId>(99_i64))
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE PostId = 99");
}

#[test]
fn fk_eq_on_fk_column() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_col(fk_eq::<Posts, posts::AuthorId>(7_i64))
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE AuthorId = 7");
}

// ── JOIN ──────────────────────────────────────────────────────────────────────

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

// ── ORDER / LIMIT / OFFSET ───────────────────────────────────────────────────

#[test]
fn order_by_asc() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .order_by::<posts::PostId>(Direction::Asc)
        .build();
    assert_eq!(sql, "SELECT * FROM posts ORDER BY PostId ASC");
}

#[test]
fn order_by_desc_with_limit_and_offset() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .order_by::<posts::PostId>(Direction::Desc)
        .limit(10)
        .offset(20)
        .build();
    assert_eq!(
        sql,
        "SELECT * FROM posts ORDER BY PostId DESC LIMIT 10 OFFSET 20"
    );
}

// ── WHERE RAW ─────────────────────────────────────────────────────────────────

#[test]
fn where_raw_appended_as_is() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_raw("UserId IN (1, 2, 3)")
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE UserId IN (1, 2, 3)");
}
