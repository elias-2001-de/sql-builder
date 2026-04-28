use async_trait::async_trait;
use std::error::Error;

#[derive(Clone)]
pub(crate) struct NotExecutable;

enum QueryData {}

pub trait Runner<T> {
    fn execute(data: QueryData) -> Result<(), Box<dyn Error>>;
    fn execute_all(data: QueryData) -> Result<Vec<T>, Box<dyn Error>>;
    fn execute_one(data: QueryData) -> Result<T, Box<dyn Error>>;
    fn execute_maybe_one(data: QueryData) -> Result<Option<T>, Box<dyn Error>>;
}

#[async_trait]
pub trait RunnerAsync<T> {
    async fn execute(data: QueryData) -> Result<(), Box<dyn Error>>;
    async fn execute_all(data: QueryData) -> Result<Vec<T>, Box<dyn Error>>;
    async fn execute_one(data: QueryData) -> Result<T, Box<dyn Error>>;
    async fn execute_maybe_one(data: QueryData) -> Result<Option<T>, Box<dyn Error>>;
}
