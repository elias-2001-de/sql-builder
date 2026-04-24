

// ──  Run ─────────────────────────────────────────────────────────────────────

use std::{error::Error, future::Future, marker::PhantomData, pin::Pin};

use crate::{QueryBuilder, QueryInternData, TableSchema, WithColumns};

#[derive(Clone, Default)]
pub struct Runable;

#[derive(Clone, Default)]
pub struct RunableAsync;

#[derive(Clone, Default)]
pub struct NotRunable;

pub type RunResult = Result<(), Box<dyn Error>>;
pub type AsyncFn =
    Box<dyn Fn(QueryInternData) -> Pin<Box<dyn Future<Output = RunResult> + Send>> + Send + Sync>;
pub type RunFn = Box<dyn Fn(QueryInternData) -> RunResult>;


impl<Phase, S> QueryBuilder<Phase, S, NotRunable> {
    pub fn runner(self, run_fn: RunFn) -> QueryBuilder<Phase, S, Runable> {
        // assert!(self.run_async_fn.is_none());
        // assert!(self.run_fn.is_none());
        QueryBuilder {
            data: self.data,
            run_fn: Some(run_fn),
            run_async_fn: None,
            _phase: PhantomData,
            _run: PhantomData,
            _seal: PhantomData,
        }
    }
}

impl<Phase, S> QueryBuilder<Phase, S, NotRunable> {
    pub fn runner_async(self, run_fn: AsyncFn) -> QueryBuilder<Phase, S, RunableAsync> {
        // assert!(self.run_async_fn.is_none());
        // assert!(self.run_fn.is_none());
        QueryBuilder {
            data: self.data,
            run_fn: None,
            run_async_fn: Some(run_fn),
            _phase: PhantomData,
            _run: PhantomData,
            _seal: PhantomData,
        }
    }
}


impl<T: TableSchema, S> QueryBuilder<WithColumns<T>, S, Runable> {
    pub fn run(self) -> RunResult {
        let run_fn = self.run_fn.expect("due to type has to be ther");
        run_fn(self.data)
    }
}

impl<T: TableSchema, S> QueryBuilder<WithColumns<T>, S, RunableAsync> {
    pub async fn run(self) -> RunResult {
        let run_fn = self.run_async_fn.expect("due to type has to be there");
        run_fn(self.data).await
    }
}
