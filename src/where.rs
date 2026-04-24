use std::marker::PhantomData;

use crate::{
    BelongsTo, NotSealed, QueryBuilder, TableSchema, WithColumns,
    join::ForeignKey,
    select::{NotNullColumn, NullableColumn},
};

// ── Typed value ───────────────────────────────────────────────────────────────

pub enum Value {
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
}

impl Value {
    fn to_sql(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => format!("'{}'", s.replace('\'', "''")),
            Value::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
        }
    }
}

pub trait IntoValue {
    fn into_value(self) -> Value;
}
impl IntoValue for i32   { fn into_value(self) -> Value { Value::Int(self as i64) } }
impl IntoValue for i64   { fn into_value(self) -> Value { Value::Int(self) } }
impl IntoValue for f32   { fn into_value(self) -> Value { Value::Float(self as f64) } }
impl IntoValue for f64   { fn into_value(self) -> Value { Value::Float(self) } }
impl IntoValue for bool  { fn into_value(self) -> Value { Value::Bool(self) } }
impl IntoValue for String { fn into_value(self) -> Value { Value::Text(self) } }
impl IntoValue for &str  { fn into_value(self) -> Value { Value::Text(self.to_owned()) } }

// ── Operator ──────────────────────────────────────────────────────────────────

enum Op {
    Eq,
    Gt,
    Lt,
    Like,
}

impl Op {
    fn sql(&self) -> &str {
        match self {
            Op::Eq   => "=",
            Op::Gt   => ">",
            Op::Lt   => "<",
            Op::Like => "LIKE",
        }
    }
}

// ── Condition ─────────────────────────────────────────────────────────────────

enum ConditionKind {
    Compare { op: Op, value: Value },
    IsNull,
    IsNotNull,
}

pub struct Condition<T: TableSchema> {
    column: &'static str,
    kind: ConditionKind,
    _t: PhantomData<T>,
}

impl<T: TableSchema> Condition<T> {
    pub(crate) fn to_sql(&self) -> String {
        match &self.kind {
            ConditionKind::Compare { op, value } => {
                format!("{} {} {}", self.column, op.sql(), value.to_sql())
            }
            ConditionKind::IsNull    => format!("{} IS NULL", self.column),
            ConditionKind::IsNotNull => format!("{} IS NOT NULL", self.column),
        }
    }
}

// ── Builder impl ──────────────────────────────────────────────────────────────

impl<T: TableSchema, R> QueryBuilder<WithColumns<T>, NotSealed, R> {
    pub fn where_col(mut self, c: Condition<T>) -> Self {
        self.data.conditions.push(c.to_sql());
        self
    }
    pub fn where_raw(mut self, raw: &str) -> Self {
        self.data.conditions.push(raw.to_string());
        self
    }
}

// ── Constructor functions ─────────────────────────────────────────────────────

fn compare<T, C>(op: Op, value: Value) -> Condition<T>
where
    T: TableSchema,
    C: NotNullColumn<T>,
{
    Condition { column: C::COLUMN_NAME, kind: ConditionKind::Compare { op, value }, _t: PhantomData }
}

pub fn eq<T: TableSchema, C: NotNullColumn<T>>(v: impl IntoValue) -> Condition<T> {
    compare::<T, C>(Op::Eq, v.into_value())
}
pub fn gt<T: TableSchema, C: NotNullColumn<T>>(v: impl IntoValue) -> Condition<T> {
    compare::<T, C>(Op::Gt, v.into_value())
}
pub fn lt<T: TableSchema, C: NotNullColumn<T>>(v: impl IntoValue) -> Condition<T> {
    compare::<T, C>(Op::Lt, v.into_value())
}
pub fn like<T: TableSchema, C: NotNullColumn<T>>(v: impl IntoValue) -> Condition<T> {
    compare::<T, C>(Op::Like, v.into_value())
}

pub fn is_null<T, C>() -> Condition<T>
where
    T: TableSchema,
    C: NullableColumn<T>,
{
    Condition { column: C::COLUMN_NAME, kind: ConditionKind::IsNull, _t: PhantomData }
}
pub fn is_not_null<T, C>() -> Condition<T>
where
    T: TableSchema,
    C: NullableColumn<T>,
{
    Condition { column: C::COLUMN_NAME, kind: ConditionKind::IsNotNull, _t: PhantomData }
}

pub fn typed_eq<T, C>(value: C::Value) -> Condition<T>
where
    T: TableSchema,
    C: BelongsTo<T>,
    C::Value: IntoValue,
{
    Condition {
        column: C::COLUMN_NAME,
        kind: ConditionKind::Compare { op: Op::Eq, value: value.into_value() },
        _t: PhantomData,
    }
}

pub fn fk_eq<T, FK>(value: FK::Value) -> Condition<T>
where
    T: TableSchema,
    FK: ForeignKey<T>,
    FK::Value: IntoValue,
{
    Condition {
        column: FK::COLUMN_NAME,
        kind: ConditionKind::Compare { op: Op::Eq, value: value.into_value() },
        _t: PhantomData,
    }
}
