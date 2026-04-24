use std::marker::PhantomData;

use crate::{NoTable, TableSchema, WithTable};
use crate::r#where::{HasCondition, WhereClause};

struct DeleteData {
    table: Option<&'static str>,
    conditions: Vec<String>,
}

pub struct DeleteBuilder<Phase> {
    data: DeleteData,
    _phase: PhantomData<Phase>,
}

impl Default for DeleteBuilder<NoTable> {
    fn default() -> Self {
        Self::new()
    }
}

impl DeleteBuilder<NoTable> {
    pub fn new() -> Self {
        Self {
            data: DeleteData { table: None, conditions: Vec::new() },
            _phase: PhantomData,
        }
    }

    pub fn from<T: TableSchema>(self) -> DeleteBuilder<WithTable<T>> {
        DeleteBuilder {
            data: DeleteData { table: Some(T::TABLE_NAME), ..self.data },
            _phase: PhantomData,
        }
    }
}

impl<T: TableSchema> DeleteBuilder<WithTable<T>> {
    pub fn where_clause(mut self, clause: WhereClause<T, HasCondition>) -> Self {
        self.data.conditions.push(format!("({})", clause.build_fragment()));
        self
    }

    pub fn build(self) -> String {
        let table = self.data.table.unwrap();
        let mut sql = format!("DELETE FROM {table}");
        if !self.data.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.data.conditions.join(" AND "));
        }
        sql
    }
}
