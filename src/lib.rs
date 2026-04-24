// =============================================================================
//  Type-safe SQL Builder — PK / FK encoded in the type system
// =============================================================================

use std::marker::PhantomData;


mod join;
mod run;
mod select;
mod table;
mod r#where;

pub use join::{ForeignKey, HasPrimaryKey, JoinClause, PrimaryKey};
pub use run::{AsyncFn, NotRunable, Runable, RunableAsync, RunFn, RunResult};
pub use select::{ColumnSet, NotNull, NotNullColumn, Nullable, NullableColumn};
pub use r#where::{cond, eq, fk_eq, gt, is_not_null, is_null, like, lt, typed_eq, Condition};

// ── Builder phases ────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct NoTable;
pub struct WithTable<T>(PhantomData<T>);
pub struct WithColumns<T>(PhantomData<T>);

// ── Core table / column traits ────────────────────────────────────────────────

pub trait TableSchema {
    const TABLE_NAME: &'static str;
}

pub trait BelongsTo<T: TableSchema> {
    type Value;
    type Null;
    const COLUMN_NAME: &'static str;
}

// ── Order direction ───────────────────────────────────────────────────────────

pub enum Direction {
    Asc,
    Desc,
}
impl Direction {
    fn sql(&self) -> &str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}
// ──  Seal ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct Sealed;

#[derive(Clone, Default)]
pub struct NotSealed;

// ── Query builder ─────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct QueryInternData {
    table: Option<&'static str>,
    columns: Vec<String>,
    joins: Vec<JoinClause>,
    conditions: Vec<String>,
    order_by: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

pub struct QueryBuilder<Phase, S, R> {
    data: QueryInternData,
    run_fn: Option<RunFn>,
    run_async_fn: Option<AsyncFn>,

    _phase: PhantomData<Phase>,
    _seal: PhantomData<S>,
    _run: PhantomData<R>,
}

impl QueryBuilder<NoTable, NotSealed, NotRunable> {
    pub fn new() -> Self {
        QueryBuilder {
            data: QueryInternData::default(),
            run_fn: None,
            run_async_fn: None,
            _phase: PhantomData,
            _run: PhantomData,
            _seal: PhantomData,
        }
    }
}

impl<R> QueryBuilder<NoTable, NotSealed, R> {
    pub fn from<T: TableSchema>(self) -> QueryBuilder<WithTable<T>, NotSealed, R> {
        let mut q: QueryBuilder<WithTable<T>, NotSealed, R> = self.cast();
        q.data.table = Some(T::TABLE_NAME);
        q
    }
}

impl<T: TableSchema, R> QueryBuilder<WithTable<T>, NotSealed, R> {
    pub fn seal(self) -> Self {
        self
    }
}

impl<T: TableSchema, S, R> QueryBuilder<WithColumns<T>, S, R> {
    pub fn build(self) -> String {
        assert!(!self.data.columns.is_empty()); // due to type
        let cols = self.data.columns.join(", ");

        let table = self.data.table.unwrap();
        let mut sql = format!("SELECT {cols} FROM {table}");

        for join in self.data.joins {
            sql.push(' ');
            sql.push_str(&join.to_sql());
        }

        if !self.data.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.data.conditions.join(" AND "));
        }
        if let Some(o) = self.data.order_by {
            sql.push_str(&format!(" ORDER BY {o}"));
        }
        if let Some(l) = self.data.limit {
            sql.push_str(&format!(" LIMIT {l}"));
        }
        if let Some(o) = self.data.offset {
            sql.push_str(&format!(" OFFSET {o}"));
        }
        sql
    }
}

impl<T: TableSchema, R> QueryBuilder<WithColumns<T>, NotSealed, R> {
    pub fn order_by<C: BelongsTo<T>>(mut self, dir: Direction) -> Self {
        self.data.order_by = Some(format!("{} {}", C::COLUMN_NAME, dir.sql()));
        self
    }
    pub fn limit(mut self, n: usize) -> Self {
        self.data.limit = Some(n);
        self
    }
    pub fn offset(mut self, n: usize) -> Self {
        self.data.offset = Some(n);
        self
    }
}

impl<A, S, R> QueryBuilder<A, S, R> {
    fn cast<B>(self) -> QueryBuilder<B, S, R> {
        QueryBuilder {
            data: self.data,
            run_fn: self.run_fn,
            run_async_fn: self.run_async_fn,
            _phase: PhantomData,
            _run: PhantomData,
            _seal: PhantomData,
        }
    }
}
