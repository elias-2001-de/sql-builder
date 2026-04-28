use std::marker::PhantomData;

use crate::{
    BelongsTo, ColumnExpr, Direction, NoTable, NotSealed, SubquerySql, TableSchema, WithColumns,
    WithTable,
    execute::NotExecutable,
    join::JoinClause,
    r#where::{HasCondition, WhereClause},
};

// ── Query internal data ───────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub(crate) struct QueryInternData {
    pub(crate) table: Option<&'static str>,
    pub(crate) subquery_source: Option<Box<QueryInternData>>,
    pub(crate) columns: Vec<ColumnExpr>,
    pub(crate) joins: Vec<JoinClause>,
    pub(crate) conditions: Vec<String>,
    pub(crate) group_by: Vec<&'static str>,
    pub(crate) having: Vec<String>,
    pub(crate) order_by: Option<(&'static str, Direction)>,
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}

impl QueryInternData {
    pub(crate) fn build_sql(self) -> String {
        assert!(!self.columns.is_empty());
        let cols = self.columns.iter().map(ColumnExpr::to_sql).collect::<Vec<_>>().join(", ");

        let table = self.table.unwrap();
        let mut sql = match self.subquery_source {
            Some(sub) => format!("SELECT {cols} FROM ({}) AS {table}", sub.build_sql()),
            None => format!("SELECT {cols} FROM {table}"),
        };

        for join in self.joins {
            sql.push(' ');
            sql.push_str(&join.to_sql());
        }

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }
        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by.join(", "));
        }
        if !self.having.is_empty() {
            sql.push_str(" HAVING ");
            sql.push_str(&self.having.join(" AND "));
        }
        if let Some((col, dir)) = self.order_by {
            sql.push_str(&format!(" ORDER BY {col} {}", dir.sql()));
        }
        if let Some(l) = self.limit {
            sql.push_str(&format!(" LIMIT {l}"));
        }
        if let Some(o) = self.offset {
            sql.push_str(&format!(" OFFSET {o}"));
        }
        sql
    }
}

// ── Query builder ─────────────────────────────────────────────────────────────

// Row is the deserialized row type: () for fire-and-forget runners, T for typed
// RunOne/RunAll runners.
pub struct QueryBuilder<Phase, S, R, Row = ()> {
    pub(crate) data: QueryInternData,
    pub(crate) runner: Option<()>,
    pub(crate) runner_async: Option<()>,

    pub(crate) _phase: PhantomData<Phase>,
    pub(crate) _seal: PhantomData<S>,
    pub(crate) _execute: PhantomData<R>,
    pub(crate) _row: PhantomData<Row>,
}

impl<Phase, S> QueryBuilder<Phase, S, NotExecutable> {
    pub fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            runner: None,
            runner_async: None,

            _execute: PhantomData,
            _phase: PhantomData,
            _seal: PhantomData,
            _row: PhantomData,
        }
    }
}

impl QueryBuilder<NoTable, NotSealed, NotExecutable> {
    pub fn new() -> Self {
        QueryBuilder {
            _execute: PhantomData,
            _phase: PhantomData,
            _seal: PhantomData,
            _row: PhantomData,
            data: QueryInternData::default(),
            runner: None,
            runner_async: None,
        }
    }
}

impl<R, Row> QueryBuilder<NoTable, NotSealed, R, Row> {
    pub fn from<T: TableSchema>(self) -> QueryBuilder<WithTable<T>, NotSealed, R, Row> {
        let mut q: QueryBuilder<WithTable<T>, NotSealed, R, Row> = self.cast();
        q.data.table = Some(T::TABLE_NAME);
        q
    }

    pub fn from_subquery<T: TableSchema>(
        self,
        sql: impl SubquerySql,
    ) -> QueryBuilder<WithTable<T>, NotSealed, R, Row> {
        let mut q: QueryBuilder<WithTable<T>, NotSealed, R, Row> = self.cast();
        q.data.table = Some(T::TABLE_NAME);
        q.data.subquery_source = Some(Box::new(sql.into_subquery_data()));
        q
    }
}

impl<T: TableSchema, R, Row> QueryBuilder<WithTable<T>, NotSealed, R, Row> {
    pub fn seal(self) -> Self {
        self
    }
}

impl<T: TableSchema, Cols, S, R, Row> QueryBuilder<WithColumns<T, Cols>, S, R, Row> {
    pub fn build(self) -> String {
        self.data.build_sql()
    }
}

impl<T: TableSchema, Cols, R, Row> QueryBuilder<WithColumns<T, Cols>, NotSealed, R, Row> {
    pub fn group_by<C: BelongsTo<T>>(mut self) -> Self {
        self.data.group_by.push(C::COLUMN_NAME);
        self
    }

    pub fn having(mut self, clause: WhereClause<T, HasCondition>) -> Self {
        self.data
            .having
            .push(format!("({})", clause.build_fragment()));
        self
    }

    pub fn order_by<C: BelongsTo<T>>(mut self, dir: Direction) -> Self {
        self.data.order_by = Some((C::COLUMN_NAME, dir));
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.data.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.data.offset = Some(n);
        self
    }
}

impl<A, S, R, Row> QueryBuilder<A, S, R, Row> {
    pub(crate) fn cast<B>(self) -> QueryBuilder<B, S, R, Row> {
        QueryBuilder {
            data: self.data,
            runner: self.runner,
            runner_async: self.runner_async,
            _phase: PhantomData,
            _execute: PhantomData,
            _seal: PhantomData,
            _row: PhantomData,
        }
    }
}
