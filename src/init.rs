use crate::TableSchema;

// ── SQL type kinds ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SqlTypeKind {
    Integer,
    BigInt,
    Real,
    Text,
    Boolean,
    Blob,
}

/// Map a Rust type to a generic SQL type kind.
/// Implement this for custom types to make them usable in `CREATE TABLE`.
pub trait ToSqlType {
    const SQL_TYPE: SqlTypeKind;
}

impl ToSqlType for i8    { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Integer; }
impl ToSqlType for i16   { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Integer; }
impl ToSqlType for i32   { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Integer; }
impl ToSqlType for u8    { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Integer; }
impl ToSqlType for u16   { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Integer; }
impl ToSqlType for u32   { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Integer; }
impl ToSqlType for i64   { const SQL_TYPE: SqlTypeKind = SqlTypeKind::BigInt;  }
impl ToSqlType for u64   { const SQL_TYPE: SqlTypeKind = SqlTypeKind::BigInt;  }
impl ToSqlType for f32   { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Real;    }
impl ToSqlType for f64   { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Real;    }
impl ToSqlType for bool  { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Boolean; }
impl ToSqlType for String { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Text;   }
impl ToSqlType for str    { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Text;   }
impl ToSqlType for Vec<u8> { const SQL_TYPE: SqlTypeKind = SqlTypeKind::Blob;  }

// ── Column metadata ───────────────────────────────────────────────────────────

pub struct ColumnDef {
    pub name: &'static str,
    pub sql_type: SqlTypeKind,
    pub nullable: bool,
    pub primary_key: bool,
}

// ── TableInit trait ───────────────────────────────────────────────────────────

/// Implemented automatically by `#[derive(Table)]`.
/// Provides the column metadata needed to generate a `CREATE TABLE` statement.
pub trait TableInit: TableSchema {
    fn column_defs() -> Vec<ColumnDef>;
}

// ── DbAdapter trait ───────────────────────────────────────────────────────────

/// Implement this trait to build a database adapter for a specific database.
///
/// You only need to provide `sql_type_name` (dialect-specific type mapping) and
/// `execute` (how to run a raw SQL string). Everything else is derived for free.
///
/// # Example
/// ```rust,ignore
/// struct SqliteAdapter { conn: rusqlite::Connection }
///
/// impl DbAdapter for SqliteAdapter {
///     fn sql_type_name(&self, ty: SqlTypeKind) -> &'static str {
///         match ty {
///             SqlTypeKind::Integer | SqlTypeKind::Boolean | SqlTypeKind::BigInt => "INTEGER",
///             SqlTypeKind::Real    => "REAL",
///             SqlTypeKind::Text    => "TEXT",
///             SqlTypeKind::Blob    => "BLOB",
///         }
///     }
///     fn execute(&self, sql: &str) { self.conn.execute_batch(sql).unwrap(); }
/// }
///
/// adapter.init_table::<Users>();
/// ```
pub trait DbAdapter {
    /// Map a generic SQL type kind to the type name for this specific database.
    fn sql_type_name(&self, ty: SqlTypeKind) -> &'static str;

    /// Execute a raw SQL string against this database.
    fn execute(&self, sql: &str);

    /// Build a `CREATE TABLE IF NOT EXISTS` statement for `T` using this adapter's type names.
    fn create_table_sql<T: TableInit>(&self) -> String {
        let cols = T::column_defs();
        let col_defs: Vec<String> = cols
            .iter()
            .map(|c| {
                let type_name = self.sql_type_name(c.sql_type);
                let mut def = format!("{} {}", c.name, type_name);
                if c.primary_key {
                    def.push_str(" PRIMARY KEY");
                } else if !c.nullable {
                    def.push_str(" NOT NULL");
                }
                def
            })
            .collect();
        format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            T::TABLE_NAME,
            col_defs.join(", ")
        )
    }

    /// Generate and execute the `CREATE TABLE` statement for `T`.
    fn init_table<T: TableInit>(&self) {
        let sql = self.create_table_sql::<T>();
        self.execute(&sql);
    }
}
