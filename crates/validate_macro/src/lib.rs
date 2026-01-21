use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Path};

#[proc_macro_attribute]
pub fn sync_validate_return(attr: TokenStream, item: TokenStream) -> TokenStream {
    let validator = parse_macro_input!(attr as Path);
    let mut input = parse_macro_input!(item as ItemFn);

    if input.sig.asyncness.is_some() {
        return TokenStream::from(quote! {
            compile_error!("sync_validate_return cannot be used on async functions");
        });
    }

    let block = input.block;
    let wrapped_block: syn::Block = syn::parse2(quote!({
        let __return_result = (|| #block)();
        match __return_result {
            Ok(__value) => {
                if let Err(__err) = #validator(&__value) {
                    return Err(__err.into());
                }
                Ok(__value)
            }
            Err(__err) => Err(__err),
        }
    }))
    .expect("failed to parse generated block");

    input.block = Box::new(wrapped_block);
    TokenStream::from(quote!(#input))
}

#[proc_macro_attribute]
pub fn async_validate_return(attr: TokenStream, item: TokenStream) -> TokenStream {
    let validator = parse_macro_input!(attr as Path);
    let mut input = parse_macro_input!(item as ItemFn);

    if input.sig.asyncness.is_none() {
        return TokenStream::from(quote! {
            compile_error!("async_validate_return requires an async function");
        });
    }

    let block = input.block;
    let wrapped_block: syn::Block = syn::parse2(quote!({
        let __return_result = (|| async move #block)().await;
        match __return_result {
            Ok(__value) => {
                if let Err(__err) = #validator(&__value) {
                    return Err(__err.into());
                }
                Ok(__value)
            }
            Err(__err) => Err(__err),
        }
    }))
    .expect("failed to parse generated block");

    input.block = Box::new(wrapped_block);
    TokenStream::from(quote!(#input))
}
