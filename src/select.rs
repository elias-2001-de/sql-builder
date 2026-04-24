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

impl<T: TableSchema, C: BelongsTo<T>> ColumnSet<T> for (C,) {
    fn column_names() -> Vec<&'static str> {
        vec![C::COLUMN_NAME]
    }
}
impl<T, C1, C2> ColumnSet<T> for (C1, C2)
where
    T: TableSchema,
    C1: BelongsTo<T>,
    C2: BelongsTo<T>,
{
    fn column_names() -> Vec<&'static str> {
        vec![C1::COLUMN_NAME, C2::COLUMN_NAME]
    }
}
impl<T, C1, C2, C3> ColumnSet<T> for (C1, C2, C3)
where
    T: TableSchema,
    C1: BelongsTo<T>,
    C2: BelongsTo<T>,
    C3: BelongsTo<T>,
{
    fn column_names() -> Vec<&'static str> {
        vec![C1::COLUMN_NAME, C2::COLUMN_NAME, C3::COLUMN_NAME]
    }
}
impl<T, C1, C2, C3, C4> ColumnSet<T> for (C1, C2, C3, C4)
where
    T: TableSchema,
    C1: BelongsTo<T>,
    C2: BelongsTo<T>,
    C3: BelongsTo<T>,
    C4: BelongsTo<T>,
{
    fn column_names() -> Vec<&'static str> {
        vec![
            C1::COLUMN_NAME,
            C2::COLUMN_NAME,
            C3::COLUMN_NAME,
            C4::COLUMN_NAME,
        ]
    }
}
impl<T, C1, C2, C3, C4, C5> ColumnSet<T> for (C1, C2, C3, C4, C5)
where
    T: TableSchema,
    C1: BelongsTo<T>,
    C2: BelongsTo<T>,
    C3: BelongsTo<T>,
    C4: BelongsTo<T>,
    C5: BelongsTo<T>,
{
    fn column_names() -> Vec<&'static str> {
        vec![
            C1::COLUMN_NAME,
            C2::COLUMN_NAME,
            C3::COLUMN_NAME,
            C4::COLUMN_NAME,
            C5::COLUMN_NAME,
        ]
    }
}

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

#[allow(dead_code)]
pub trait UniqueColumn<T: TableSchema>: BelongsTo<T> {}
