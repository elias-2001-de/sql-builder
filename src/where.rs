use std::marker::PhantomData;

use crate::{
    NotSealed, QueryBuilder, TableSchema, WithColumns,
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
    pub(crate) fn to_sql(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => format!("'{}'", s.replace('\'', "''")),
            Value::Bool(b) => {
                if *b {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
        }
    }
}

pub trait IntoValue {
    fn into_value(self) -> Value;
}

impl IntoValue for isize {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}
impl IntoValue for usize {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}
impl IntoValue for u32 {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}
impl IntoValue for u64 {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}
impl IntoValue for i32 {
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}
impl IntoValue for i64 {
    fn into_value(self) -> Value {
        Value::Int(self)
    }
}
impl IntoValue for f32 {
    fn into_value(self) -> Value {
        Value::Float(self as f64)
    }
}
impl IntoValue for f64 {
    fn into_value(self) -> Value {
        Value::Float(self)
    }
}
impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}
impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::Text(self)
    }
}
impl IntoValue for &str {
    fn into_value(self) -> Value {
        Value::Text(self.to_owned())
    }
}

// ── WhereClause typestates ────────────────────────────────────────────────────

pub struct NoCondition;
pub struct HasCondition;
pub struct NeedsOperand;

// ── WhereClause builder ───────────────────────────────────────────────────────

pub struct WhereClause<T: TableSchema, State> {
    fragments: Vec<String>,
    _table: PhantomData<T>,
    _state: PhantomData<State>,
}

impl<T: TableSchema> WhereClause<T, NoCondition> {
    pub fn new() -> Self {
        Self {
            fragments: Vec::new(),
            _table: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<T: TableSchema> Default for WhereClause<T, NoCondition> {
    fn default() -> Self {
        Self::new()
    }
}

fn wc_transition<T: TableSchema, S>(frags: Vec<String>) -> WhereClause<T, S> {
    WhereClause {
        fragments: frags,
        _table: PhantomData,
        _state: PhantomData,
    }
}

macro_rules! impl_where_predicates {
    ($State:ty) => {
        impl<T: TableSchema> WhereClause<T, $State> {
            pub fn eq<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(format!(
                    "{} = {}",
                    C::COLUMN_NAME,
                    value.into_value().to_sql()
                ));
                wc_transition(self.fragments)
            }
            pub fn not_eq<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(format!(
                    "{} <> {}",
                    C::COLUMN_NAME,
                    value.into_value().to_sql()
                ));
                wc_transition(self.fragments)
            }
            pub fn lt<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(format!(
                    "{} < {}",
                    C::COLUMN_NAME,
                    value.into_value().to_sql()
                ));
                wc_transition(self.fragments)
            }
            pub fn lt_eq<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(format!(
                    "{} <= {}",
                    C::COLUMN_NAME,
                    value.into_value().to_sql()
                ));
                wc_transition(self.fragments)
            }
            pub fn gt<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(format!(
                    "{} > {}",
                    C::COLUMN_NAME,
                    value.into_value().to_sql()
                ));
                wc_transition(self.fragments)
            }
            pub fn gt_eq<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(format!(
                    "{} >= {}",
                    C::COLUMN_NAME,
                    value.into_value().to_sql()
                ));
                wc_transition(self.fragments)
            }
            pub fn like<C, V>(mut self, pattern: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(format!(
                    "{} LIKE {}",
                    C::COLUMN_NAME,
                    pattern.into_value().to_sql()
                ));
                wc_transition(self.fragments)
            }
            pub fn is_null<C>(mut self) -> WhereClause<T, HasCondition>
            where
                C: NullableColumn<T>,
            {
                self.fragments.push(format!("{} IS NULL", C::COLUMN_NAME));
                wc_transition(self.fragments)
            }
            pub fn is_not_null<C>(mut self) -> WhereClause<T, HasCondition>
            where
                C: NullableColumn<T>,
            {
                self.fragments
                    .push(format!("{} IS NOT NULL", C::COLUMN_NAME));
                wc_transition(self.fragments)
            }
            pub fn between<C, V>(mut self, lo: V, hi: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(format!(
                    "{} BETWEEN {} AND {}",
                    C::COLUMN_NAME,
                    lo.into_value().to_sql(),
                    hi.into_value().to_sql(),
                ));
                wc_transition(self.fragments)
            }
            pub fn in_values<C, V>(
                mut self,
                values: impl IntoIterator<Item = V>,
            ) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                let list: Vec<String> = values
                    .into_iter()
                    .map(|v| v.into_value().to_sql())
                    .collect();
                self.fragments
                    .push(format!("{} IN ({})", C::COLUMN_NAME, list.join(", ")));
                wc_transition(self.fragments)
            }
            pub fn in_subquery<C>(mut self, sql: impl Into<String>) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
            {
                self.fragments
                    .push(format!("{} IN ({})", C::COLUMN_NAME, sql.into()));
                wc_transition(self.fragments)
            }
            pub fn not_in_subquery<C>(
                mut self,
                sql: impl Into<String>,
            ) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
            {
                self.fragments
                    .push(format!("{} NOT IN ({})", C::COLUMN_NAME, sql.into()));
                wc_transition(self.fragments)
            }
            pub fn exists(mut self, sql: impl Into<String>) -> WhereClause<T, HasCondition> {
                self.fragments.push(format!("EXISTS ({})", sql.into()));
                wc_transition(self.fragments)
            }
            pub fn not_exists(mut self, sql: impl Into<String>) -> WhereClause<T, HasCondition> {
                self.fragments.push(format!("NOT EXISTS ({})", sql.into()));
                wc_transition(self.fragments)
            }
        }
    };
}

impl_where_predicates!(NoCondition);
impl_where_predicates!(NeedsOperand);

impl<T: TableSchema> WhereClause<T, HasCondition> {
    pub fn and(mut self) -> WhereClause<T, NeedsOperand> {
        self.fragments.push("AND".to_owned());
        wc_transition(self.fragments)
    }
    pub fn or(mut self) -> WhereClause<T, NeedsOperand> {
        self.fragments.push("OR".to_owned());
        wc_transition(self.fragments)
    }
    pub(crate) fn build_fragment(self) -> String {
        self.fragments.join(" ")
    }
}

// ── QueryBuilder integration ──────────────────────────────────────────────────

impl<T: TableSchema, R> QueryBuilder<WithColumns<T>, NotSealed, R> {
    pub fn where_clause(mut self, clause: WhereClause<T, HasCondition>) -> Self {
        self.data
            .conditions
            .push(format!("({})", clause.build_fragment()));
        self
    }
}
