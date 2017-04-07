use quote;
use syn;

use quote::ToTokens;

use mangling;
use signature;


fn panic_usage() -> ! {
    panic!("incorrect usage; please consult documentation");
}


fn separate_method_name<'a>(path: &'a str) -> (&'a str, &'a str) {
    match path.rfind('.') {
        Some(idx) => (&path[..idx], &path[idx + 1..]),
        None => panic!("illegal fully qualified method name"),
    }
}


pub fn jni_export_impl(args: syn::Attribute, body: syn::Item) -> quote::Tokens {
    // get rust function name
    let fn_name: &str = body.ident.as_ref();

    // pull params out of args
    let (class, name, sig) = match args.value {
        syn::MetaItem::List(_, params) => {
            let mut class = None;
            let mut name = None;
            let mut sig = None;

            #[derive(Copy, Clone, PartialEq, Eq)]
            enum InvocationMode {
                TBD,
                ClassNameSig,
                PathSig,
            }

            let mut mode = InvocationMode::TBD;

            for (idx, item) in params.into_iter().enumerate() {
                use syn::NestedMetaItem;
                use syn::MetaItem::NameValue;
                use syn::Lit::Str;

                match item {
                    NestedMetaItem::MetaItem(NameValue(k, Str(v, _))) => {
                        match mode {
                            InvocationMode::TBD => {
                                mode = InvocationMode::ClassNameSig;
                            }
                            InvocationMode::ClassNameSig => {}
                            _ => panic_usage(),
                        }

                        let k: &str = k.as_ref();
                        match k {
                            "class" => class = Some(v),
                            "name" => name = Some(v),
                            "sig" => sig = Some(v),
                            _ => panic_usage(),
                        }
                    }
                    NestedMetaItem::Literal(Str(v, _)) => {
                        match mode {
                            InvocationMode::TBD => {
                                mode = InvocationMode::PathSig;
                            }
                            InvocationMode::PathSig => {}
                            _ => panic_usage(),
                        }

                        match idx {
                            0 => {
                                let (c, n) = separate_method_name(&v);
                                class = Some(c.to_owned());
                                name = Some(n.to_owned());
                            }
                            1 => sig = Some(v),
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

    let sym_name = syn::Ident::new(mangling::get_symbol_name(class, name, &sig));

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

    // pass through original function
    let mut result = quote::Tokens::new();
    body.to_tokens(&mut result);

    // append export fn to token stream
    let export_fn = emit_export_fn(fn_name, sym_name, arg_names, arg_ty, ret_ty);
    export_fn.to_tokens(&mut result);

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
