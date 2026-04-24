use crate::{BelongsTo, NotSealed, QueryBuilder, TableSchema, WithColumns, WithTable};

impl<T: TableSchema, R> QueryBuilder<WithTable<T>, NotSealed, R> {
    pub fn select<Cols: ColumnSet<T>>(self) -> QueryBuilder<WithColumns<T>, NotSealed, R> {
        let mut q: QueryBuilder<WithColumns<T>, NotSealed, R> = self.cast();
        q.data.columns = Cols::column_names()
            .iter()
            .map(|c| format!("{}.{}", T::TABLE_NAME, c))
            .collect();
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

// ── ColumnSet ─────────────────────────────────────────────────────────────────
//
// Implemented for single columns via the (C,) 1-tuple, and for multi-column
// tuples. The blanket impl `for C` is omitted to avoid a coherence conflict
// with the tuple impls (Rust cannot rule out downstream `BelongsTo` impls on
// tuple types).
pub trait ColumnSet<T: TableSchema> {
    fn column_names() -> Vec<&'static str>;
}

macro_rules! impl_column_set {
    ($($C:ident),+) => {
        impl<T, $($C),+> ColumnSet<T> for ($($C,)+)
        where
            T: TableSchema,
            $($C: BelongsTo<T>,)+
        {
            fn column_names() -> Vec<&'static str> {
                vec![$($C::COLUMN_NAME,)+]
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
