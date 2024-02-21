use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct, Error, Ident, Type, Meta};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
// enum JoinType{
//     Foreign(Ident),
//     Master(Ident)
// }
pub(crate) fn join_helper(input: TokenStream) -> TokenStream{
    let input = parse_macro_input!(input as ItemStruct);
    let struct_name = &input.ident;
    let fields = match input.fields{
        syn::Fields::Named(t) => t,
        _ => return Error::new(input.ident.span(), "support named struct only").into_compile_error().into(),
    };
    let mut join_key: Option<(TokenStream2, Type)> = None;
    let mut join_object: Option<(Ident, Type)> = None;
    for f in &fields.named{
        for attr in &f.attrs{
            if attr.meta.path().is_ident("foreign_key") {
                let outer_ident = f.ident.clone().unwrap();
                join_key = Some((quote!{#outer_ident}, f.ty.clone()));
                if let Meta::NameValue(_) = attr.meta{
                    return Error::new(attr.span(), "attr should be list or path only").into_compile_error().into();
                } else if let Meta::List(nv) = &attr.meta{
                    if let Err(e) = nv.parse_nested_meta(|meta| {
                        // println!("{:?}", meta.path);
                        if meta.path.is_ident("key"){
                            let key_name: Ident = meta.value()?.parse()?;
                            // let key_name: Ident = meta.parse_args_with()?;
                            join_key.as_mut().unwrap().0 = quote!{#outer_ident.#key_name};
                            return Ok(());
                        }
                        if meta.path.is_ident("type"){
                            // let content;
                            // parenthesized!(content in meta.input);
                            join_key.as_mut().unwrap().1 = meta.value()?.parse()?;
                            return Ok(());
                        }
                        Err(meta.error("unrecognized attr"))
                    }){
                        return e.into_compile_error().into();
                    }
                }
                
                
            } else if attr.meta.path().is_ident("foreign_object"){
                join_object = Some((f.ident.clone().unwrap(), f.ty.clone()));

            }
        }
    }
    if join_key.is_none() || join_object.is_none(){
        return Error::new(input.ident.span(), "no proper foreign key/foreign object attr found").into_compile_error().into();
    }
    let (join_key_name, join_key_ty) = join_key.unwrap();
    let (join_object_name, join_object_ty) = join_object.unwrap();
    (quote!{
        impl crate::types::join::Joinable<#join_key_ty, #join_object_ty> for #struct_name{
            // fn get_key(&self) -> &#join_key_ty{
            //     &self.#join_key_name
            // }
            // fn get_object(&mut self) -> &mut #join_object_ty{
            //     &mut self.#join_object_name
            // }
            fn get_ref(&mut self) -> (&#join_key_ty, &mut #join_object_ty){
                (&self.#join_key_name, &mut self.#join_object_name)
            }
        }
    }).into()
}