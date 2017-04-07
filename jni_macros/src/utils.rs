use proc_macro::TokenStream;
use quote;
use syn;


type AttrImpl = fn(syn::Attribute, syn::Item) -> quote::Tokens;


pub fn expand_attr(name: &str,
                   args: TokenStream,
                   body: TokenStream,
                   f: AttrImpl) -> TokenStream {
    // FIXME: how to best parse bare parameter list without allocation?
    // like `(class = "")` instead of `#[jni_export(class = "")]`
    let args = {
        let s = format!("#[{}{}]", name, args.to_string());
        syn::parse_outer_attr(&s).unwrap()
    };
    let body = syn::parse_item(&body.to_string()).unwrap();
    f(args, body).to_string().parse().unwrap()
}
