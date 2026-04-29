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
mod string_runner;
mod update;
mod r#where;

pub use delete::DeleteBuilder;
pub use execute::{DeleteData, InsertData, NotExecutable, QueryData, Runner, RunnerAsync, UpdateData};
pub use init::{ColumnDef, DbAdapter, SqlTypeKind, TableInit, ToSqlType};
pub use insert::{InsertBuilder, WithValues};
pub use join::{ForeignKey, HasPrimaryKey, PrimaryKey};
pub use query::QueryBuilder;
pub use select::{
    ColumnSet, Count, Max, Min, NotNull, NotNullColumn, Nullable, NullableColumn, SelectExpr, Sum,
    UniqueColumn,
};
pub use sql_builder_derive::Table;
pub use string_runner::StringRunner;
pub use update::{UpdateBuilder, WithSet};
pub use r#where::{HasCondition, IntoValue, NeedsOperand, NoCondition, Value, WhereClause};

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
    pub(crate) data: query::QueryInternData,
    _phantom: PhantomData<Val>,
}

#[allow(private_interfaces)]
pub trait SubquerySql {
    fn into_subquery_data(self) -> query::QueryInternData;
}

#[allow(private_interfaces)]
impl<Val> SubquerySql for Subquery<Val> {
    fn into_subquery_data(self) -> query::QueryInternData {
        self.data
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

// ── Column expression data ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum ColumnExpr {
    All,
    Column {
        table: &'static str,
        name: &'static str,
    },
    Count,
    Max {
        table: &'static str,
        name: &'static str,
    },
    Min {
        table: &'static str,
        name: &'static str,
    },
    Sum {
        table: &'static str,
        name: &'static str,
    },
}

// ── Order direction ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Asc,
    Desc,
}

impl Direction {
    #[allow(dead_code)]
    pub(crate) fn sql(&self) -> &str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}
