// =============================================================================
//  Type-safe SQL Builder — PK / FK encoded in the type system
// =============================================================================

use std::marker::PhantomData;

mod delete;
mod execute;
mod init;
mod insert;
mod join;
mod query;
mod select;
mod update;
mod r#where;

pub use delete::DeleteBuilder;
pub use execute::{
    AsyncFn, ExecuteAllFn, ExecuteAllResult, ExecuteFn, ExecuteOneFn, ExecuteOneResult,
    ExecuteResult, Executable, ExecutableAll, ExecutableAsync, ExecutableOne, NotExecutable,
};
pub use init::{ColumnDef, DbAdapter, SqlTypeKind, TableInit, ToSqlType};
pub use insert::{InsertBuilder, WithValues};
pub use join::{ForeignKey, HasPrimaryKey, PrimaryKey};
pub use query::QueryBuilder;
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

// ── Seal markers ──────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct Sealed;

#[derive(Clone, Default)]
pub struct NotSealed;

// ── Typed subquery ────────────────────────────────────────────────────────────

pub struct Subquery<Val> {
    pub(crate) sql: String,
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
    pub(crate) fn sql(&self) -> &str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}
