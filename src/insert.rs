use std::marker::PhantomData;

use crate::{BelongsTo, NoTable, TableSchema, WithTable};
use crate::r#where::IntoValue;

struct InsertData {
    table: Option<&'static str>,
    columns: Vec<&'static str>,
    values: Vec<String>,
}

pub struct WithValues<T>(PhantomData<T>);

pub struct InsertBuilder<Phase> {
    data: InsertData,
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
            data: InsertData { table: None, columns: Vec::new(), values: Vec::new() },
            _phase: PhantomData,
        }
    }

    pub fn into_table<T: TableSchema>(self) -> InsertBuilder<WithTable<T>> {
        InsertBuilder {
            data: InsertData { table: Some(T::TABLE_NAME), ..self.data },
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
        self.data.values.push(val.into_value().to_sql());
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
        self.data.values.push(val.into_value().to_sql());
        self
    }

    pub fn build(self) -> String {
        let table = self.data.table.unwrap();
        let cols = self.data.columns.join(", ");
        let vals = self.data.values.join(", ");
        format!("INSERT INTO {table} ({cols}) VALUES ({vals})")
    }
}
