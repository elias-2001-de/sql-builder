use sql_builder::*;

// ── Test schema ───────────────────────────────────────────────────────────────

#[derive(Table)]
#[table_name = "users"]
pub struct Users {
    #[primary_key]
    #[column_name = "UserId"]
    pub user_id: i64,
    #[column_name = "UserName"]
    pub user_name: String,
    #[column_name = "Email"]
    pub email: Option<String>,
}

#[derive(Table)]
#[table_name = "posts"]
pub struct Posts {
    #[primary_key]
    #[column_name = "PostId"]
    pub post_id: i64,
    #[column_name = "Title"]
    pub title: String,
    #[foreign_key(Users)]
    #[column_name = "AuthorId"]
    pub author_id: i64,
    #[column_name = "Draft"]
    pub draft: Option<bool>,
}

#[derive(Table)]
#[table_name = "comments"]
pub struct Comments {
    #[primary_key]
    #[column_name = "CommentId"]
    pub comment_id: i64,
    #[column_name = "Body"]
    pub body: String,
    #[foreign_key(Posts)]
    #[column_name = "PostId"]
    pub post_id: i64,
    #[foreign_key(Users)]
    #[column_name = "AuthorId"]
    pub author_id: i64,
    #[foreign_key(Comments)]
    #[column_name = "ParentId"]
    pub parent_id: Option<i64>,
}

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

// ── WHERE (WhereClause builder) ───────────────────────────────────────────────

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

// ── JOIN + WHERE ──────────────────────────────────────────────────────────────

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

// ── Self-join + WHERE ─────────────────────────────────────────────────────────

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
