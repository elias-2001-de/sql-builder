use sql_builder::{Direction, QueryBuilder, StringRunner, Table, WhereClause};

// ── Schema defined with #[derive(Table)] ─────────────────────────────────────

#[derive(Table)]
#[table_name = "posts"]
pub struct Post {
    #[primary_key]
    pub id: i64,
    pub title: String,
    pub body: Option<String>,
}

#[derive(Table)]
#[table_name = "comments"]
pub struct Comment {
    #[primary_key]
    pub id: i64,

    // custom SQL column name
    #[column_name = "content_text"]
    pub content: String,

    // unique constraint
    #[unique]
    pub slug: String,

    // nullable field
    pub extra: Option<String>,

    // foreign key
    #[foreign_key(Post)]
    pub post_id: i64,
}

// Table without a primary key (no HasPrimaryKey impl generated)
#[derive(Table)]
pub struct Tag {
    pub name: String,
}

// ── Basic TableSchema ─────────────────────────────────────────────────────────

#[test]
fn table_name_custom() {
    use sql_builder::TableSchema;
    assert_eq!(Post::TABLE_NAME, "posts");
    assert_eq!(Comment::TABLE_NAME, "comments");
}

#[test]
fn table_name_default_lowercase() {
    use sql_builder::TableSchema;
    assert_eq!(Tag::TABLE_NAME, "tag");
}

// ── Column names ──────────────────────────────────────────────────────────────

#[test]
fn column_name_default() {
    use sql_builder::BelongsTo;
    assert_eq!(post::Id::COLUMN_NAME, "id");
    assert_eq!(post::Title::COLUMN_NAME, "title");
    assert_eq!(post::Body::COLUMN_NAME, "body");
}

#[test]
fn column_name_custom() {
    use sql_builder::BelongsTo;
    // #[column_name = "content_text"] on field `content`
    assert_eq!(comment::Content::COLUMN_NAME, "content_text");
}

// ── SELECT queries ────────────────────────────────────────────────────────────

#[test]
fn select_all() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Post>()
        .select_all()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT * FROM posts");
}

#[test]
fn select_single_column() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Post>()
        .select::<(post::Title,)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT posts.title FROM posts");
}

#[test]
fn select_multiple_columns() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Post>()
        .select::<(post::Id, post::Title)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT posts.id, posts.title FROM posts");
}

#[test]
fn select_custom_column_name() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Comment>()
        .select::<(comment::Content,)>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT comments.content_text FROM comments");
}

// ── WHERE clauses ─────────────────────────────────────────────────────────────

#[test]
fn where_eq() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Post>()
        .select_all()
        .where_clause(WhereClause::<Post, _>::new().eq::<post::Title, _>("hello"))
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT * FROM posts WHERE (title = 'hello')");
}

#[test]
fn where_typed_eq_pk() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Post>()
        .select_all()
        .where_clause(WhereClause::<Post, _>::new().eq::<post::Id, _>(42_i64))
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT * FROM posts WHERE (id = 42)");
}

#[test]
fn where_is_null_on_nullable_field() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Post>()
        .select_all()
        .where_clause(WhereClause::<Post, _>::new().is_null::<post::Body>())
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT * FROM posts WHERE (body IS NULL)");
}

// ── JOIN via FK ───────────────────────────────────────────────────────────────

#[test]
fn inner_join_via_fk() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Comment>()
        .select_all()
        .join::<Post, comment::PostId>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(
        runner.query().unwrap(),
        "SELECT * FROM comments INNER JOIN posts ON comments.post_id = posts.id"
    );
}

#[test]
fn left_join_via_fk() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Comment>()
        .select_all()
        .left_join::<Post, comment::PostId>()
        .execute_all(&runner)
        .unwrap();
    assert_eq!(
        runner.query().unwrap(),
        "SELECT * FROM comments LEFT JOIN posts ON comments.post_id = posts.id"
    );
}

// ── ORDER BY / LIMIT / OFFSET ─────────────────────────────────────────────────

#[test]
fn order_limit_offset() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Post>()
        .select_all()
        .order_by::<post::Title>(Direction::Asc)
        .limit(10)
        .offset(20)
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT * FROM posts ORDER BY title ASC LIMIT 10 OFFSET 20");
}
