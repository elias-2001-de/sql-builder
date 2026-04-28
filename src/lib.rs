// =============================================================================
//  Type-safe SQL Builder — PK / FK encoded in the type system
// =============================================================================

use std::marker::PhantomData;

mod delete;
mod insert;
mod join;
mod run;
mod select;
mod update;
mod r#where;

pub use delete::DeleteBuilder;
pub use insert::{InsertBuilder, WithValues};
pub use join::{ForeignKey, HasPrimaryKey, JoinClause, PrimaryKey};
pub use run::{AsyncFn, NotRunable, RunFn, RunResult, Runable, RunableAsync};
pub use select::{
    ColumnSet, Count, Max, Min, NotNull, NotNullColumn, Nullable, NullableColumn, SelectExpr, Sum,
    UniqueColumn,
};
pub use sql_builder_derive::Table;
pub use update::{UpdateBuilder, WithSet};
pub use r#where::{HasCondition, IntoValue, NoCondition, NeedsOperand, Value, WhereClause};

// ── Builder phases ────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct NoTable;
pub struct WithTable<T>(PhantomData<T>);

pub struct AllColumns;
pub struct WithColumns<T, Cols = AllColumns>(PhantomData<(T, Cols)>);

// ── Typed subquery ────────────────────────────────────────────────────────────

pub struct Subquery<Val> {
    sql: String,
    _phantom: PhantomData<Val>,
}

pub trait SubquerySql {
    fn into_subquery_sql(self) -> String;
}

impl SubquerySql for String {
    fn into_subquery_sql(self) -> String {
        self
    }
}

impl<Val> SubquerySql for Subquery<Val> {
    fn into_subquery_sql(self) -> String {
        self.sql
    }
}

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

#[derive(Clone, Copy, Debug)]
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
    subquery_source: Option<String>,
    columns: Vec<String>,
    joins: Vec<JoinClause>,
    conditions: Vec<String>,
    group_by: Vec<&'static str>,
    having: Vec<String>,
    order_by: Option<(&'static str, Direction)>,
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

impl<Phase, S> QueryBuilder<Phase, S, NotRunable> {
    pub fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            run_fn: None,
            run_async_fn: None,
            _run: PhantomData,
            _phase: PhantomData,
            _seal: PhantomData,
        }
    }
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
    pub fn from_subquery<T: TableSchema>(
        self,
        sql: impl SubquerySql,
    ) -> QueryBuilder<WithTable<T>, NotSealed, R> {
        let mut q: QueryBuilder<WithTable<T>, NotSealed, R> = self.cast();
        q.data.table = Some(T::TABLE_NAME);
        q.data.subquery_source = Some(sql.into_subquery_sql());
        q
    }
}

impl<T: TableSchema, R> QueryBuilder<WithTable<T>, NotSealed, R> {
    pub fn seal(self) -> Self {
        self
    }
}

impl<T: TableSchema, Cols, S, R> QueryBuilder<WithColumns<T, Cols>, S, R> {
    pub fn build(self) -> String {
        assert!(!self.data.columns.is_empty()); // due to type
        let cols = self.data.columns.join(", ");

        let table = self.data.table.unwrap();
        let mut sql = match self.data.subquery_source {
            Some(ref sub) => format!("SELECT {cols} FROM ({sub}) AS {table}"),
            None => format!("SELECT {cols} FROM {table}"),
        };

        for join in self.data.joins {
            sql.push(' ');
            sql.push_str(&join.to_sql());
        }

        if !self.data.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.data.conditions.join(" AND "));
        }
        if !self.data.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.data.group_by.join(", "));
        }
        if !self.data.having.is_empty() {
            sql.push_str(" HAVING ");
            sql.push_str(&self.data.having.join(" AND "));
        }
        if let Some((col, dir)) = self.data.order_by {
            sql.push_str(&format!(" ORDER BY {col} {}", dir.sql()));
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

impl<T: TableSchema, Cols, R> QueryBuilder<WithColumns<T, Cols>, NotSealed, R> {
    pub fn group_by<C: BelongsTo<T>>(mut self) -> Self {
        self.data.group_by.push(C::COLUMN_NAME);
        self
    }
    pub fn having(mut self, clause: WhereClause<T, HasCondition>) -> Self {
        self.data.having.push(format!("({})", clause.build_fragment()));
        self
    }
    pub fn order_by<C: BelongsTo<T>>(mut self, dir: Direction) -> Self {
        self.data.order_by = Some((C::COLUMN_NAME, dir));
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
