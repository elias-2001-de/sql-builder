use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, ExprLit, Fields, GenericArgument, Ident, Lit, Meta,
    PathArguments, Type, parse_macro_input, spanned::Spanned,
};

/// Derives `TableSchema` + column types for a struct, equivalent to the `table!` macro.
///
/// ## Struct attribute
/// - `#[table_name = "sql_name"]` — custom SQL table name (defaults to lowercase struct name)
///
/// ## Field attributes
/// - `#[primary_key]`              — at most one per struct; generates `PrimaryKey` impl
/// - `#[foreign_key(RefTable)]`   — at most one per struct; generates `ForeignKey` impl
/// - `#[unique]`                   — generates `UniqueColumn` impl
/// - `#[column_name = "sql_col"]` — custom SQL column name (defaults to field name)
///
/// `Option<T>` fields become `Nullable`, bare `T` fields become `NotNull`.
#[proc_macro_derive(Table, attributes(table_name, primary_key, foreign_key, unique, column_name))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_impl(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    let struct_ident = &input.ident;
    let struct_name_str = struct_ident.to_string();

    let table_name =
        get_table_name(&input).unwrap_or_else(|| struct_name_str.to_lowercase());

    let named_fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => {
                return Err(syn::Error::new(
                    struct_ident.span(),
                    "#[derive(Table)] requires named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new(
                struct_ident.span(),
                "#[derive(Table)] only works on structs",
            ))
        }
    };

    let mod_ident = Ident::new(&struct_name_str.to_lowercase(), struct_ident.span());

    let mut pk_col_ident: Option<Ident> = None;
    let mut col_tokens: Vec<TokenStream2> = Vec::new();
    let mut col_def_tokens: Vec<TokenStream2> = Vec::new();

    for field in named_fields {
        let field_ident = field.ident.as_ref().unwrap();
        let col_struct_ident =
            Ident::new(&to_pascal_case(&field_ident.to_string()), field_ident.span());

        let is_pk = has_attr(&field.attrs, "primary_key");
        let fk_ref = get_fk_ref(&field.attrs)?;
        let is_unique = has_attr(&field.attrs, "unique");
        let col_name =
            get_col_name(&field.attrs)?.unwrap_or_else(|| field_ident.to_string());

        let (inner_ty, nullable) = unwrap_option(&field.ty);

        if is_pk {
            if pk_col_ident.is_some() {
                return Err(syn::Error::new(
                    field_ident.span(),
                    "only one #[primary_key] is allowed per table",
                ));
            }
            if nullable {
                return Err(syn::Error::new(
                    field_ident.span(),
                    "#[primary_key] field cannot be Option<T>",
                ));
            }
            pk_col_ident = Some(col_struct_ident.clone());
        }


        let null_ty = if nullable {
            quote! { ::sql_builder::Nullable }
        } else {
            quote! { ::sql_builder::NotNull }
        };

        let mut tokens = quote! {
            pub struct #col_struct_ident;
            impl ::sql_builder::BelongsTo<#struct_ident> for #col_struct_ident {
                type Value = #inner_ty;
                type Null  = #null_ty;
                const COLUMN_NAME: &'static str = #col_name;
            }
            impl ::sql_builder::SelectExpr<#struct_ident> for #col_struct_ident {
                fn column_expr() -> ::sql_builder::ColumnExpr {
                    ::sql_builder::ColumnExpr::Column { table: #table_name, name: #col_name }
                }
            }
        };

        if is_pk {
            tokens.extend(quote! {
                impl ::sql_builder::PrimaryKey<#struct_ident> for #col_struct_ident {}
            });
        }

        if let Some(ref_table) = fk_ref {
            tokens.extend(quote! {
                impl ::sql_builder::ForeignKey<#struct_ident> for #col_struct_ident {
                    type References = #ref_table;
                    type RefColumn  = <#ref_table as ::sql_builder::HasPrimaryKey>::PkColumn;
                }
            });
        }

        if is_unique {
            tokens.extend(quote! {
                impl ::sql_builder::UniqueColumn<#struct_ident> for #col_struct_ident {}
            });
        }

        col_tokens.push(tokens);

        let is_nullable = nullable;
        let is_pk = is_pk;
        col_def_tokens.push(quote! {
            ::sql_builder::ColumnDef {
                name: #col_name,
                sql_type: <#inner_ty as ::sql_builder::ToSqlType>::SQL_TYPE,
                nullable: #is_nullable,
                primary_key: #is_pk,
            }
        });
    }

    let has_pk_impl = if let Some(pk_col) = pk_col_ident {
        quote! {
            impl ::sql_builder::HasPrimaryKey for #struct_ident {
                type PkColumn = #mod_ident::#pk_col;
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        impl ::sql_builder::TableSchema for #struct_ident {
            const TABLE_NAME: &'static str = #table_name;
        }

        impl ::sql_builder::TableInit for #struct_ident {
            fn column_defs() -> ::std::vec::Vec<::sql_builder::ColumnDef> {
                vec![ #(#col_def_tokens),* ]
            }
        }

        pub mod #mod_ident {
            #[allow(unused_imports)]
            use super::*;

            #(#col_tokens)*
        }

        #has_pk_impl
    })
}

// ── Attribute helpers ─────────────────────────────────────────────────────────

fn get_table_name(input: &DeriveInput) -> Option<String> {
    for attr in &input.attrs {
        if attr.path().is_ident("table_name") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value {
                    return Some(s.value());
                }
            }
        }
    }
    None
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(name))
}

fn get_fk_ref(attrs: &[syn::Attribute]) -> syn::Result<Option<Ident>> {
    for attr in attrs {
        if attr.path().is_ident("foreign_key") {
            let ident: Ident = attr.parse_args()?;
            return Ok(Some(ident));
        }
    }
    Ok(None)
}

fn get_col_name(attrs: &[syn::Attribute]) -> syn::Result<Option<String>> {
    for attr in attrs {
        if attr.path().is_ident("column_name") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value {
                    return Ok(Some(s.value()));
                }
            }
            return Err(syn::Error::new(
                attr.span(),
                r#"expected #[column_name = "sql_name"]"#,
            ));
        }
    }
    Ok(None)
}

// ── Type helpers ──────────────────────────────────────────────────────────────

/// Returns `(inner_type, true)` if `ty` is `Option<inner_type>`, else `(ty, false)`.
fn unwrap_option(ty: &Type) -> (&Type, bool) {
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            if seg.ident == "Option" {
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    if let Some(GenericArgument::Type(inner)) = ab.args.first() {
                        return (inner, true);
                    }
                }
            }
        }
    }
    (ty, false)
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
