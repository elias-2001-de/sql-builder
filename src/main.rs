use sql_builder::*;
use sql_builder::{DeleteBuilder, InsertBuilder, UpdateBuilder};

// ── Schema ────────────────────────────────────────────────────────────────────

#[derive(Table)]
#[table_name = "users"]
pub struct Users {
    #[primary_key]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Table)]
#[table_name = "posts"]
pub struct Posts {
    #[primary_key]
    pub id: i64,
    pub title: String,
    pub body: String,
    #[foreign_key(Users)]
    pub author_id: i64,
    pub published_at: Option<i64>,
    pub deleted_at: Option<i64>,
}

#[derive(Table)]
#[table_name = "comments"]
pub struct Comments {
    #[primary_key]
    pub id: i64,
    pub content: String,
    #[foreign_key(Posts)]
    pub post_id: i64,
    #[foreign_key(Users)]
    pub author_id: i64,
    #[foreign_key(Comments)]
    pub parent_id: Option<i64>,
}

// ── Demo ──────────────────────────────────────────────────────────────────────

fn main() {
    // PK lookup
    let q1 = QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(WhereClause::<Posts, _>::new().eq::<posts::Id, _>(42_i64))
        .build();
    println!("Q1 (pk lookup):\n  {q1}\n");

    // JOIN via FK with WHERE and ORDER BY
    let q2 = QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::Title, posts::AuthorId)>()
        .join::<Users, posts::AuthorId>()
        .where_clause(WhereClause::<Posts, _>::new().gt::<posts::Id, _>(100_i64))
        .order_by::<posts::Id>(Direction::Desc)
        .limit(5)
        .build();
    println!("Q2 (join posts→users):\n  {q2}\n");

    // Multiple JOINs — comments has three FKs
    let q3 = QueryBuilder::new()
        .from::<Comments>()
        .select_all()
        .join::<Posts, comments::PostId>()
        .join::<Users, comments::AuthorId>()
        .left_join::<Comments, comments::ParentId>()
        .build();
    println!("Q3 (multi-join with self-ref):\n  {q3}\n");

    // Nullable FK — find top-level comments (no parent)
    let q4 = QueryBuilder::new()
        .from::<Comments>()
        .select::<(comments::Id, comments::Content)>()
        .where_clause(WhereClause::<Comments, _>::new().is_null::<comments::ParentId>())
        .build();
    println!("Q4 (top-level comments):\n  {q4}\n");

    // INSERT
    let q5 = InsertBuilder::new()
        .into_table::<Users>()
        .value::<users::Name, _>("Alice")
        .value::<users::Email, _>("alice@example.com")
        .value::<users::Age, _>(30_i32)
        .build();
    println!("Q5 (insert user):\n  {q5}\n");

    // UPDATE with WHERE
    let q6 = UpdateBuilder::new()
        .table::<Users>()
        .set::<users::Name, _>("Bob")
        .set::<users::Age, _>(25_i32)
        .set_null::<users::Bio>()
        .where_clause(WhereClause::<Users, _>::new().eq::<users::Id, _>(1_i64))
        .build();
    println!("Q6 (update user):\n  {q6}\n");

    // DELETE with WHERE
    let q7 = DeleteBuilder::new()
        .from::<Posts>()
        .where_clause(
            WhereClause::<Posts, _>::new()
                .is_not_null::<posts::DeletedAt>()
                .and()
                .lt::<posts::Id, _>(100_i64),
        )
        .build();
    println!("Q7 (delete old deleted posts):\n  {q7}\n");
}
