mod common;
use common::*;
use sql_builder::*;

#[test]
fn order_by_asc() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .order_by::<posts::PostId>(Direction::Asc)
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT * FROM posts ORDER BY PostId ASC");
}

#[test]
fn order_by_desc_with_limit_and_offset() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Posts>()
        .select_all()
        .order_by::<posts::PostId>(Direction::Desc)
        .limit(10)
        .offset(20)
        .execute_all(&runner)
        .unwrap();
    assert_eq!(
        runner.query().unwrap(),
        "SELECT * FROM posts ORDER BY PostId DESC LIMIT 10 OFFSET 20"
    );
}

#[test]
fn limit_without_offset() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .limit(3)
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT * FROM users LIMIT 3");
}

#[test]
fn offset_without_limit() {
    let runner = StringRunner::new();
    QueryBuilder::new()
        .from::<Users>()
        .select_all()
        .offset(10)
        .execute_all(&runner)
        .unwrap();
    assert_eq!(runner.query().unwrap(), "SELECT * FROM users OFFSET 10");
}
