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

#[test]
fn where_raw_and_typed_combined() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_col(gt::<Posts, posts::PostId>("10"))
        .where_raw("Title != ''")
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE PostId > 10 AND Title != ''");
}

// ── WHERE lt / like / cond ───────────────────────────────────────────────────

#[test]
fn where_lt() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_col(lt::<Posts, posts::PostId>("50"))
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE PostId < 50");
}

#[test]
fn where_like() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .where_col(like::<Users, users::UserName>("%alice%"))
        .build();
    assert_eq!(sql, "SELECT * FROM users WHERE UserName LIKE %alice%");
}

#[test]
fn where_cond_custom_operator() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_col(cond::<Posts, posts::PostId>(">=", "7"))
        .build();
    assert_eq!(sql, "SELECT * FROM posts WHERE PostId >= 7");
}

// ── WHERE IS NOT NULL on nullable FK ─────────────────────────────────────────

#[test]
fn where_is_not_null_on_nullable_fk() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select_all()
        .where_col(is_not_null::<Comments, comments::ParentId>())
        .build();
    assert_eq!(sql, "SELECT * FROM comments WHERE ParentId IS NOT NULL");
}

// ── typed_eq on FK (non-PK) column ───────────────────────────────────────────

#[test]
fn typed_eq_on_fk_column() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select_all()
        .where_col(typed_eq::<Comments, comments::PostId>(5_i64))
        .build();
    assert_eq!(sql, "SELECT * FROM comments WHERE PostId = 5");
}

// ── SELECT three columns ──────────────────────────────────────────────────────

#[test]
fn select_three_columns() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select::<(comments::CommentId, comments::Body, comments::PostId)>()
        .build();
    assert_eq!(
        sql,
        "SELECT comments.CommentId, comments.Body, comments.PostId FROM comments"
    );
}

// ── JOIN + WHERE ──────────────────────────────────────────────────────────────

#[test]
fn join_with_where_condition() {
    let sql = QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::Title, posts::AuthorId)>()
        .join::<Users, posts::AuthorId>()
        .where_col(gt::<Posts, posts::PostId>("100"))
        .build();
    assert_eq!(
        sql,
        "SELECT posts.Title, posts.AuthorId FROM posts \
         INNER JOIN users ON posts.AuthorId = users.UserId \
         WHERE PostId > 100"
    );
}

// ── JOIN + ORDER BY + LIMIT ───────────────────────────────────────────────────

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

// ── LIMIT only / OFFSET only ──────────────────────────────────────────────────

#[test]
fn limit_without_offset() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .limit(3)
        .build();
    assert_eq!(sql, "SELECT * FROM users LIMIT 3");
}

#[test]
fn offset_without_limit() {
    let sql = QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .offset(10)
        .build();
    assert_eq!(sql, "SELECT * FROM users OFFSET 10");
}

// ── Self-join + WHERE ─────────────────────────────────────────────────────────

#[test]
fn self_join_with_is_null_where() {
    let sql = QueryBuilder::new()
        .from::<Comments>()
        .select::<(comments::CommentId, comments::Body)>()
        .left_join::<Comments, comments::ParentId>()
        .where_col(is_null::<Comments, comments::ParentId>())
        .build();
    assert_eq!(
        sql,
        "SELECT comments.CommentId, comments.Body FROM comments \
         LEFT JOIN comments ON comments.ParentId = comments.CommentId \
         WHERE ParentId IS NULL"
    );
}
