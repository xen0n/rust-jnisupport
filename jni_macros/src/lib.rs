#![feature(proc_macro)]

extern crate jnisupport;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;


mod utils;


use jnisupport::jni_export;
use proc_macro::TokenStream;


#[proc_macro_attribute]
pub fn jni_export(args: TokenStream, body: TokenStream) -> TokenStream {
    utils::expand_attr("jni_export", args, body, jni_export::jni_export_impl)
}
