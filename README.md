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
QueryBuilder<NoTable>  →  .from::<T>()
QueryBuilder<WithTable<T>>  →  .select() / .select_all()
QueryBuilder<WithColumns<T>>  →  .where_clause() / .join() / .order_by() / .build()
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
    pub post_id: i64,           // PK, NOT NULL

    #[column_name = "Title"]
    pub title: String,          // NOT NULL

    #[foreign_key(Users)]
    #[column_name = "AuthorId"]
    pub author_id: i64,         // FK → Users, NOT NULL

    #[column_name = "DeletedAt"]
    pub deleted_at: Option<i64> // nullable
}
```

Column structs are generated in a module named after the table, so
`posts::Title` and `users::Title` never collide even if two tables share a
column name.

### Compile-time safety

- `join::<B, FK>()` only compiles when `FK` is actually a foreign key pointing
  to `B`. Joining the wrong table is a type error.
- `is_null` / `is_not_null` only accept nullable columns (`Option<_>`).
- `eq`, `gt`, `lt`, etc. check the value type of the column at compile time.
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

### WhereClause predicates

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

## TODO

### Query capabilities
- [ ] **`INSERT` / `UPDATE` / `DELETE` builders** — follow the same typestate
  approach for write queries.
- [ ] **`GROUP BY` / `HAVING`** — aggregate query support.
- [ ] **Aggregate columns** — `COUNT(*)`, `MAX(col)`, `SUM(col)` as selectable
  column expressions.
- [ ] **Subqueries** — allow a `QueryBuilder` to be used as a nested expression
  inside a WHERE clause or FROM.

### Ergonomics
- [ ] **`ColumnSet` for tuples > 5** — the blanket impls currently top out at 5;
  extend or generate them with a macro.

### Runtime integration
- [ ] **Table registry** — a global registry (possibly via the
  [`inventory`](https://github.com/dtolnay/inventory) crate) where each table
  self-registers on startup, enabling a single `db.init_all_tables()` call
  that runs every table's DDL without manually listing them.
- [ ] **Async runner** — the `runner_async` / `RunableAsync` infrastructure is
  in place; wire it up to a concrete adapter (e.g. `sqlx`, `tokio-postgres`)
  and document the pattern.
- [ ] **`seal()` / `Sealed` phase** — the `Sealed` marker type exists but isn't
  used yet; define what sealing means (e.g. frozen, no further mutation) and
  enforce it.
