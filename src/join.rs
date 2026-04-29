use crate::{BelongsTo, NotSealed, QueryBuilder, TableSchema, WithColumns, select::NotNull};

// ── PK / FK traits ───────────────────────────────────────────────────────────

pub trait PrimaryKey<T: TableSchema>: BelongsTo<T, Null = NotNull> {}

pub trait ForeignKey<T: TableSchema>: BelongsTo<T> {
    type References: TableSchema;
    type RefColumn: PrimaryKey<Self::References>;
}

/// Looked up by the `#[derive(Table)]` macro for FK `RefColumn`.
pub trait HasPrimaryKey: TableSchema + Sized {
    type PkColumn: PrimaryKey<Self>;
}

// ── Join clause ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum JoinType {
    Inner,
    Left,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct JoinClause {
    kind: JoinType,
    table_name_a: String,
    table_name_b: String,
    table_fk: String,
    table_fk_ref: String,
}

impl JoinClause {
    #[allow(dead_code)]
    pub(crate) fn to_sql(self) -> String {
        let keyword = match self.kind {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
        };
        format!(
            "{} {} ON {}.{} = {}.{}",
            keyword,
            self.table_name_b,
            self.table_name_a,
            self.table_fk,
            self.table_name_b,
            self.table_fk_ref,
        )
    }
}

// ── QueryBuilder integration ──────────────────────────────────────────────────

impl<T: TableSchema, Cols, R, Row> QueryBuilder<WithColumns<T, Cols>, NotSealed, R, Row> {
    pub fn join<B, FK>(mut self) -> Self
    where
        B: TableSchema,
        FK: ForeignKey<T, References = B>,
    {
        self.data.joins.push(JoinClause {
            kind: JoinType::Inner,
            table_name_a: T::TABLE_NAME.into(),
            table_name_b: B::TABLE_NAME.into(),
            table_fk: FK::COLUMN_NAME.into(),
            table_fk_ref: FK::RefColumn::COLUMN_NAME.into(),
        });
        self
    }

    pub fn left_join<B, FK>(mut self) -> Self
    where
        B: TableSchema,
        FK: ForeignKey<T, References = B>,
    {
        self.data.joins.push(JoinClause {
            kind: JoinType::Left,
            table_name_a: T::TABLE_NAME.into(),
            table_name_b: B::TABLE_NAME.into(),
            table_fk: FK::COLUMN_NAME.into(),
            table_fk_ref: FK::RefColumn::COLUMN_NAME.into(),
        });
        self
    }
}
