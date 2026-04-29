use async_trait::async_trait;
use std::error::Error;

use crate::query::QueryInternData;
use crate::r#where::{ConditionAtom, Value};

#[derive(Clone)]
pub struct NotExecutable;

#[allow(dead_code)]
pub struct QueryData(pub(crate) QueryInternData);

pub struct InsertData {
    pub table: &'static str,
    pub columns: Vec<&'static str>,
    pub values: Vec<Value>,
}

pub struct UpdateData {
    pub table: &'static str,
    pub assignments: Vec<(&'static str, Option<Value>)>,
    #[allow(dead_code)]
    pub(crate) conditions: Vec<Vec<ConditionAtom>>,
}

pub struct DeleteData {
    pub table: &'static str,
    #[allow(dead_code)]
    pub(crate) conditions: Vec<Vec<ConditionAtom>>,
}

pub trait Runner<T> {
    fn execute(&self, data: QueryData) -> Result<(), Box<dyn Error>>;
    fn execute_all(&self, data: QueryData) -> Result<Vec<T>, Box<dyn Error>>;
    fn execute_one(&self, data: QueryData) -> Result<T, Box<dyn Error>>;
    fn execute_maybe_one(&self, data: QueryData) -> Result<Option<T>, Box<dyn Error>>;
    fn insert(&self, data: InsertData) -> Result<(), Box<dyn Error>>;
    fn update(&self, data: UpdateData) -> Result<(), Box<dyn Error>>;
    fn delete(&self, data: DeleteData) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
pub trait RunnerAsync<T: Send> {
    async fn execute(&self, data: QueryData) -> Result<(), Box<dyn Error>>;
    async fn execute_all(&self, data: QueryData) -> Result<Vec<T>, Box<dyn Error>>;
    async fn execute_one(&self, data: QueryData) -> Result<T, Box<dyn Error>>;
    async fn execute_maybe_one(&self, data: QueryData) -> Result<Option<T>, Box<dyn Error>>;
    async fn insert(&self, data: InsertData) -> Result<(), Box<dyn Error>>;
    async fn update(&self, data: UpdateData) -> Result<(), Box<dyn Error>>;
    async fn delete(&self, data: DeleteData) -> Result<(), Box<dyn Error>>;
}
