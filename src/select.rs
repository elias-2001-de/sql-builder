use std::marker::PhantomData;

use crate::{BelongsTo, NotSealed, QueryBuilder, TableSchema, WithColumns, WithTable};

impl<T: TableSchema, R> QueryBuilder<WithTable<T>, NotSealed, R> {
    pub fn select<Cols: ColumnSet<T>>(self) -> QueryBuilder<WithColumns<T>, NotSealed, R> {
        let mut q: QueryBuilder<WithColumns<T>, NotSealed, R> = self.cast();
        q.data.columns = Cols::sql_exprs();
        q
    }
    pub fn select_all(self) -> QueryBuilder<WithColumns<T>, NotSealed, R> {
        let mut q: QueryBuilder<WithColumns<T>, NotSealed, R> = self.cast();
        q.data.columns = vec!["*".to_string()];
        q
    }
}

// ── Nullability markers ───────────────────────────────────────────────────────

pub struct NotNull;
pub struct Nullable;

// ── SelectExpr ────────────────────────────────────────────────────────────────
//
// Produces the SQL fragment for a single select item. Regular columns implement
// this via the derive macro (emitting `"table.col"`). Aggregate types implement
// it directly. No blanket impl is used to avoid coherence conflicts.

pub trait SelectExpr<T: TableSchema> {
    fn sql_expr() -> String;
}

// ── Aggregate types ───────────────────────────────────────────────────────────

pub struct Count;

impl<T: TableSchema> SelectExpr<T> for Count {
    fn sql_expr() -> String {
        "COUNT(*)".to_string()
    }
}

pub struct Max<C>(PhantomData<C>);

impl<T: TableSchema, C: BelongsTo<T>> SelectExpr<T> for Max<C> {
    fn sql_expr() -> String {
        format!("MAX({}.{})", T::TABLE_NAME, C::COLUMN_NAME)
    }
}

pub struct Min<C>(PhantomData<C>);

impl<T: TableSchema, C: BelongsTo<T>> SelectExpr<T> for Min<C> {
    fn sql_expr() -> String {
        format!("MIN({}.{})", T::TABLE_NAME, C::COLUMN_NAME)
    }
}

pub struct Sum<C>(PhantomData<C>);

impl<T: TableSchema, C: BelongsTo<T>> SelectExpr<T> for Sum<C> {
    fn sql_expr() -> String {
        format!("SUM({}.{})", T::TABLE_NAME, C::COLUMN_NAME)
    }
}

// ── ColumnSet ─────────────────────────────────────────────────────────────────
//
// Implemented for tuples of SelectExpr items. This allows mixing regular
// columns and aggregate expressions in a single select call. The blanket impl
// `for C` is omitted to avoid a coherence conflict with the tuple impls.

pub trait ColumnSet<T: TableSchema> {
    fn sql_exprs() -> Vec<String>;
}

macro_rules! impl_column_set {
    ($($C:ident),+) => {
        impl<T, $($C),+> ColumnSet<T> for ($($C,)+)
        where
            T: TableSchema,
            $($C: SelectExpr<T>,)+
        {
            fn sql_exprs() -> Vec<String> {
                vec![$($C::sql_expr(),)+]
            }
        }
    };
}

impl_column_set!(C1);
impl_column_set!(C1, C2);
impl_column_set!(C1, C2, C3);
impl_column_set!(C1, C2, C3, C4);
impl_column_set!(C1, C2, C3, C4, C5);
impl_column_set!(C1, C2, C3, C4, C5, C6);
impl_column_set!(C1, C2, C3, C4, C5, C6, C7);
impl_column_set!(C1, C2, C3, C4, C5, C6, C7, C8);
impl_column_set!(C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_column_set!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_column_set!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_column_set!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_column_set!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_column_set!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
impl_column_set!(
    C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15
);
impl_column_set!(
    C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16
);

// if you need more your desing is shit

// ── Nullability helpers ───────────────────────────────────────────────────────

pub trait NotNullColumn<T: TableSchema>: BelongsTo<T, Null = NotNull> {}
impl<T, C> NotNullColumn<T> for C
where
    T: TableSchema,
    C: BelongsTo<T, Null = NotNull>,
{
}

pub trait NullableColumn<T: TableSchema>: BelongsTo<T, Null = Nullable> {}
impl<T, C> NullableColumn<T> for C
where
    T: TableSchema,
    C: BelongsTo<T, Null = Nullable>,
{
}

pub trait UniqueColumn<T: TableSchema>: BelongsTo<T> {}
