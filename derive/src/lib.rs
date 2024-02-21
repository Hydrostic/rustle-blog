


mod join;
mod err;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::Meta::Path;
use syn::{parse_macro_input, Ident, ItemStruct};


#[proc_macro_derive(ErrorHelper, attributes(err))]
pub fn error_helper(input: TokenStream) -> TokenStream {
    err::error_helper(input)
}
#[proc_macro_derive(JoinHelper, attributes(foreign_key, foreign_object))]
pub fn join_helper(input: TokenStream) -> TokenStream {
    join::join_helper(input)
}


#[proc_macro_derive(FilterParams, attributes(filterable))]
pub fn filter_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let ident_name = input.ident;
    let new_ident_name = format_ident!("{}Filterable", ident_name);
    let filterable_fields = input.fields.iter().filter(|field| {
        field.attrs.iter().any(|t| {
            if let Path(p) = &t.meta
            {
                if p.is_ident("filterable"){
                    true
                }else{
                    false
                }
                
            } else {
                false
            }
        })
    });
    let enum_variants = filterable_fields.clone().map(|field| {
        let t = field.ident.clone().unwrap();
        let camel_ident = Ident::new(&t.to_string().to_case(Case::Pascal), t.span());
        let t = t.to_string();
        let ty = &field.ty;
        quote! {
            #[serde(rename = #t)]
            #camel_ident(#ty)
        }
    });
    let pat_arms_value = filterable_fields.clone().map(|field| {
        let t = field.ident.as_ref().unwrap();
        let camel_ident = Ident::new(&t.to_string().to_case(Case::Pascal), field.span());
        quote! {
            #new_ident_name::#camel_ident(k) => instance.bind(k)
        }
    });
    let pat_arms_name = filterable_fields.map(|field| {
        let t = field.ident.as_ref().unwrap();

        let t_str = &t.to_string();
        let camel_ident = Ident::new(&t.to_string().to_case(Case::Pascal), field.span());
        quote! {
            #new_ident_name::#camel_ident(_) => #t_str
        }
    });
    (quote!{

        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        pub enum #new_ident_name{
            #(#enum_variants),*
        }
        impl #new_ident_name{
            pub fn get_field_name(&self) -> &'static str{
                match self{
                    #(#pat_arms_name),*
                }
            }
            pub fn bind_value<T>(self, instance: sqlx::query::QueryAs<'_, sqlx::mysql::MySql, T, sqlx::mysql::MySqlArguments>) -> sqlx::query::QueryAs<'_, sqlx::mysql::MySql, T, sqlx::mysql::MySqlArguments>{
                match self{
                    #(#pat_arms_value),*
                }
            }
        }
    }).into()
}

#[proc_macro_derive(SortParams, attributes(sortable))]
pub fn sort_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let ident_name = input.ident;
    let new_ident_name = format_ident!("{}Sortable", ident_name);
    let filterable_fields = input.fields.iter().filter(|field| {
        field.attrs.iter().any(|t| {
            if let Path(p) = &t.meta{
                if p.is_ident("sortable"){
                    true
                }else{
                    false
                }
            } else {
                false
            }
        })
    });
    let enum_variants = filterable_fields.clone().map(|field| {
        let t = field.ident.clone().unwrap();
        let camel_ident = Ident::new(&t.to_string().to_case(Case::Pascal), t.span());
        let t = t.to_string();
        quote! {
            #[serde(rename = #t)]
            #camel_ident(crate::db::SortOrder)
        }
    });
    let pat_arms_name = filterable_fields.map(|field| {
        let t = field.ident.as_ref().unwrap();

        let t_str = &t.to_string();
        let camel_ident = Ident::new(&t.to_string().to_case(Case::Pascal), field.span());
        quote! {
            #new_ident_name::#camel_ident(order) => format!("{} {}", #t_str, match order{
                crate::db::SortOrder::Asc => "ASC",
                crate::db::SortOrder::Desc => "DESC"
            })
        }
    });

    (quote! {

        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        pub enum #new_ident_name{
            #(#enum_variants),*
        }
        impl #new_ident_name{
            pub fn to_sql(&self) -> String{
                match self{
                    #(#pat_arms_name),*
                }
            }
        }
    })
    .into()
}
