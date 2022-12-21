#[macro_use]
extern crate lazy_static;
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::{collections::HashMap, sync::Mutex};
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, ItemFn, LitStr, Token};

#[derive(Debug)]
struct CommandParams {
    name: String,
    description: String,
    handler: Option<String>,
    function: Option<String>,
}

impl Parse for CommandParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vars = Punctuated::<LitStr, Token![,]>::parse_terminated(input)?;
        Ok(CommandParams {
            name: vars[0].value(),
            description: vars[1].value(),
            handler: None,
            function: None,
        })
    }
}

lazy_static! {
    static ref HASHMAP: Mutex<HashMap<String, Box<CommandParams>>> = Mutex::new(HashMap::new());
}

#[proc_macro_attribute]
pub fn command(attribute: TokenStream, function: TokenStream) -> TokenStream {
    let mut args = parse_macro_input!(attribute as CommandParams);
    args.handler = Some(function.to_string());
    let ItemFn { sig, .. } = syn::parse(function).expect("failed to parse function");
    args.function = Some(sig.ident.to_string());
    HASHMAP
        .lock()
        .unwrap()
        .insert(args.name.clone(), Box::new(args));
    quote! {}.into()
}

#[proc_macro]
pub fn command_tree(input: TokenStream) -> TokenStream {
    let ident = parse_macro_input!(input as syn::Ident);
    let functions: Vec<syn::Item> = HASHMAP
        .lock()
        .unwrap()
        .values()
        .map(|item| {
            syn::parse_str(&item.handler.as_ref().expect("Handler").to_string())
                .expect("unable to parse")
        })
        .collect();

    if functions.len() == 0 {
        return quote! {}.into();
    }

    let cases: Vec<syn::Arm> = HASHMAP
        .lock()
        .unwrap()
        .values()
        .map(|item| {
            syn::parse_str(
                format!(
                    "\"{}\" => {}()",
                    item.name,
                    item.function.as_ref().expect("Function name")
                )
                .as_str(),
            )
            .expect("valid expression")
        })
        .collect();

    let code = quote! {
        #(#functions)*

        match #ident {
            #(#cases),*,
            _ => panic!("unhandled command")
        }
    };

    code.into()
}

#[proc_macro]
pub fn commands(_input: TokenStream) -> TokenStream {
    let cmds: Vec<syn::ExprTuple> = HASHMAP
        .lock()
        .unwrap()
        .values()
        .map(|item| {
            syn::parse_str(format!("(\"{}\",\"{}\")", item.name, item.description).as_str())
                .expect("valid tuple")
        })
        .collect();

    if cmds.len() == 0 {
        return quote! {
            []
        }
        .into();
    }

    let code = quote! {
        [#(#cmds),*]
    };

    code.into()
}
