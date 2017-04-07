use quote;
use syn;

use quote::ToTokens;

use mangling;
use signature;


fn panic_usage() -> ! {
    panic!("incorrect usage; please consult documentation");
}


pub fn jni_export_impl(args: syn::Attribute, body: syn::Item) -> quote::Tokens {
    println!("DEBUG: args = {:?}", args);
    println!("DEBUG: body = {:?}", body);

    // get rust function name
    let fn_name: &str = body.ident.as_ref();

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
    let sig = signature::MethodSignature::from_utf8(sig);

    println!("DEBUG: class = {}", class);
    println!("DEBUG: name = {}", name);
    println!("DEBUG: sig = {:?}", sig);
    println!("DEBUG: rust fn name = {}", fn_name);

    let sym_name = syn::Ident::new(mangling::get_symbol_name(class, name, &sig));
    println!("DEBUG: symbol name = {:?}", sym_name);

    // generate signature
    let fn_name = syn::Ident::new(fn_name);
    let arg_names: Vec<_> = (0..sig.args().len())
        .map(|i| syn::Ident::new(format!("arg{}", i)))
        .collect();
    let arg_ty: Vec<_> = sig.args()
        .iter()
        .map(|ty| jni_ident_from_ty(ty))
        .collect();
    let ret_ty = match sig.ret() {
        Some(ty) => Some(jni_ident_from_ty(ty)),
        None => None,
    };
    println!("DEBUG: args ty = {:?}", arg_ty);
    println!("DEBUG: ret ty = {:?}", ret_ty);

    // pass through original function
    let mut result = quote::Tokens::new();
    body.to_tokens(&mut result);

    // append export fn to token stream
    let export_fn = emit_export_fn(fn_name, sym_name, arg_names, arg_ty, ret_ty);
    export_fn.to_tokens(&mut result);

    println!("DEBUG: result = {:?}", result);

    result
}


fn emit_export_fn(name: syn::Ident,
                  sym_name: syn::Ident,
                  arg_names: Vec<syn::Ident>,
                  arg_ty: Vec<syn::Ident>,
                  ret: Option<syn::Ident>)
                  -> quote::Tokens {
    let ret = match ret {
        Some(ty) => vec![ty],
        None => vec![],
    };
    let arg_names1 = arg_names.clone();

    // TODO: support non-static methods
    quote!(
        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn #sym_name(vm: ::jni::JNIEnv,
                                    clazz: ::jni::objects::JClass,
                                    #(#arg_names1: #arg_ty),*
                                    ) #(-> #ret)* {
            #name(vm, clazz, #(#arg_names),*)
        }
    )
}


fn jni_ident_from_ty(ty: &signature::JavaTy) -> syn::Ident {
    use signature::JavaTy;

    let ident = match ty {
        &JavaTy::Boolean => "::jni::sys::jboolean",
        &JavaTy::Byte => "::jni::sys::jbyte",
        &JavaTy::Char => "::jni::sys::jchar",
        &JavaTy::Short => "::jni::sys::jshort",
        &JavaTy::Int => "::jni::sys::jint",
        &JavaTy::Long => "::jni::sys::jlong",
        &JavaTy::Float => "::jni::sys::jfloat",
        &JavaTy::Double => "::jni::sys::jdouble",

        &JavaTy::Array(_) => {
            // TODO: differentiate between the distinct names although
            // identical otherwise
            "::jni::sys::jarray"
        }

        &JavaTy::Class(ref s) => {
            match s.as_str() {
                "java/lang/String" => "::jni::objects::JString",
                _ => "::jni::objects::JObject",
            }
        }
    };

    syn::Ident::new(ident)
}
