use std::marker::PhantomData;

use std::error::Error;

use crate::{
    BelongsTo, ColumnExpr, Direction, NoTable, NotSealed, SubquerySql, TableSchema, WithColumns,
    WithTable,
    execute::{NotExecutable, QueryData, Runner, RunnerAsync},
    join::JoinClause,
    r#where::{ConditionAtom, HasCondition, WhereClause},
};

// ── Query internal data ───────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub(crate) struct QueryInternData {
    pub(crate) table: Option<&'static str>,
    pub(crate) subquery_source: Option<Box<QueryInternData>>,
    pub(crate) columns: Vec<ColumnExpr>,
    pub(crate) joins: Vec<JoinClause>,
    pub(crate) conditions: Vec<Vec<ConditionAtom>>,
    pub(crate) group_by: Vec<&'static str>,
    pub(crate) having: Vec<Vec<ConditionAtom>>,
    pub(crate) order_by: Option<(&'static str, Direction)>,
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}

// ── Query builder ─────────────────────────────────────────────────────────────

// Row is the deserialized row type: () for fire-and-forget runners, T for typed
// RunOne/RunAll runners.
pub struct QueryBuilder<Phase, S, R, Row = ()> {
    pub(crate) data: QueryInternData,
    pub(crate) runner: Option<Box<dyn Runner<Row>>>,
    pub(crate) runner_async: Option<Box<dyn RunnerAsync<Row> + Send + Sync>>,

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

impl<T: TableSchema, Cols, R, Row> QueryBuilder<WithColumns<T, Cols>, NotSealed, R, Row> {
    pub fn group_by<C: BelongsTo<T>>(mut self) -> Self {
        self.data.group_by.push(C::COLUMN_NAME);
        self
    }

    pub fn having(mut self, clause: WhereClause<T, HasCondition>) -> Self {
        self.data.having.push(clause.fragments);
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

impl<T: TableSchema, Cols, R, Row> QueryBuilder<WithColumns<T, Cols>, NotSealed, R, Row> {
    pub fn execute(self, runner: &impl Runner<Row>) -> Result<(), Box<dyn Error>> {
        runner.execute(QueryData(self.data))
    }

    pub fn execute_all(self, runner: &impl Runner<Row>) -> Result<Vec<Row>, Box<dyn Error>> {
        runner.execute_all(QueryData(self.data))
    }

    pub fn execute_one(self, runner: &impl Runner<Row>) -> Result<Row, Box<dyn Error>> {
        runner.execute_one(QueryData(self.data))
    }

    pub fn execute_maybe_one(
        self,
        runner: &impl Runner<Row>,
    ) -> Result<Option<Row>, Box<dyn Error>> {
        runner.execute_maybe_one(QueryData(self.data))
    }
}

#[allow(private_interfaces)]
impl<T: TableSchema, Cols, S, R, Row> SubquerySql for QueryBuilder<WithColumns<T, Cols>, S, R, Row> {
    fn into_subquery_data(self) -> QueryInternData {
        self.data
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
