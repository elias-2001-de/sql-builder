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

pub trait SubquerySql {
    fn into_subquery_data(self) -> query::QueryInternData;
}

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
    Column { table: &'static str, name: &'static str },
    Count,
    Max { table: &'static str, name: &'static str },
    Min { table: &'static str, name: &'static str },
    Sum { table: &'static str, name: &'static str },
}

impl ColumnExpr {
    pub(crate) fn to_sql(&self) -> String {
        match self {
            Self::All => "*".to_string(),
            Self::Column { table, name } => format!("{table}.{name}"),
            Self::Count => "COUNT(*)".to_string(),
            Self::Max { table, name } => format!("MAX({table}.{name})"),
            Self::Min { table, name } => format!("MIN({table}.{name})"),
            Self::Sum { table, name } => format!("SUM({table}.{name})"),
        }
    }
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
