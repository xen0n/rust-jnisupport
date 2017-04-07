use quote;
use syn;

use quote::ToTokens;

use mangling;


fn panic_usage() -> ! {
    panic!("incorrect usage; please consult documentation");
}


pub fn jni_export_impl(args: syn::Attribute, body: syn::Item) -> quote::Tokens {
    println!("DEBUG: args = {:?}", args);
    println!("DEBUG: body = {:?}", body);

    // pull params out of args
    let (class, name, sig) = match args.value {
        syn::MetaItem::List(_, params) => {
            let mut class = None;
            let mut name = None;
            let mut sig = None;

            for item in params {
                use syn::NestedMetaItem;
                use syn::MetaItem::NameValue;
                use syn::Lit::Str;

                match item {
                    NestedMetaItem::MetaItem(NameValue(k, Str(v, _))) => {
                        let k: &str = k.as_ref();
                        match k {
                            "class" => class = Some(v),
                            "name" => name = Some(v),
                            "sig" => sig = Some(v),
                            _ => panic_usage(),
                        }
                    }
                    _ => panic_usage(),
                }
            }

            match (class, name, sig) {
                (Some(class), Some(name), Some(sig)) => (class, name, sig),
                _ => panic_usage(),
            }
        }
        _ => panic_usage(),
    };

    // automatically replace dots in class path with slashes for better
    // ergonomics
    let class = class.replace('.', "/");

    println!("DEBUG: class = {}", class);
    println!("DEBUG: name = {}", name);
    println!("DEBUG: sig = {}", sig);

    let sym_name = syn::Ident::new(mangling::get_symbol_name(class, name, sig));
    println!("DEBUG: symbol name = {:?}", sym_name);

    // TODO
    let mut result = quote::Tokens::new();
    body.to_tokens(&mut result);

    result
}
