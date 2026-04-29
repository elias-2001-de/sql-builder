use std::cell::RefCell;
use std::error::Error;

use crate::{ColumnExpr, Direction};
use crate::execute::{DeleteData, InsertData, QueryData, Runner, UpdateData};
use crate::query::QueryInternData;
use crate::r#where::ConditionAtom;

pub struct StringRunner {
    query: RefCell<Option<String>>,
}

impl Default for StringRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl StringRunner {
    pub fn new() -> Self {
        Self { query: RefCell::new(None) }
    }

    pub fn query(&self) -> Option<String> {
        self.query.borrow().clone()
    }

    fn set(&self, sql: String) {
        *self.query.borrow_mut() = Some(sql);
    }
}

// ── SQL builders ──────────────────────────────────────────────────────────────

fn col_expr_to_sql(expr: &ColumnExpr) -> String {
    match expr {
        ColumnExpr::All => "*".to_string(),
        ColumnExpr::Column { table, name } => format!("{}.{}", table, name),
        ColumnExpr::Count => "COUNT(*)".to_string(),
        ColumnExpr::Max { table, name } => format!("MAX({}.{})", table, name),
        ColumnExpr::Min { table, name } => format!("MIN({}.{})", table, name),
        ColumnExpr::Sum { table, name } => format!("SUM({}.{})", table, name),
    }
}

fn atom_to_sql(atom: &ConditionAtom) -> String {
    match atom {
        ConditionAtom::InSubquery(col, sub) => format!("{} IN ({})", col, build_select(sub)),
        ConditionAtom::NotInSubquery(col, sub) => format!("{} NOT IN ({})", col, build_select(sub)),
        ConditionAtom::Exists(sub) => format!("EXISTS ({})", build_select(sub)),
        ConditionAtom::NotExists(sub) => format!("NOT EXISTS ({})", build_select(sub)),
        other => other.to_sql(),
    }
}

fn build_conditions(groups: &[Vec<ConditionAtom>]) -> String {
    groups
        .iter()
        .map(|group| {
            let fragment = group.iter().map(atom_to_sql).collect::<Vec<_>>().join(" ");
            format!("({})", fragment)
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

pub(crate) fn build_select(data: &QueryInternData) -> String {
    let cols = if data.columns.is_empty() {
        "*".to_string()
    } else {
        data.columns.iter().map(col_expr_to_sql).collect::<Vec<_>>().join(", ")
    };

    let from = match &data.subquery_source {
        Some(sub) => format!("({}) AS {}", build_select(sub), data.table.unwrap_or("sub")),
        None => data.table.unwrap_or("unknown").to_string(),
    };

    let mut sql = format!("SELECT {} FROM {}", cols, from);

    for join in &data.joins {
        sql.push(' ');
        sql.push_str(&join.clone().to_sql());
    }

    if !data.conditions.is_empty() {
        sql.push_str(&format!(" WHERE {}", build_conditions(&data.conditions)));
    }

    if !data.group_by.is_empty() {
        sql.push_str(&format!(" GROUP BY {}", data.group_by.join(", ")));
    }

    if !data.having.is_empty() {
        sql.push_str(&format!(" HAVING {}", build_conditions(&data.having)));
    }

    if let Some((col, dir)) = &data.order_by {
        let dir_sql = match dir {
            Direction::Asc => "ASC",
            Direction::Desc => "DESC",
        };
        sql.push_str(&format!(" ORDER BY {} {}", col, dir_sql));
    }

    if let Some(n) = data.limit {
        sql.push_str(&format!(" LIMIT {}", n));
    }
    if let Some(n) = data.offset {
        sql.push_str(&format!(" OFFSET {}", n));
    }

    sql
}

pub(crate) fn build_insert(data: &InsertData) -> String {
    let cols = data.columns.join(", ");
    let vals = data.values.iter().map(|v| v.to_sql()).collect::<Vec<_>>().join(", ");
    format!("INSERT INTO {} ({}) VALUES ({})", data.table, cols, vals)
}

pub(crate) fn build_update(data: &UpdateData) -> String {
    let set_clause = data
        .assignments
        .iter()
        .map(|(col, val)| match val {
            Some(v) => format!("{} = {}", col, v.to_sql()),
            None => format!("{} = NULL", col),
        })
        .collect::<Vec<_>>()
        .join(", ");
    let mut sql = format!("UPDATE {} SET {}", data.table, set_clause);
    if !data.conditions.is_empty() {
        sql.push_str(&format!(" WHERE {}", build_conditions(&data.conditions)));
    }
    sql
}

pub(crate) fn build_delete(data: &DeleteData) -> String {
    let mut sql = format!("DELETE FROM {}", data.table);
    if !data.conditions.is_empty() {
        sql.push_str(&format!(" WHERE {}", build_conditions(&data.conditions)));
    }
    sql
}

// ── Runner impl ───────────────────────────────────────────────────────────────

impl Runner<()> for StringRunner {
    fn execute(&self, data: QueryData) -> Result<(), Box<dyn Error>> {
        self.set(build_select(&data.0));
        Ok(())
    }

    fn execute_all(&self, data: QueryData) -> Result<Vec<()>, Box<dyn Error>> {
        self.set(build_select(&data.0));
        Ok(vec![])
    }

    fn execute_one(&self, data: QueryData) -> Result<(), Box<dyn Error>> {
        self.set(build_select(&data.0));
        Ok(())
    }

    fn execute_maybe_one(&self, data: QueryData) -> Result<Option<()>, Box<dyn Error>> {
        self.set(build_select(&data.0));
        Ok(None)
    }

    fn insert(&self, data: InsertData) -> Result<(), Box<dyn Error>> {
        self.set(build_insert(&data));
        Ok(())
    }

    fn update(&self, data: UpdateData) -> Result<(), Box<dyn Error>> {
        self.set(build_update(&data));
        Ok(())
    }

    fn delete(&self, data: DeleteData) -> Result<(), Box<dyn Error>> {
        self.set(build_delete(&data));
        Ok(())
    }
}
