// ── Condition helpers ─────────────────────────────────────────────────────────

use std::marker::PhantomData;

use crate::{BelongsTo, NotSealed, QueryBuilder, TableSchema, WithColumns, join::ForeignKey, select::{NotNullColumn, NullableColumn}};

impl<T: TableSchema, R> QueryBuilder<WithColumns<T>, NotSealed, R> {
    pub fn where_col(mut self, c: Condition<T>) -> Self {
        self.data.conditions.push(c.sql);
        self
    }
    pub fn where_raw(mut self, raw: &str) -> Self {
        self.data.conditions.push(raw.to_string());
        self
    }
}
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
