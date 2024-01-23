#![feature(let_chains)]
use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemEnum, Meta, Expr, Fields, Index, Lit, Member, LitStr, ItemFn, ItemStruct, Ident};
use quote::{quote, format_ident};
use syn::Meta::Path;
use convert_case::{Case, Casing};



#[proc_macro_derive(NormalError, attributes(msg))]
pub fn normal_error(input: TokenStream) -> TokenStream {
    
    let input = parse_macro_input!(input as ItemEnum);
    let ident_name = input.ident;
    let pat_arms = input.variants.iter().map(|variant|{
        let mut literal_text: Option<LitStr> = None; 
        variant.attrs.iter().for_each(|attr|{
            
            if let Meta::NameValue(ref m) = attr.meta &&
            let Expr::Lit(ref e) = m.value &&
            let Lit::Str(ref s) = e.lit{
                literal_text = Some(s.clone());
            }
        });
        if let None = literal_text {
            panic!("couldn'd find any attr with msg");
        }
        let members = variant.fields.iter().enumerate().map(|(i,field)|{
            field.ident.clone()
            .map(Member::Named)
            .unwrap_or_else(||{Member::Unnamed(Index { 
                index: i as u32, span: field.span() 
            })})
        });
        let pat = match variant.fields{
            Fields::Named(_) => quote!({ #(#members),* }),
            Fields::Unit => quote!(),
            Fields::Unnamed(_) => {
                let vars = members.map(|member| match member {
                    Member::Unnamed(member) => format_ident!("_{}", member),
                    Member::Named(_) => unreachable!(),
                });
                quote!((#(#vars),*))
            }
        };
        let temp=variant.ident.clone();
        quote!{
            #ident_name::#temp #pat => crate::utils::error_handling::AppError::ExpectedError(format!(#literal_text))
        }
    });
    quote!{
        impl crate::utils::response::NormalErrorHelper for #ident_name{
            fn to_error(&self) -> crate::utils::error_handling::AppError{
                match self{
                    #(#pat_arms,)*
                }
            }
        }
        use crate::utils::response::NormalErrorHelper;
        impl<T> From<#ident_name> for AppResult<T>{
            fn from(e: #ident_name) -> AppResult<T>{
                Err(e.to_error())
            }
        }
        impl From<#ident_name> for AppError{
            fn from(e: #ident_name) -> AppError{
                e.to_error()
            }
        }
    }.into()
}

#[proc_macro_attribute]
pub fn handler_with_instrument(_args: TokenStream, input: TokenStream) -> TokenStream {
    
    let input = parse_macro_input!(input as ItemFn);
    quote!{
        #[::salvo::handler]
        #[::tracing::instrument(fields(uri = req.uri().path(), method = req.method().as_str(), request_id=depot.get::<String>("request_id").unwrap_or(&String::from("unknown"))),skip_all)]
        #input
        
    }.into()
}

#[proc_macro_derive(FilterParams, attributes(filterable))]
pub fn filter_params(input: TokenStream) -> TokenStream {
    
    let input = parse_macro_input!(input as ItemStruct);
    let ident_name = input.ident;
    let new_ident_name = format_ident!("{}Filterable", ident_name);
    let filterable_fields= input.fields.iter().filter(|field| field.attrs.iter().any(|t|  {
        if let Path(p) = &t.meta && p.is_ident("filterable"){
            true
        }else{
            false
        }
    } ));
    let enum_variants = filterable_fields.clone().map(|field| {
        let t = field.ident.clone().unwrap();
        let camel_ident = Ident::new(&t.to_string().to_case(Case::Pascal), t.span());
        let t = t.to_string();
        let ty = &field.ty;
        quote!{
            #[serde(rename = #t)]
            #camel_ident(#ty)
        }
    });
    let pat_arms_value = filterable_fields.clone().map(|field| {
        let t = field.ident.as_ref().unwrap();
        let camel_ident = Ident::new(&t.to_string().to_case(Case::Pascal), field.span());
        quote!{
            #new_ident_name::#camel_ident(k) => instance.bind(k)
        }
    });
    let pat_arms_name = filterable_fields.map(|field| {
        let t = field.ident.as_ref().unwrap();

        let t_str = &t.to_string();
        let camel_ident = Ident::new(&t.to_string().to_case(Case::Pascal), field.span());
        quote!{
            #new_ident_name::#camel_ident(_) => #t_str
        }
    });
    quote!{

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
    }.into()
}

#[proc_macro_derive(SortParams, attributes(sortable))]
pub fn sort_params(input: TokenStream) -> TokenStream {
    
    let input = parse_macro_input!(input as ItemStruct);
    let ident_name = input.ident;
    let new_ident_name = format_ident!("{}Sortable", ident_name);
    let filterable_fields= input.fields.iter().filter(|field| field.attrs.iter().any(|t|  {
        if let Path(p) = &t.meta && p.is_ident("sortable"){
            true
        }else{
            false
        }
    } ));
    let enum_variants = filterable_fields.clone().map(|field| {
        let t = field.ident.clone().unwrap();
        let camel_ident = Ident::new(&t.to_string().to_case(Case::Pascal), t.span());
        let t = t.to_string();
        quote!{
            #[serde(rename = #t)]
            #camel_ident(crate::db::SortOrder)
        }
    });
    let pat_arms_name = filterable_fields.map(|field| {
        let t = field.ident.as_ref().unwrap();

        let t_str = &t.to_string();
        let camel_ident = Ident::new(&t.to_string().to_case(Case::Pascal), field.span());
        quote!{
            #new_ident_name::#camel_ident(order) => format!("{} {}", #t_str, match order{
                crate::db::SortOrder::Asc => "ASC",
                crate::db::SortOrder::Desc => "DESC"
            })
        }
    });
    
    quote!{

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
    }.into()
}