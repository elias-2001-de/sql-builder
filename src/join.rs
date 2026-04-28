
impl<T: TableSchema, Cols, R> QueryBuilder<WithColumns<T, Cols>, NotSealed, R> {
    pub fn join<B, FK>(mut self) -> Self
    where
        B: TableSchema,
        FK: ForeignKey<T, References = B>,
    {
        self.data.joins.push(inner_join::<T, B, FK>());
        self
    }

    pub fn left_join<B, FK>(mut self) -> Self
    where
        B: TableSchema,
        FK: ForeignKey<T, References = B>,
    {
        self.data.joins.push(left_join::<T, B, FK>());
        self
    }
}

// ── PK / FK traits ───────────────────────────────────────────────────────────

pub trait PrimaryKey<T: TableSchema>: BelongsTo<T, Null = NotNull> {}

pub trait ForeignKey<T: TableSchema>: BelongsTo<T> {
    type References: TableSchema;
    type RefColumn: PrimaryKey<Self::References>;
}

// ── HasPrimaryKey — looked up by the `table!` macro for FK RefColumn ─────────

pub trait HasPrimaryKey: TableSchema + Sized {
    type PkColumn: PrimaryKey<Self>;
}

// ── Join clause ───────────────────────────────────────────────────────────────

use crate::{BelongsTo, NotSealed, QueryBuilder, TableSchema, WithColumns, select::NotNull};

#[derive(Clone)]
pub enum JoinClause {
    Inner {
        table_name_a: String,
        table_name_b: String,
        table_fk: String,
        table_fk_ref: String,
    },
    Left {
        table_name_a: String,
        table_name_b: String,
        table_fk: String,
        table_fk_ref: String,
    },
}


impl JoinClause {
    pub(crate)  fn to_sql(self) -> String {
        match self {
            Self::Inner {
                table_name_a,
                table_name_b,
                table_fk,
                table_fk_ref,
            } => format!(
                "INNER JOIN {} ON {}.{} = {}.{}",
                table_name_b, table_name_a, table_fk, table_name_b, table_fk_ref,
            ),
            Self::Left {
                table_name_a,
                table_name_b,
                table_fk,
                table_fk_ref,
            } => format!(
                "LEFT JOIN {} ON {}.{} = {}.{}",
                table_name_b, table_name_a, table_fk, table_name_b, table_fk_ref,
            ),
        }
    }
}

pub fn inner_join<A, B, FK>() -> JoinClause
where
    A: TableSchema,
    B: TableSchema,
    FK: ForeignKey<A, References = B>,
{
    JoinClause::Inner {
        table_name_a: A::TABLE_NAME.into(),
        table_name_b: B::TABLE_NAME.into(),
        table_fk: FK::COLUMN_NAME.into(),
        table_fk_ref: FK::RefColumn::COLUMN_NAME.into(),
    }
}

pub fn left_join<A, B, FK>() -> JoinClause
where
    A: TableSchema,
    B: TableSchema,
    FK: ForeignKey<A, References = B>,
{
    JoinClause::Left {
        table_name_a: A::TABLE_NAME.into(),
        table_name_b: B::TABLE_NAME.into(),
        table_fk: FK::COLUMN_NAME.into(),
        table_fk_ref: FK::RefColumn::COLUMN_NAME.into(),
    }
}
