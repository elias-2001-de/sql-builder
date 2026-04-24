use std::marker::PhantomData;

use crate::{BelongsTo, NoTable, TableSchema, WithTable};
use crate::r#where::{HasCondition, IntoValue, WhereClause};
use crate::select::NullableColumn;

struct UpdateData {
    table: Option<&'static str>,
    assignments: Vec<String>,
    conditions: Vec<String>,
}

pub struct WithSet<T>(PhantomData<T>);

pub struct UpdateBuilder<Phase> {
    data: UpdateData,
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
            data: UpdateData { table: None, assignments: Vec::new(), conditions: Vec::new() },
            _phase: PhantomData,
        }
    }

    pub fn table<T: TableSchema>(self) -> UpdateBuilder<WithTable<T>> {
        UpdateBuilder {
            data: UpdateData { table: Some(T::TABLE_NAME), ..self.data },
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
        self.data.assignments.push(format!(
            "{} = {}",
            C::COLUMN_NAME,
            val.into_value().to_sql(),
        ));
        UpdateBuilder { data: self.data, _phase: PhantomData }
    }
}

impl<T: TableSchema> UpdateBuilder<WithSet<T>> {
    pub fn set<C, V>(mut self, val: V) -> Self
    where
        C: BelongsTo<T>,
        V: IntoValue,
    {
        self.data.assignments.push(format!(
            "{} = {}",
            C::COLUMN_NAME,
            val.into_value().to_sql(),
        ));
        self
    }

    pub fn set_null<C>(mut self) -> Self
    where
        C: NullableColumn<T>,
    {
        self.data.assignments.push(format!("{} = NULL", C::COLUMN_NAME));
        self
    }

    pub fn where_clause(mut self, clause: WhereClause<T, HasCondition>) -> Self {
        self.data.conditions.push(format!("({})", clause.build_fragment()));
        self
    }

    pub fn build(self) -> String {
        let table = self.data.table.unwrap();
        let set = self.data.assignments.join(", ");
        let mut sql = format!("UPDATE {table} SET {set}");
        if !self.data.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.data.conditions.join(" AND "));
        }
        sql
    }
}
