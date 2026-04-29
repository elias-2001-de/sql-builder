use std::marker::PhantomData;

use crate::{BelongsTo, NoTable, TableSchema, WithTable};
use crate::execute::{InsertData, Runner};
use crate::r#where::{IntoValue, Value};

struct InsertBuilderData {
    table: Option<&'static str>,
    columns: Vec<&'static str>,
    values: Vec<Value>,
}

pub struct WithValues<T>(PhantomData<T>);

pub struct InsertBuilder<Phase> {
    data: InsertBuilderData,
    _phase: PhantomData<Phase>,
}

impl Default for InsertBuilder<NoTable> {
    fn default() -> Self {
        Self::new()
    }
}

impl InsertBuilder<NoTable> {
    pub fn new() -> Self {
        Self {
            data: InsertBuilderData { table: None, columns: Vec::new(), values: Vec::new() },
            _phase: PhantomData,
        }
    }

    pub fn into_table<T: TableSchema>(self) -> InsertBuilder<WithTable<T>> {
        InsertBuilder {
            data: InsertBuilderData { table: Some(T::TABLE_NAME), ..self.data },
            _phase: PhantomData,
        }
    }
}

impl<T: TableSchema> InsertBuilder<WithTable<T>> {
    pub fn value<C, V>(mut self, val: V) -> InsertBuilder<WithValues<T>>
    where
        C: BelongsTo<T>,
        V: IntoValue,
    {
        self.data.columns.push(C::COLUMN_NAME);
        self.data.values.push(val.into_value());
        InsertBuilder { data: self.data, _phase: PhantomData }
    }
}

impl<T: TableSchema> InsertBuilder<WithValues<T>> {
    pub fn value<C, V>(mut self, val: V) -> Self
    where
        C: BelongsTo<T>,
        V: IntoValue,
    {
        self.data.columns.push(C::COLUMN_NAME);
        self.data.values.push(val.into_value());
        self
    }

    pub(crate) fn into_data(self) -> InsertData {
        InsertData {
            table: self.data.table.unwrap(),
            columns: self.data.columns,
            values: self.data.values,
        }
    }

    pub fn execute(self, runner: &impl Runner<()>) -> Result<(), Box<dyn std::error::Error>> {
        runner.insert(self.into_data())
    }
}
