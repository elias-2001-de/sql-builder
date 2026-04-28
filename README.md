# sql-builder

A type-safe SQL query builder for Rust, where the schema — tables, columns,
primary keys, foreign keys, and nullability — is fully encoded in the type
system. Invalid queries are rejected at **compile time**, not at runtime.

Inspired by the [TypeState Builder Pattern video](https://www.youtube.com/watch?v=pwmIQzLuYl0).

---

## How it works

### The typestate pattern

The `QueryBuilder` moves through phases as you add clauses. Each phase is a
distinct type, so calling methods out of order — or calling `build()` before
selecting any columns — is a compile error, not a runtime panic.

```
QueryBuilder<NoTable>       →  .from::<T>()  /  .from_subquery::<T>(sql)
QueryBuilder<WithTable<T>>  →  .select()  /  .select_all()
QueryBuilder<WithColumns<T>> →  .where_clause()  /  .join()  /  .order_by()  /  .build()
```

### Schema DSL

Tables are declared with `#[derive(Table)]`. Each field maps to a column; its
type, nullability, and role (PK / FK / plain) are captured in the type system
via attributes:

```rust
use sql_builder::Table;

#[derive(Table)]
#[table_name = "posts"]
pub struct Posts {
    #[primary_key]
    #[column_name = "PostId"]
    pub post_id: i64,            // PK, NOT NULL

    #[column_name = "Title"]
    pub title: String,           // NOT NULL

    #[foreign_key(Users)]
    #[column_name = "AuthorId"]
    pub author_id: i64,          // FK → Users, NOT NULL

    #[column_name = "DeletedAt"]
    pub deleted_at: Option<i64>, // nullable
}
```

Column structs are generated in a module named after the lowercased struct name,
so `posts::Title` and `users::Title` never collide even if two tables share a
column name.

### Compile-time safety

- `join::<B, FK>()` only compiles when `FK` is actually a foreign key pointing
  to `B`. Joining the wrong table is a type error.
- `is_null` / `is_not_null` only accept nullable columns (`Option<_>`).
- `eq`, `gt`, `lt`, etc. check the value type against the column at compile time.
- Multiple `where_clause()` calls are AND-joined at the top level; inside a
  single clause you can chain `.and()` / `.or()` freely.

---

## Quick example

```rust
use sql_builder::*;

// Simple PK lookup
let sql = QueryBuilder::new()
    .from::<Posts>()
    .select_all()
    .where_clause(WhereClause::<Posts, _>::new().eq::<posts::PostId, _>(42_i64))
    .build();
// SELECT * FROM posts WHERE (PostId = 42)

// JOIN with WHERE, ORDER BY, LIMIT
let sql = QueryBuilder::new()
    .from::<Posts>()
    .select::<(posts::Title, posts::AuthorId)>()
    .join::<Users, posts::AuthorId>()
    .where_clause(WhereClause::<Posts, _>::new().gt::<posts::PostId, _>(100_i64))
    .order_by::<posts::PostId>(Direction::Desc)
    .limit(10)
    .build();
// SELECT posts.Title, posts.AuthorId FROM posts
// INNER JOIN users ON posts.AuthorId = users.UserId
// WHERE (PostId > 100) ORDER BY PostId DESC LIMIT 10

// Self-referential LEFT JOIN + IS NULL
let sql = QueryBuilder::new()
    .from::<Comments>()
    .select::<(comments::CommentId, comments::Body)>()
    .left_join::<Comments, comments::ParentId>()
    .where_clause(WhereClause::<Comments, _>::new().is_null::<comments::ParentId>())
    .build();
// SELECT comments.CommentId, comments.Body FROM comments
// LEFT JOIN comments ON comments.ParentId = comments.CommentId
// WHERE (ParentId IS NULL)
```

---

## Subqueries

### WHERE subqueries

Pass a pre-built SQL string to the subquery predicates on `WhereClause`:

```rust
// col IN (SELECT ...)
let active_user_ids = QueryBuilder::new()
    .from::<Users>()
    .select::<(users::UserId,)>()
    .where_clause(WhereClause::new().eq::<users::UserName, _>("Alice"))
    .build();

let sql = QueryBuilder::new()
    .from::<Posts>()
    .select_all()
    .where_clause(
        WhereClause::new().in_subquery::<posts::AuthorId>(active_user_ids)
    )
    .build();
// SELECT * FROM posts
// WHERE (AuthorId IN (SELECT users.UserId FROM users WHERE (UserName = 'Alice')))
```

Subquery predicates chain with `.and()` / `.or()` like any other predicate.

#### WHERE subquery predicates

| Method | SQL emitted |
|---|---|
| `.in_subquery::<C>(sql)` | `col IN (SELECT …)` |
| `.not_in_subquery::<C>(sql)` | `col NOT IN (SELECT …)` |
| `.exists(sql)` | `EXISTS (SELECT …)` |
| `.not_exists(sql)` | `NOT EXISTS (SELECT …)` |

### FROM subquery

Define a struct that mirrors the subquery's output columns, then pass the
built SQL to `from_subquery`. The struct's `TABLE_NAME` becomes the alias.

```rust
#[derive(Table)]
#[table_name = "active_authors"]
struct ActiveAuthors {
    #[primary_key]
    #[column_name = "UserId"]
    user_id: i64,
    #[column_name = "UserName"]
    user_name: String,
}

let inner = QueryBuilder::new()
    .from::<Users>()
    .select::<(users::UserId, users::UserName)>()
    .where_clause(WhereClause::new().eq::<users::Email, _>("admin@example.com"))
    .build();

let sql = QueryBuilder::new()
    .from_subquery::<ActiveAuthors>(inner)
    .select_all()
    .build();
// SELECT * FROM
//   (SELECT users.UserId, users.UserName FROM users WHERE (Email = 'admin@example.com'))
//   AS active_authors
```

---

## WhereClause predicates

| Method | SQL emitted |
|---|---|
| `.eq::<C, _>(v)` | `col = v` |
| `.not_eq::<C, _>(v)` | `col <> v` |
| `.lt::<C, _>(v)` | `col < v` |
| `.lt_eq::<C, _>(v)` | `col <= v` |
| `.gt::<C, _>(v)` | `col > v` |
| `.gt_eq::<C, _>(v)` | `col >= v` |
| `.like::<C, _>(pat)` | `col LIKE pat` |
| `.between::<C, _>(lo, hi)` | `col BETWEEN lo AND hi` |
| `.in_values::<C, _>(iter)` | `col IN (v1, v2, …)` |
| `.in_subquery::<C>(sql)` | `col IN (SELECT …)` |
| `.not_in_subquery::<C>(sql)` | `col NOT IN (SELECT …)` |
| `.exists(sql)` | `EXISTS (SELECT …)` |
| `.not_exists(sql)` | `NOT EXISTS (SELECT …)` |
| `.is_null::<C>()` | `col IS NULL` *(nullable only)* |
| `.is_not_null::<C>()` | `col IS NOT NULL` *(nullable only)* |

Chain predicates with `.and()` or `.or()` between them:

```rust
WhereClause::<Posts, _>::new()
    .gt::<posts::PostId, _>(10_i64)
    .and()
    .lt::<posts::PostId, _>(50_i64)
// PostId > 10 AND PostId < 50
```

---

## Non-goals

This library is a **query builder only** — it intentionally does not:

- **Connect to a database.** You supply a runner by implementing the `Run` /
  `RunAsync` traits. The builder produces a SQL string; execution is your
  responsibility.
- **Sanitize input.** SQL value escaping (e.g. `'` → `''` for text) is handled
  internally for the literal-value predicates. Parameterized queries are the
  concern of the runner you plug in.
- **Handle migrations.** Schema evolution is outside scope.
- **Support database-specific extensions.** The output targets standard SQL.
  Vendor-specific syntax (window functions, CTEs, JSON operators, etc.) is out
  of scope; write a raw string for those cases.

