use std::collections::HashMap;

use itertools::izip;
use quote::format_ident;
use syn::{
    parse_quote, parse_quote_spanned, punctuated::Punctuated, spanned::Spanned, token::Comma, Arm,
    Expr, FnArg, ItemEnum, ItemTrait, Pat, Stmt, TraitItem, Type,
};

use crate::handle_interface::pascal_to_snake;

pub fn generate_trait(
    original: ItemEnum,
    message: ItemEnum,
    response: ItemEnum,
    _gates: HashMap<String, String>,
) -> ItemTrait {
    let name = format_ident!("{}Trait", original.ident);
    let vis = &original.vis;
    let variants = izip![
        original.variants.to_owned(),
        message.variants.to_owned(),
        response.variants.to_owned()
    ]
    .collect::<Vec<_>>();
    let methods = variants
        .iter()
        .map(|(original, message, response)| -> TraitItem {
            let name = format_ident!("{}", pascal_to_snake(original.ident.to_string()));

            let args = message
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| -> FnArg {
                    let name = format_ident!("arg{}", i + 1);
                    let ty = &f.ty;
                    parse_quote_spanned! {f.span()=>#name: #ty}
                })
                .collect::<Punctuated<_, Comma>>();

            let return_type: Type = {
                let types = response
                    .fields
                    .iter()
                    .map(|f| f.ty.to_owned())
                    .collect::<Punctuated<_, Comma>>();
                if let Some(ty) = types.first()
                    && types.len() == 1
                {
                    ty.to_owned()
                } else {
                    parse_quote_spanned!(original.span()=>(#types))
                }
            };
            parse_quote_spanned!(original.span()=>fn #name(&mut self, #args) -> #return_type;)
        })
        .collect::<Vec<_>>();

    let arms = variants
        .iter()
        .zip(&methods)
        .map(|((original_v, message_v, response_v), method)| -> Arm {
            let message_idents = message_v
                .fields
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("arg{}", i + 1))
                .collect::<Punctuated<_, Comma>>();

            let response_idents = response_v
                .fields
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("arg{}", i + 1))
                .collect::<Punctuated<_, Comma>>();

            let pat: Pat = {
                let ty = &message.ident;
                let v = &message_v.ident;
                if message_idents.len() > 0 {
                    parse_quote!(#ty::#v(#message_idents))
                } else {
                    parse_quote!(#ty::#v)
                }
            };
            let method_call: Option<Stmt> = if let TraitItem::Fn(method) = method {
                let method_ident = &method.sig.ident;
                Some(parse_quote!(let (#response_idents) = self.#method_ident(#message_idents);))
            } else {
                None
            };
            let return_expr: Expr = {
                let ty = &response.ident;
                let v = &response_v.ident;
                if response_idents.len() > 0 {
                    parse_quote!(#ty::#v(#response_idents))
                } else {
                    parse_quote!(#ty::#v)
                }
            };
            parse_quote_spanned!(original_v.span()=>#pat => {
                #method_call
                #return_expr
            })
        })
        .collect::<Vec<_>>();

    let message_name = &message.ident;
    parse_quote_spanned!(original.span()=>
    #vis trait #name {
        #(#methods)*
        fn main_loop(mut self) -> ! where Self: Sized {
                    loop {
                        (|| -> Result<(), Box<dyn std::error::Error>> {
                            let response = match from_read::<_, #message_name>(stdin())? {
                                #(#arms)*
                            };
                            stdout().write_all(&to_vec(&Ok::<_, String>(response))?)?;
                            Ok(())
                        })()
                        .unwrap_or_else(|e| {
                            let _ = stdout().write_all(&to_vec(&Err::<(), _>(format!("{e:#?}"))).unwrap_or_default());
                        });
                    }
                }
            }
        )
}
