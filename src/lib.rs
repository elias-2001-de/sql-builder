// =============================================================================
//  Type-safe SQL Builder — PK / FK encoded in the type system
// =============================================================================

#![allow(unused)]

use std::marker::PhantomData;

// ── Nullability markers ───────────────────────────────────────────────────────

pub struct NotNull;
pub struct Nullable;

// ── Builder phases ────────────────────────────────────────────────────────────

pub struct NoTable;
pub struct WithTable<T>(PhantomData<T>);
pub struct WithColumns<T>(PhantomData<T>);

// ── Core table / column traits ────────────────────────────────────────────────

pub trait TableSchema {
    const TABLE_NAME: &'static str;
}

pub trait BelongsTo<T: TableSchema> {
    type Value;
    type Null;
    const COLUMN_NAME: &'static str;
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

// ── Schema macro ──────────────────────────────────────────────────────────────
//
//  Syntax: `table! { mod_name: TableType => "sql_name" { columns... } }`
//
//  Column syntax:
//    col*:      Type   →  PRIMARY KEY, NOT NULL
//    col:       Type   →  NOT NULL
//    col?:      Type   →  NULLABLE
//    col->      Table: Type  →  FOREIGN KEY → Table's PK, NOT NULL
//    col?->     Table: Type  →  FOREIGN KEY → Table's PK, NULLABLE
//
//  Each table's column structs live in `pub mod mod_name`, which prevents
//  name collisions when the same logical column name (e.g. `AuthorId`) appears
//  in multiple tables.
//
//  Implemented as a TT muncher to avoid the ambiguity that arises when
//  `$($rest:tt)*` is nested inside `$(...),*` (rejected in edition 2024).

#[macro_export]
macro_rules! table {
    ($mod_name:ident: $table:ident => $table_name:literal { $($body:tt)* }) => {
        pub struct $table;
        impl $crate::TableSchema for $table {
            const TABLE_NAME: &'static str = $table_name;
        }
        pub mod $mod_name {
            #[allow(unused_imports)]
            use super::*;
            $crate::table!(@col $table $($body)*);
        }
    };

    (@col $table:ident) => {};

    // FK NULLABLE: col?-> RefTable: Type
    (@col $table:ident $col:ident ?-> $ref_table:ident : $vtype:ty $(, $($rest:tt)*)?) => {
        pub struct $col;
        impl $crate::BelongsTo<$table> for $col {
            type Value = $vtype;
            type Null  = $crate::Nullable;
            const COLUMN_NAME: &'static str = stringify!($col);
        }
        impl $crate::ForeignKey<$table> for $col {
            type References = $ref_table;
            type RefColumn  = <$ref_table as $crate::HasPrimaryKey>::PkColumn;
        }
        $($crate::table!(@col $table $($rest)*);)?
    };

    // NULLABLE: col?: Type
    (@col $table:ident $col:ident ?: $vtype:ty $(, $($rest:tt)*)?) => {
        pub struct $col;
        impl $crate::BelongsTo<$table> for $col {
            type Value = $vtype;
            type Null  = $crate::Nullable;
            const COLUMN_NAME: &'static str = stringify!($col);
        }
        $($crate::table!(@col $table $($rest)*);)?
    };

    // FK NOT NULL: col-> RefTable: Type
    (@col $table:ident $col:ident -> $ref_table:ident : $vtype:ty $(, $($rest:tt)*)?) => {
        pub struct $col;
        impl $crate::BelongsTo<$table> for $col {
            type Value = $vtype;
            type Null  = $crate::NotNull;
            const COLUMN_NAME: &'static str = stringify!($col);
        }
        impl $crate::ForeignKey<$table> for $col {
            type References = $ref_table;
            type RefColumn  = <$ref_table as $crate::HasPrimaryKey>::PkColumn;
        }
        $($crate::table!(@col $table $($rest)*);)?
    };

    // PRIMARY KEY: col*: Type
    (@col $table:ident $col:ident *: $vtype:ty $(, $($rest:tt)*)?) => {
        pub struct $col;
        impl $crate::BelongsTo<$table> for $col {
            type Value = $vtype;
            type Null  = $crate::NotNull;
            const COLUMN_NAME: &'static str = stringify!($col);
        }
        impl $crate::PrimaryKey<$table> for $col {}
        $($crate::table!(@col $table $($rest)*);)?
    };

    // NOT NULL: col: Type
    (@col $table:ident $col:ident : $vtype:ty $(, $($rest:tt)*)?) => {
        pub struct $col;
        impl $crate::BelongsTo<$table> for $col {
            type Value = $vtype;
            type Null  = $crate::NotNull;
            const COLUMN_NAME: &'static str = stringify!($col);
        }
        $($crate::table!(@col $table $($rest)*);)?
    };
}

#[macro_export]
macro_rules! impl_has_pk {
    ($table:ident, $pk_col:ty) => {
        impl $crate::HasPrimaryKey for $table {
            type PkColumn = $pk_col;
        }
    };
}

// ── Condition helpers ─────────────────────────────────────────────────────────

pub struct Condition<T: TableSchema> {
    pub sql: String,
    _t: PhantomData<T>,
}

pub fn cond<T, C>(op: &str, value: &str) -> Condition<T>
where
    T: TableSchema,
    C: NotNullColumn<T>,
{
    Condition {
        sql: format!("{} {} {}", C::COLUMN_NAME, op, value),
        _t: PhantomData,
    }
}
pub fn eq<T: TableSchema, C: NotNullColumn<T>>(v: &str) -> Condition<T> {
    cond::<T, C>("=", v)
}
pub fn gt<T: TableSchema, C: NotNullColumn<T>>(v: &str) -> Condition<T> {
    cond::<T, C>(">", v)
}
pub fn lt<T: TableSchema, C: NotNullColumn<T>>(v: &str) -> Condition<T> {
    cond::<T, C>("<", v)
}
pub fn like<T: TableSchema, C: NotNullColumn<T>>(v: &str) -> Condition<T> {
    cond::<T, C>("LIKE", v)
}

pub fn is_null<T, C>() -> Condition<T>
where
    T: TableSchema,
    C: NullableColumn<T>,
{
    Condition {
        sql: format!("{} IS NULL", C::COLUMN_NAME),
        _t: PhantomData,
    }
}
pub fn is_not_null<T, C>() -> Condition<T>
where
    T: TableSchema,
    C: NullableColumn<T>,
{
    Condition {
        sql: format!("{} IS NOT NULL", C::COLUMN_NAME),
        _t: PhantomData,
    }
}

/// Typed equality check on any column — the value type is checked at compile time.
pub fn typed_eq<T, C>(id: C::Value) -> Condition<T>
where
    T: TableSchema,
    C: BelongsTo<T>,
    C::Value: std::fmt::Display,
{
    Condition {
        sql: format!("{} = {}", C::COLUMN_NAME, id),
        _t: PhantomData,
    }
}

/// Typed equality check restricted to FK columns.
pub fn fk_eq<T, FK>(id: FK::Value) -> Condition<T>
where
    T: TableSchema,
    FK: ForeignKey<T>,
    FK::Value: std::fmt::Display,
{
    Condition {
        sql: format!("{} = {}", FK::COLUMN_NAME, id),
        _t: PhantomData,
    }
}

// ── Order direction ───────────────────────────────────────────────────────────

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

// ── Join clause ───────────────────────────────────────────────────────────────

pub struct JoinClause {
    pub sql: String,
}

pub fn inner_join<A, B, FK>() -> JoinClause
where
    A: TableSchema,
    B: TableSchema,
    FK: ForeignKey<A, References = B>,
{
    JoinClause {
        sql: format!(
            "INNER JOIN {} ON {}.{} = {}.{}",
            B::TABLE_NAME,
            A::TABLE_NAME,
            FK::COLUMN_NAME,
            B::TABLE_NAME,
            FK::RefColumn::COLUMN_NAME,
        ),
    }
}

pub fn left_join<A, B, FK>() -> JoinClause
where
    A: TableSchema,
    B: TableSchema,
    FK: ForeignKey<A, References = B>,
{
    JoinClause {
        sql: format!(
            "LEFT JOIN {} ON {}.{} = {}.{}",
            B::TABLE_NAME,
            A::TABLE_NAME,
            FK::COLUMN_NAME,
            B::TABLE_NAME,
            FK::RefColumn::COLUMN_NAME,
        ),
    }
}

// ── Query builder ─────────────────────────────────────────────────────────────

pub struct QueryBuilder<Phase> {
    table: Option<&'static str>,
    columns: Vec<String>,
    joins: Vec<String>,
    conditions: Vec<String>,
    order_by: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    _phase: PhantomData<Phase>,
}

impl QueryBuilder<NoTable> {
    pub fn new() -> Self {
        QueryBuilder {
            table: None,
            columns: Vec::new(),
            joins: Vec::new(),
            conditions: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
            _phase: PhantomData,
        }
    }
    pub fn from<T: TableSchema>(self) -> QueryBuilder<WithTable<T>> {
        let mut q: QueryBuilder<WithTable<T>> = self.cast();
        q.table = Some(T::TABLE_NAME);
        q
    }
}

impl<T: TableSchema> QueryBuilder<WithTable<T>> {
    pub fn select<Cols: ColumnSet<T>>(self) -> QueryBuilder<WithColumns<T>> {
        let mut q: QueryBuilder<WithColumns<T>> = self.cast();
        q.columns = Cols::column_names()
            .iter()
            .map(|c| format!("{}.{}", T::TABLE_NAME, c))
            .collect();
        q
    }
    pub fn select_all(self) -> QueryBuilder<WithColumns<T>> {
        let mut q: QueryBuilder<WithColumns<T>> = self.cast();
        q.columns = vec!["*".to_string()];
        q
    }
}

impl<T: TableSchema> QueryBuilder<WithColumns<T>> {
    pub fn join<B, FK>(mut self) -> Self
    where
        B: TableSchema,
        FK: ForeignKey<T, References = B>,
    {
        self.joins.push(inner_join::<T, B, FK>().sql);
        self
    }

    pub fn left_join<B, FK>(mut self) -> Self
    where
        B: TableSchema,
        FK: ForeignKey<T, References = B>,
    {
        self.joins.push(left_join::<T, B, FK>().sql);
        self
    }

    pub fn where_col(mut self, c: Condition<T>) -> Self {
        self.conditions.push(c.sql);
        self
    }
    pub fn where_raw(mut self, raw: &str) -> Self {
        self.conditions.push(raw.to_string());
        self
    }
    pub fn order_by<C: BelongsTo<T>>(mut self, dir: Direction) -> Self {
        self.order_by = Some(format!("{} {}", C::COLUMN_NAME, dir.sql()));
        self
    }
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }
    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }

    pub fn build(self) -> String {
        let cols = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };
        let table = self.table.unwrap();
        let mut sql = format!("SELECT {cols} FROM {table}");

        for join in &self.joins {
            sql.push(' ');
            sql.push_str(join);
        }

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }
        if let Some(o) = self.order_by {
            sql.push_str(&format!(" ORDER BY {o}"));
        }
        if let Some(l) = self.limit {
            sql.push_str(&format!(" LIMIT {l}"));
        }
        if let Some(o) = self.offset {
            sql.push_str(&format!(" OFFSET {o}"));
        }
        sql
    }
}

impl<A> QueryBuilder<A> {
    fn cast<B>(self) -> QueryBuilder<B> {
        QueryBuilder {
            table: self.table,
            columns: self.columns,
            joins: self.joins,
            conditions: self.conditions,
            order_by: self.order_by,
            limit: self.limit,
            offset: self.offset,
            _phase: PhantomData,
        }
    }
}

pub fn select() -> QueryBuilder<NoTable> {
    QueryBuilder::new()
}
