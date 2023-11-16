#![feature(let_chains)]
use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{parse_macro_input, ItemEnum, Meta, Expr, Fields, Index, Lit, Member, LitStr};
use quote::{quote, format_ident};
extern crate rustle_derive_additional;



#[proc_macro_derive(NormalResponse, attributes(msg))]
pub fn normal_response(input: TokenStream) -> TokenStream {
    
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
            #ident_name::#temp #pat => format!(#literal_text)
        }
    });
    quote!{
        impl ::rustle_derive_additional::MessagePrintable for #ident_name{
            fn print(&self) -> String{
                match self{
                    #(#pat_arms,)*
                }
            }
        }
        
    }.into()
}
