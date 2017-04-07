use quote;
use syn;

use quote::ToTokens;


pub fn jni_export_impl(args: syn::Attribute, body: syn::Item) -> quote::Tokens {
    println!("DEBUG: args = {:?}", args);
    println!("DEBUG: body = {:?}", body);

    // TODO
    let mut result = quote::Tokens::new();
    body.to_tokens(&mut result);

    result
}
