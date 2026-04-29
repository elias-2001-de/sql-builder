use sql_builder::*;
use sql_builder::{DbAdapter, DeleteBuilder, InsertBuilder, SqlTypeKind, StringRunner, UpdateBuilder};

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

// ── Example DB adapter (generic SQL — swap type names for your real DB) ───────

struct GenericAdapter;

impl DbAdapter for GenericAdapter {
    fn sql_type_name(&self, ty: SqlTypeKind) -> &'static str {
        match ty {
            SqlTypeKind::Integer => "INTEGER",
            SqlTypeKind::BigInt  => "BIGINT",
            SqlTypeKind::Real    => "REAL",
            SqlTypeKind::Text    => "TEXT",
            SqlTypeKind::Boolean => "BOOLEAN",
            SqlTypeKind::Blob    => "BLOB",
        }
    }
    fn execute(&self, sql: &str) {
        println!("  [execute] {sql}");
    }
}

// ── Demo ──────────────────────────────────────────────────────────────────────

fn main() {
    // Table init via adapter
    let db = GenericAdapter;
    println!("Init SQL (generic adapter):");
    db.init_table::<Users>();
    db.init_table::<Posts>();
    db.init_table::<Comments>();
    println!();

    // Or just get the SQL string without executing
    let sql = db.create_table_sql::<Users>();
    println!("create_table_sql for Users:\n  {sql}\n");

    let runner = StringRunner::new();

    // PK lookup
    QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .where_clause(WhereClause::<Posts, _>::new().eq::<posts::Id, _>(42_i64))
        .execute_all(&runner)
        .unwrap();
    println!("Q1 (pk lookup):\n  {}\n", runner.query().unwrap());

    // JOIN via FK with WHERE and ORDER BY
    QueryBuilder::new()
        .from::<Posts>()
        .select::<(posts::Title, posts::AuthorId)>()
        .join::<Users, posts::AuthorId>()
        .where_clause(WhereClause::<Posts, _>::new().gt::<posts::Id, _>(100_i64))
        .order_by::<posts::Id>(Direction::Desc)
        .limit(5)
        .execute_all(&runner)
        .unwrap();
    println!("Q2 (join posts→users):\n  {}\n", runner.query().unwrap());

    // Multiple JOINs — comments has three FKs
    QueryBuilder::new()
        .from::<Comments>()
        .select_all()
        .join::<Posts, comments::PostId>()
        .join::<Users, comments::AuthorId>()
        .left_join::<Comments, comments::ParentId>()
        .execute_all(&runner)
        .unwrap();
    println!("Q3 (multi-join with self-ref):\n  {}\n", runner.query().unwrap());

    // Nullable FK — find top-level comments (no parent)
    QueryBuilder::new()
        .from::<Comments>()
        .select::<(comments::Id, comments::Content)>()
        .where_clause(WhereClause::<Comments, _>::new().is_null::<comments::ParentId>())
        .execute_all(&runner)
        .unwrap();
    println!("Q4 (top-level comments):\n  {}\n", runner.query().unwrap());

    // INSERT
    InsertBuilder::new()
        .into_table::<Users>()
        .value::<users::Name, _>("Alice")
        .value::<users::Email, _>("alice@example.com")
        .value::<users::Age, _>(30_i32)
        .execute(&runner)
        .unwrap();
    println!("Q5 (insert user):\n  {}\n", runner.query().unwrap());

    // UPDATE with WHERE
    UpdateBuilder::new()
        .table::<Users>()
        .set::<users::Name, _>("Bob")
        .set::<users::Age, _>(25_i32)
        .set_null::<users::Bio>()
        .where_clause(WhereClause::<Users, _>::new().eq::<users::Id, _>(1_i64))
        .execute(&runner)
        .unwrap();
    println!("Q6 (update user):\n  {}\n", runner.query().unwrap());

    // DELETE with WHERE
    DeleteBuilder::new()
        .from::<Posts>()
        .where_clause(
            WhereClause::<Posts, _>::new()
                .is_not_null::<posts::DeletedAt>()
                .and()
                .lt::<posts::Id, _>(100_i64),
        )
        .execute(&runner)
        .unwrap();
    println!("Q7 (delete old deleted posts):\n  {}\n", runner.query().unwrap());
}
