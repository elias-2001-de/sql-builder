// ── Schema macro ──────────────────────────────────────────────────────────────
//
//  Syntax: `table! { mod_name: TableType => "sql_name" { columns... } }`
//
//  Column syntax:
//    col*:      Type   →  PRIMARY KEY, NOT NULL
//    col:       Type   →  NOT NULL
//    col?:      Type   →  NULLABLE
//    col->      Table: Type  →  FOREIGN KEY → Table's PK, NOT NULL
//    col?->     Table: Type  →  FOREIGN KEY → Table's PK, NULLABLE
//
//  Each table's column structs live in `pub mod mod_name`, which prevents
//  name collisions when the same logical column name (e.g. `AuthorId`) appears
//  in multiple tables.
//
//  Implemented as a TT muncher to avoid the ambiguity that arises when
//  `$($rest:tt)*` is nested inside `$(...),*` (rejected in edition 2024).

#[macro_export]
macro_rules! table {
    ($mod_name:ident: $table:ident => $table_name:literal { $($body:tt)* }) => {
        pub struct $table;
        impl $crate::TableSchema for $table {
            const TABLE_NAME: &'static str = $table_name;
        }
        pub mod $mod_name {
            #[allow(unused_imports)]
            use super::*;
            $crate::table!(@col $table $($body)*);
        }
    };

    (@col $table:ident) => {};

    // FK NULLABLE: col?-> RefTable: Type
    (@col $table:ident $col:ident ?-> $ref_table:ident : $vtype:ty $(, $($rest:tt)*)?) => {
        pub struct $col;
        impl $crate::BelongsTo<$table> for $col {
            type Value = $vtype;
            type Null  = $crate::Nullable;
            const COLUMN_NAME: &'static str = stringify!($col);
        }
        impl $crate::ForeignKey<$table> for $col {
            type References = $ref_table;
            type RefColumn  = <$ref_table as $crate::HasPrimaryKey>::PkColumn;
        }
        $($crate::table!(@col $table $($rest)*);)?
    };

    // NULLABLE: col?: Type
    (@col $table:ident $col:ident ?: $vtype:ty $(, $($rest:tt)*)?) => {
        pub struct $col;
        impl $crate::BelongsTo<$table> for $col {
            type Value = $vtype;
            type Null  = $crate::Nullable;
            const COLUMN_NAME: &'static str = stringify!($col);
        }
        $($crate::table!(@col $table $($rest)*);)?
    };

    // FK NOT NULL: col-> RefTable: Type
    (@col $table:ident $col:ident -> $ref_table:ident : $vtype:ty $(, $($rest:tt)*)?) => {
        pub struct $col;
        impl $crate::BelongsTo<$table> for $col {
            type Value = $vtype;
            type Null  = $crate::NotNull;
            const COLUMN_NAME: &'static str = stringify!($col);
        }
        impl $crate::ForeignKey<$table> for $col {
            type References = $ref_table;
            type RefColumn  = <$ref_table as $crate::HasPrimaryKey>::PkColumn;
        }
        $($crate::table!(@col $table $($rest)*);)?
    };

    // PRIMARY KEY: col*: Type
    (@col $table:ident $col:ident *: $vtype:ty $(, $($rest:tt)*)?) => {
        pub struct $col;
        impl $crate::BelongsTo<$table> for $col {
            type Value = $vtype;
            type Null  = $crate::NotNull;
            const COLUMN_NAME: &'static str = stringify!($col);
        }
        impl $crate::PrimaryKey<$table> for $col {}
        $($crate::table!(@col $table $($rest)*);)?
    };

    // NOT NULL: col: Type
    (@col $table:ident $col:ident : $vtype:ty $(, $($rest:tt)*)?) => {
        pub struct $col;
        impl $crate::BelongsTo<$table> for $col {
            type Value = $vtype;
            type Null  = $crate::NotNull;
            const COLUMN_NAME: &'static str = stringify!($col);
        }
        $($crate::table!(@col $table $($rest)*);)?
    };
}

#[macro_export]
macro_rules! impl_has_pk {
    ($table:ident, $pk_col:ty) => {
        impl $crate::HasPrimaryKey for $table {
            type PkColumn = $pk_col;
        }
    };
}
