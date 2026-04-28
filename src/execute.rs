use std::{error::Error, future::Future, marker::PhantomData, pin::Pin};

use crate::{QueryBuilder, TableSchema, WithColumns};

// ── Marker types ──────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct Executable;

#[derive(Clone, Default)]
pub struct ExecutableAsync;

#[derive(Clone, Default)]
pub struct NotExecutable;

/// Marker: query returns exactly one deserialized row.
#[derive(Clone, Default)]
pub struct ExecutableOne;

/// Marker: query returns zero-or-more deserialized rows.
#[derive(Clone, Default)]
pub struct ExecutableAll;

// ── Result / function type aliases ────────────────────────────────────────────

pub type ExecuteResult = Result<(), Box<dyn Error>>;
pub type ExecuteFn = Box<dyn Fn(String) -> ExecuteResult + Send + Sync>;
pub type AsyncFn =
    Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = ExecuteResult> + Send>> + Send + Sync>;

pub type ExecuteOneResult<T> = Result<T, Box<dyn Error>>;
pub type ExecuteAllResult<T> = Result<Vec<T>, Box<dyn Error>>;

/// Sync runner that deserializes a single row from the pre-built SQL string.
pub type ExecuteOneFn<T> = Box<dyn Fn(String) -> ExecuteOneResult<T> + Send + Sync>;

/// Sync runner that deserializes all rows from the pre-built SQL string.
pub type ExecuteAllFn<T> = Box<dyn Fn(String) -> ExecuteAllResult<T> + Send + Sync>;

// ── Fire-and-forget runners ───────────────────────────────────────────────────

impl<Phase, S> QueryBuilder<Phase, S, NotExecutable> {
    pub fn runner(self, execute_fn: ExecuteFn) -> QueryBuilder<Phase, S, Executable> {
        QueryBuilder {
            data: self.data,
            execute_fn: Some(execute_fn),
            execute_async_fn: None,
            execute_one_fn: None,
            execute_all_fn: None,
            _phase: PhantomData,
            _execute: PhantomData,
            _seal: PhantomData,
        }
    }

    pub fn runner_async(self, execute_fn: AsyncFn) -> QueryBuilder<Phase, S, ExecutableAsync> {
        QueryBuilder {
            data: self.data,
            execute_fn: None,
            execute_async_fn: Some(execute_fn),
            execute_one_fn: None,
            execute_all_fn: None,
            _phase: PhantomData,
            _execute: PhantomData,
            _seal: PhantomData,
        }
    }
}

impl<T: TableSchema, Cols, S> QueryBuilder<WithColumns<T, Cols>, S, Executable> {
    pub fn execute(self) -> ExecuteResult {
        let execute_fn = self.execute_fn.expect("due to type has to be there");
        execute_fn(self.data.build_sql())
    }
}

impl<T: TableSchema, Cols, S> QueryBuilder<WithColumns<T, Cols>, S, ExecutableAsync> {
    pub async fn execute(self) -> ExecuteResult {
        let execute_fn = self.execute_async_fn.expect("due to type has to be there");
        execute_fn(self.data.build_sql()).await
    }
}

// ── Typed runners (ExecuteOne / ExecuteAll) ───────────────────────────────────

impl<T: TableSchema + 'static, Cols, S> QueryBuilder<WithColumns<T, Cols>, S, NotExecutable> {
    /// Attach a runner that executes the SQL and deserializes a single row.
    pub fn runner_one(
        self,
        f: impl Fn(String) -> ExecuteOneResult<T> + Send + Sync + 'static,
    ) -> QueryBuilder<WithColumns<T, Cols>, S, ExecutableOne, T> {
        QueryBuilder {
            data: self.data,
            execute_fn: None,
            execute_async_fn: None,
            execute_one_fn: Some(Box::new(f)),
            execute_all_fn: None,
            _phase: PhantomData,
            _execute: PhantomData,
            _seal: PhantomData,
        }
    }

    /// Attach a runner that executes the SQL and deserializes all result rows.
    pub fn runner_all(
        self,
        f: impl Fn(String) -> ExecuteAllResult<T> + Send + Sync + 'static,
    ) -> QueryBuilder<WithColumns<T, Cols>, S, ExecutableAll, T> {
        QueryBuilder {
            data: self.data,
            execute_fn: None,
            execute_async_fn: None,
            execute_one_fn: None,
            execute_all_fn: Some(Box::new(f)),
            _phase: PhantomData,
            _execute: PhantomData,
            _seal: PhantomData,
        }
    }
}

impl<T: TableSchema, Cols, S> QueryBuilder<WithColumns<T, Cols>, S, ExecutableOne, T> {
    pub fn execute_one(self) -> ExecuteOneResult<T> {
        let f = self.execute_one_fn.expect("due to type has to be there");
        f(self.data.build_sql())
    }
}

impl<T: TableSchema, Cols, S> QueryBuilder<WithColumns<T, Cols>, S, ExecutableAll, T> {
    pub fn execute_all(self) -> ExecuteAllResult<T> {
        let f = self.execute_all_fn.expect("due to type has to be there");
        f(self.data.build_sql())
    }
}
