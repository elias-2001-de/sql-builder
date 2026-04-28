use std::marker::PhantomData;

use crate::{AllColumns, BelongsTo, ColumnExpr, NotSealed, QueryBuilder, Subquery, TableSchema, WithColumns, WithTable};

impl<T: TableSchema, R, Row> QueryBuilder<WithTable<T>, NotSealed, R, Row> {
    pub fn select<Cols: ColumnSet<T>>(self) -> QueryBuilder<WithColumns<T, Cols>, NotSealed, R, Row> {
        let mut q: QueryBuilder<WithColumns<T, Cols>, NotSealed, R, Row> = self.cast();
        q.data.columns = Cols::column_exprs();
        q
    }
    pub fn select_all(self) -> QueryBuilder<WithColumns<T, AllColumns>, NotSealed, R, Row> {
        let mut q: QueryBuilder<WithColumns<T, AllColumns>, NotSealed, R, Row> = self.cast();
        q.data.columns = vec![ColumnExpr::All];
        q
    }
}

impl<T, C, S, R, Row> QueryBuilder<WithColumns<T, (C,)>, S, R, Row>
where
    T: TableSchema,
    C: BelongsTo<T>,
{
    pub fn into_subquery(self) -> Subquery<<C as BelongsTo<T>>::Value> {
        Subquery {
            data: self.data,
            _phantom: PhantomData,
        }
    }
}

// ── Nullability markers ───────────────────────────────────────────────────────

pub struct NotNull;
pub struct Nullable;

// ── SelectExpr ────────────────────────────────────────────────────────────────

pub trait SelectExpr<T: TableSchema> {
    fn column_expr() -> ColumnExpr;
}

// ── Aggregate types ───────────────────────────────────────────────────────────

pub struct Count;

impl<T: TableSchema> SelectExpr<T> for Count {
    fn column_expr() -> ColumnExpr {
        ColumnExpr::Count
    }
}

pub struct Max<C>(PhantomData<C>);

impl<T: TableSchema, C: BelongsTo<T>> SelectExpr<T> for Max<C> {
    fn column_expr() -> ColumnExpr {
        ColumnExpr::Max { table: T::TABLE_NAME, name: C::COLUMN_NAME }
    }
}

pub struct Min<C>(PhantomData<C>);

impl<T: TableSchema, C: BelongsTo<T>> SelectExpr<T> for Min<C> {
    fn column_expr() -> ColumnExpr {
        ColumnExpr::Min { table: T::TABLE_NAME, name: C::COLUMN_NAME }
    }
}

pub struct Sum<C>(PhantomData<C>);

impl<T: TableSchema, C: BelongsTo<T>> SelectExpr<T> for Sum<C> {
    fn column_expr() -> ColumnExpr {
        ColumnExpr::Sum { table: T::TABLE_NAME, name: C::COLUMN_NAME }
    }
}

// ── ColumnSet ─────────────────────────────────────────────────────────────────

pub trait ColumnSet<T: TableSchema> {
    fn column_exprs() -> Vec<ColumnExpr>;
}

macro_rules! impl_column_set {
    ($($C:ident),+) => {
        impl<T, $($C),+> ColumnSet<T> for ($($C,)+)
        where
            T: TableSchema,
            $($C: SelectExpr<T>,)+
        {
            fn column_exprs() -> Vec<ColumnExpr> {
                vec![$($C::column_expr(),)+]
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
