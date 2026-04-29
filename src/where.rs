use std::marker::PhantomData;

use crate::{
    BelongsTo, NotSealed, QueryBuilder, Subquery, SubquerySql, TableSchema, WithColumns,
    query::QueryInternData,
    select::{NotNullColumn, NullableColumn},
};

// ── Typed value ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
}

impl Value {
    #[allow(dead_code)]
    pub(crate) fn to_sql(&self) -> String {
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

impl IntoValue for isize {
    fn into_value(self) -> Value { Value::Int(self as i64) }
}
impl IntoValue for usize {
    fn into_value(self) -> Value { Value::Int(self as i64) }
}
impl IntoValue for u32 {
    fn into_value(self) -> Value { Value::Int(self as i64) }
}
impl IntoValue for u64 {
    fn into_value(self) -> Value { Value::Int(self as i64) }
}
impl IntoValue for i32 {
    fn into_value(self) -> Value { Value::Int(self as i64) }
}
impl IntoValue for i64 {
    fn into_value(self) -> Value { Value::Int(self) }
}
impl IntoValue for f32 {
    fn into_value(self) -> Value { Value::Float(self as f64) }
}
impl IntoValue for f64 {
    fn into_value(self) -> Value { Value::Float(self) }
}
impl IntoValue for bool {
    fn into_value(self) -> Value { Value::Bool(self) }
}
impl IntoValue for String {
    fn into_value(self) -> Value { Value::Text(self) }
}
impl IntoValue for &str {
    fn into_value(self) -> Value { Value::Text(self.to_owned()) }
}

// ── Condition atom ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) enum ConditionAtom {
    Eq(&'static str, Value),
    NotEq(&'static str, Value),
    Lt(&'static str, Value),
    LtEq(&'static str, Value),
    Gt(&'static str, Value),
    GtEq(&'static str, Value),
    Like(&'static str, Value),
    IsNull(&'static str),
    IsNotNull(&'static str),
    Between(&'static str, Value, Value),
    InValues(&'static str, Vec<Value>),
    #[allow(dead_code)]
    InSubquery(&'static str, Box<QueryInternData>),
    #[allow(dead_code)]
    NotInSubquery(&'static str, Box<QueryInternData>),
    #[allow(dead_code)]
    Exists(Box<QueryInternData>),
    #[allow(dead_code)]
    NotExists(Box<QueryInternData>),
    And,
    Or,
}

impl ConditionAtom {
    #[allow(dead_code)]
    pub(crate) fn to_sql(&self) -> String {
        match self {
            ConditionAtom::Eq(col, val) => format!("{} = {}", col, val.to_sql()),
            ConditionAtom::NotEq(col, val) => format!("{} <> {}", col, val.to_sql()),
            ConditionAtom::Lt(col, val) => format!("{} < {}", col, val.to_sql()),
            ConditionAtom::LtEq(col, val) => format!("{} <= {}", col, val.to_sql()),
            ConditionAtom::Gt(col, val) => format!("{} > {}", col, val.to_sql()),
            ConditionAtom::GtEq(col, val) => format!("{} >= {}", col, val.to_sql()),
            ConditionAtom::Like(col, val) => format!("{} LIKE {}", col, val.to_sql()),
            ConditionAtom::IsNull(col) => format!("{} IS NULL", col),
            ConditionAtom::IsNotNull(col) => format!("{} IS NOT NULL", col),
            ConditionAtom::Between(col, lo, hi) => {
                format!("{} BETWEEN {} AND {}", col, lo.to_sql(), hi.to_sql())
            }
            ConditionAtom::InValues(col, values) => {
                let list: Vec<String> = values.iter().map(|v| v.to_sql()).collect();
                format!("{} IN ({})", col, list.join(", "))
            }
            ConditionAtom::InSubquery(_, _)
            | ConditionAtom::NotInSubquery(_, _)
            | ConditionAtom::Exists(_)
            | ConditionAtom::NotExists(_) => todo!("subquery SQL building"),
            ConditionAtom::And => "AND".to_string(),
            ConditionAtom::Or => "OR".to_string(),
        }
    }
}

// ── WhereClause typestates ────────────────────────────────────────────────────

pub struct NoCondition;
pub struct HasCondition;
pub struct NeedsOperand;

// ── WhereClause builder ───────────────────────────────────────────────────────

pub struct WhereClause<T: TableSchema, State> {
    pub(crate) fragments: Vec<ConditionAtom>,
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

fn wc_transition<T: TableSchema, S>(frags: Vec<ConditionAtom>) -> WhereClause<T, S> {
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
                self.fragments.push(ConditionAtom::Eq(C::COLUMN_NAME, value.into_value()));
                wc_transition(self.fragments)
            }
            pub fn not_eq<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(ConditionAtom::NotEq(C::COLUMN_NAME, value.into_value()));
                wc_transition(self.fragments)
            }
            pub fn lt<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(ConditionAtom::Lt(C::COLUMN_NAME, value.into_value()));
                wc_transition(self.fragments)
            }
            pub fn lt_eq<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(ConditionAtom::LtEq(C::COLUMN_NAME, value.into_value()));
                wc_transition(self.fragments)
            }
            pub fn gt<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(ConditionAtom::Gt(C::COLUMN_NAME, value.into_value()));
                wc_transition(self.fragments)
            }
            pub fn gt_eq<C, V>(mut self, value: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(ConditionAtom::GtEq(C::COLUMN_NAME, value.into_value()));
                wc_transition(self.fragments)
            }
            pub fn like<C, V>(mut self, pattern: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(ConditionAtom::Like(C::COLUMN_NAME, pattern.into_value()));
                wc_transition(self.fragments)
            }
            pub fn is_null<C>(mut self) -> WhereClause<T, HasCondition>
            where
                C: NullableColumn<T>,
            {
                self.fragments.push(ConditionAtom::IsNull(C::COLUMN_NAME));
                wc_transition(self.fragments)
            }
            pub fn is_not_null<C>(mut self) -> WhereClause<T, HasCondition>
            where
                C: NullableColumn<T>,
            {
                self.fragments.push(ConditionAtom::IsNotNull(C::COLUMN_NAME));
                wc_transition(self.fragments)
            }
            pub fn between<C, V>(mut self, lo: V, hi: V) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
                V: IntoValue,
            {
                self.fragments.push(ConditionAtom::Between(
                    C::COLUMN_NAME,
                    lo.into_value(),
                    hi.into_value(),
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
                let vals: Vec<Value> = values.into_iter().map(|v| v.into_value()).collect();
                self.fragments.push(ConditionAtom::InValues(C::COLUMN_NAME, vals));
                wc_transition(self.fragments)
            }
            pub fn in_subquery<C>(
                mut self,
                sq: Subquery<<C as BelongsTo<T>>::Value>,
            ) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
            {
                self.fragments.push(ConditionAtom::InSubquery(C::COLUMN_NAME, Box::new(sq.data)));
                wc_transition(self.fragments)
            }
            pub fn not_in_subquery<C>(
                mut self,
                sq: Subquery<<C as BelongsTo<T>>::Value>,
            ) -> WhereClause<T, HasCondition>
            where
                C: NotNullColumn<T>,
            {
                self.fragments.push(ConditionAtom::NotInSubquery(C::COLUMN_NAME, Box::new(sq.data)));
                wc_transition(self.fragments)
            }
            pub fn exists(mut self, sql: impl SubquerySql) -> WhereClause<T, HasCondition> {
                self.fragments.push(ConditionAtom::Exists(Box::new(sql.into_subquery_data())));
                wc_transition(self.fragments)
            }
            pub fn not_exists(mut self, sql: impl SubquerySql) -> WhereClause<T, HasCondition> {
                self.fragments.push(ConditionAtom::NotExists(Box::new(sql.into_subquery_data())));
                wc_transition(self.fragments)
            }
        }
    };
}

impl_where_predicates!(NoCondition);
impl_where_predicates!(NeedsOperand);

impl<T: TableSchema> WhereClause<T, HasCondition> {
    pub fn and(mut self) -> WhereClause<T, NeedsOperand> {
        self.fragments.push(ConditionAtom::And);
        wc_transition(self.fragments)
    }
    pub fn or(mut self) -> WhereClause<T, NeedsOperand> {
        self.fragments.push(ConditionAtom::Or);
        wc_transition(self.fragments)
    }
    #[allow(dead_code)]
    pub(crate) fn build_fragment(&self) -> String {
        self.fragments.iter().map(|a| a.to_sql()).collect::<Vec<_>>().join(" ")
    }
}

// ── QueryBuilder integration ──────────────────────────────────────────────────

impl<T: TableSchema, Cols, R, Row> QueryBuilder<WithColumns<T, Cols>, NotSealed, R, Row> {
    pub fn where_clause(mut self, clause: WhereClause<T, HasCondition>) -> Self {
        self.data.conditions.push(clause.fragments);
        self
    }
}
