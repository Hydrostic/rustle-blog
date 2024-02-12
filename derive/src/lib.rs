#![feature(let_chains)]


mod handler;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::Meta::Path;
use syn::{parse_macro_input, Error, Ident, Item, ItemStruct, LitInt, LitStr, Attribute};
enum ErrorType {
    User,
    Internal,
}
impl ErrorType {
    fn get_path(&self) -> proc_macro2::TokenStream{
        match self {
            ErrorType::User => quote!(crate::types::err::AppError::new_user),
            ErrorType::Internal => quote!(crate::types::err::AppError::new_internal)
        }
    }
}
enum ErrorConfig<'a>{
    ItemStruct{
        msg: &'a mut Option<String>,
        code: &'a mut Option<u16>,
    },
    ItemEnum{
        enable_whole_default: &'a mut bool,
    }
}

fn process_root_attr(attrs: &Vec<Attribute>,
                     config: &mut ErrorConfig,
                     error_type: &mut ErrorType) -> Result<(), Error>{
    for attr in attrs {
        if !attr.meta.path().is_ident("err") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("msg") && let ErrorConfig::ItemStruct{msg, .. } = config{
                let value = meta.value()?;
                **msg = Some(value.parse::<LitStr>()?.value());
                return Ok(());
            }
            if meta.path.is_ident("default_msg") && let ErrorConfig::ItemEnum{enable_whole_default} = config {
                **enable_whole_default = true;
                return Ok(());
            }
            if meta.path.is_ident("code") && let ErrorConfig::ItemStruct{code, .. } = config {
                let value = meta.value()?;
                let u16_code: u16 = value.parse::<LitInt>()?.base10_parse()?;
                ntex::http::StatusCode::from_u16(u16_code)
                    .map_err(|_| meta.error("invalid code"))?;
                *(*code) = Some(u16_code);
                return Ok(());
            }
            if meta.path.is_ident("user") {
                *error_type = ErrorType::User;
                return Ok(());
            }
            if meta.path.is_ident("internal") {
                return Ok(());
            }
            Err(meta.error("unrecognized err expr"))
        })?;
    }
    Ok(())
}
#[proc_macro_derive(ErrorHelper, attributes(err))]
pub fn error_helper(input: TokenStream) -> TokenStream {
    let item: Item = syn::parse(input).unwrap();
    match item {
        Item::Struct(input) => {
            let struct_name = input.ident;
            let mut msg: Option<String> = None;
            let mut code: Option<u16> = None;
            let mut error_type = ErrorType::Internal;
            if let Err(e) = process_root_attr(&input.attrs, &mut ErrorConfig::ItemStruct {
                                                  msg: &mut msg,
                                                  code: &mut code }, &mut error_type) {
                return e.into_compile_error().into();
            }
            let msg = msg.unwrap_or_else(|| struct_name.to_string().to_case(Case::Lower));
            let err_path = error_type.get_path();
            if code.is_none(){
                code = Some(match error_type{
                    ErrorType::User => 200,
                    ErrorType::Internal => 500
                })
            }
            (quote!{
                impl From<#struct_name> for crate::types::err::AppError {
                    fn from(value: #struct_name) -> Self{
                        #err_path(::ntex::http::StatusCode::from_u16(#code).unwrap(), #msg)
                    }
                }
                impl #struct_name{
                    pub fn to_error(self) -> crate::types::err::AppError{
                        self.into()
                    }
                }
            }).into()
        }
        Item::Enum(input) => {
            let enum_name = input.ident;
            let mut enable_whole_default = false;
            let mut error_type = ErrorType::Internal;
            if let Err(e) = process_root_attr(&input.attrs, &mut ErrorConfig::ItemEnum {
                enable_whole_default: &mut enable_whole_default }, &mut error_type) {
                return e.into_compile_error().into();
            }
            let pat_arms = input.variants.iter().map(|variant| -> Result<proc_macro2::TokenStream, Error>{
                let variant_name = &variant.ident;
                let mut msg: Option<String> = None;
                let mut code: Option<u16> = None;
                for variant_attr in &variant.attrs{
                    if variant_attr.path().is_ident("err") {
                        variant_attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("msg"){
                                let value = meta.value()?;
                                msg = Some(value.parse::<LitStr>()?.value());
                                return Ok(());
                            }
                            if meta.path.is_ident("code"){
                                let value = meta.value()?;
                                let u16_code: u16 = value.parse::<LitInt>()?.base10_parse()?;
                                ntex::http::StatusCode::from_u16(u16_code).map_err(|_| meta.error("invalid code"))?;
                                code = Some(u16_code);
                                return Ok(());
                            }
                            Err(meta.error("unrecognized err expr"))
                        })?;
                    }
                }
                if enable_whole_default && msg.is_none(){
                    msg = Some(variant_name.to_string().to_case(Case::Lower));
                } else if msg.is_none() {
                    return Err(Error::new(variant.span(),
                                      "msg not found in attrs, if you intent to use the default, add #[err(default_msg)] to enum")
                        );
                }
                if code.is_none(){
                    code = Some(match error_type{
                        ErrorType::User => 200,
                        ErrorType::Internal => 500
                    })
                }
                Ok(quote!(#enum_name::#variant_name=>(#msg,#code)))
            }).collect::<Result<Vec<proc_macro2::TokenStream>, Error>>();
            if let Err(e) = pat_arms{
                return e.into_compile_error().into();
            }
            let pat_arms = pat_arms.unwrap();
            let err_path = error_type.get_path();
            (quote!{
                impl From<#enum_name> for crate::types::err::AppError {
                    fn from(value: #enum_name) -> Self{
                        let (msg, code) = match value{
                            #(#pat_arms,)*
                        };
                        #err_path(::ntex::http::StatusCode::from_u16(code).unwrap(), msg)
                    }
                }
                impl #enum_name{
                    pub fn to_error(self) -> crate::types::err::AppError{
                        self.into()
                    }
                }
            }).into()
        }
        _ => Error::new(item.span(), "expect a enum or struct")
            .to_compile_error()
            .into(),
    }
}



#[proc_macro_derive(FilterParams, attributes(filterable))]
pub fn filter_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let ident_name = input.ident;
    let new_ident_name = format_ident!("{}Filterable", ident_name);
    let filterable_fields = input.fields.iter().filter(|field| {
        field.attrs.iter().any(|t| {
            if let Path(p) = &t.meta
                && p.is_ident("filterable")
            {
                true
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
            if let Path(p) = &t.meta
                && p.is_ident("sortable")
            {
                true
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

#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    handler::parse_handler(args, input)
}