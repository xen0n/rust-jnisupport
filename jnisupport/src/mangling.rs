use std::fmt::Write;

use signature::MethodSignature;


pub fn get_symbol_name<S, T>(class: S, method: T, sig: &MethodSignature) -> String
    where S: AsRef<str>,
          T: AsRef<str>
{
    let mut result = String::new();

    result.push_str("Java_");
    result.push_str(&mangle_name(class));
    result.push('_');
    result.push_str(&mangle_name(method));
    result.push('_');
    result.push('_');

    result.push_str(&mangle_name(sig.args_string()));

    result.shrink_to_fit();
    result
}


fn mangle_name<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref();
    let mut result = String::with_capacity(s.len());

    for ch in s.chars() {
        match ch {
            '0'...'9' | 'A'...'Z' | 'a'...'z' => result.push(ch),
            '/' => result.push('_'),
            '_' => result.push_str("_1"),
            ';' => result.push_str("_2"),
            '[' => result.push_str("_3"),
            _ => {
                let mut buf = [0u16; 2];
                let buf = ch.encode_utf16(&mut buf);
                for i in buf {
                    result.push_str("_0");
                    write!(result, "{:04x}", i).unwrap();
                }
            }
        }
    }

    result.shrink_to_fit();
    result
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_get_symbol_name() {
        assert_eq!(&get_symbol_name("pkg/Cls", "f", "(ILjava/lang/String;)D"),
                   "Java_pkg_Cls_f__ILjava_lang_String_2");
    }


    #[test]
    fn test_mangle_name() {
        assert_eq!(&mangle_name(""), "");
        assert_eq!(&mangle_name("abc"), "abc");
        assert_eq!(&mangle_name("java/lang/String"), "java_lang_String");
        assert_eq!(&mangle_name("Ljava/lang/String;"), "Ljava_lang_String_2");
        assert_eq!(&mangle_name("[III"), "_3III");
        assert_eq!(&mangle_name("L测试;"), "L_06d4b_08bd5_2");
        assert_eq!(&mangle_name("L𝕊;"), "L_0d835_0dd4a_2");
    }
}
