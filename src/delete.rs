use std::marker::PhantomData;

use crate::{NoTable, TableSchema, WithTable};
use crate::execute::{DeleteData, Runner};
use crate::r#where::{ConditionAtom, HasCondition, WhereClause};

struct DeleteBuilderData {
    table: Option<&'static str>,
    conditions: Vec<Vec<ConditionAtom>>,
}

pub struct DeleteBuilder<Phase> {
    data: DeleteBuilderData,
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
            data: DeleteBuilderData { table: None, conditions: Vec::new() },
            _phase: PhantomData,
        }
    }

    pub fn from<T: TableSchema>(self) -> DeleteBuilder<WithTable<T>> {
        DeleteBuilder {
            data: DeleteBuilderData { table: Some(T::TABLE_NAME), ..self.data },
            _phase: PhantomData,
        }
    }
}

impl<T: TableSchema> DeleteBuilder<WithTable<T>> {
    pub fn where_clause(mut self, clause: WhereClause<T, HasCondition>) -> Self {
        self.data.conditions.push(clause.fragments);
        self
    }

    pub(crate) fn into_data(self) -> DeleteData {
        DeleteData {
            table: self.data.table.unwrap(),
            conditions: self.data.conditions,
        }
    }

    pub fn execute(self, runner: &impl Runner<()>) -> Result<(), Box<dyn std::error::Error>> {
        runner.delete(self.into_data())
    }
}
