use std::marker::PhantomData;

use crate::{BelongsTo, NoTable, TableSchema, WithTable};
use crate::execute::{Runner, UpdateData};
use crate::r#where::{ConditionAtom, HasCondition, IntoValue, Value, WhereClause};
use crate::select::NullableColumn;

struct UpdateBuilderData {
    table: Option<&'static str>,
    assignments: Vec<(&'static str, Option<Value>)>,
    conditions: Vec<Vec<ConditionAtom>>,
}

pub struct WithSet<T>(PhantomData<T>);

pub struct UpdateBuilder<Phase> {
    data: UpdateBuilderData,
    _phase: PhantomData<Phase>,
}

impl Default for UpdateBuilder<NoTable> {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateBuilder<NoTable> {
    pub fn new() -> Self {
        Self {
            data: UpdateBuilderData { table: None, assignments: Vec::new(), conditions: Vec::new() },
            _phase: PhantomData,
        }
    }

    pub fn table<T: TableSchema>(self) -> UpdateBuilder<WithTable<T>> {
        UpdateBuilder {
            data: UpdateBuilderData { table: Some(T::TABLE_NAME), ..self.data },
            _phase: PhantomData,
        }
    }
}

impl<T: TableSchema> UpdateBuilder<WithTable<T>> {
    pub fn set<C, V>(mut self, val: V) -> UpdateBuilder<WithSet<T>>
    where
        C: BelongsTo<T>,
        V: IntoValue,
    {
        self.data.assignments.push((C::COLUMN_NAME, Some(val.into_value())));
        UpdateBuilder { data: self.data, _phase: PhantomData }
    }
}

impl<T: TableSchema> UpdateBuilder<WithSet<T>> {
    pub fn set<C, V>(mut self, val: V) -> Self
    where
        C: BelongsTo<T>,
        V: IntoValue,
    {
        self.data.assignments.push((C::COLUMN_NAME, Some(val.into_value())));
        self
    }

    pub fn set_null<C>(mut self) -> Self
    where
        C: NullableColumn<T>,
    {
        self.data.assignments.push((C::COLUMN_NAME, None));
        self
    }

    pub fn where_clause(mut self, clause: WhereClause<T, HasCondition>) -> Self {
        self.data.conditions.push(clause.fragments);
        self
    }

    pub(crate) fn into_data(self) -> UpdateData {
        UpdateData {
            table: self.data.table.unwrap(),
            assignments: self.data.assignments,
            conditions: self.data.conditions,
        }
    }

    pub fn execute(self, runner: &impl Runner<()>) -> Result<(), Box<dyn std::error::Error>> {
        runner.update(self.into_data())
    }
}
