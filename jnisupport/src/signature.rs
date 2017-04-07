use std::borrow::Cow;
use std::str;


#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum JavaTy {
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Class(String),
    // this is impossible, have to workaround
    // Array(JavaTy),
    Array(String),
}


impl JavaTy {
    fn primitive_from_char(s: char) -> JavaTy {
        match s {
            'Z' => JavaTy::Boolean,
            'B' => JavaTy::Byte,
            'C' => JavaTy::Char,
            'S' => JavaTy::Short,
            'I' => JavaTy::Int,
            'J' => JavaTy::Long,
            'F' => JavaTy::Float,
            'D' => JavaTy::Double,
            _ => unreachable!(),
        }
    }

    fn class_from_u8(s: &[u8]) -> JavaTy {
        JavaTy::Class(str::from_utf8(s).unwrap().to_owned())
    }

    fn array_from_ty(ty: JavaTy) -> JavaTy {
        JavaTy::Array(ty.to_str().into_owned())
    }

    fn to_str(&self) -> Cow<'static, str> {
        match self {
            &JavaTy::Boolean => "Z".into(),
            &JavaTy::Byte => "B".into(),
            &JavaTy::Char => "C".into(),
            &JavaTy::Short => "S".into(),
            &JavaTy::Int => "I".into(),
            &JavaTy::Long => "J".into(),
            &JavaTy::Float => "F".into(),
            &JavaTy::Double => "D".into(),
            &JavaTy::Class(ref cls) => format!("L{};", cls).into(),
            &JavaTy::Array(ref ty) => format!("[{}", ty).into(),
        }
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MethodSignature {
    args: Vec<JavaTy>,
    ret: Option<JavaTy>,
}


impl MethodSignature {
    pub fn from_utf8<S: AsRef<[u8]>>(s: S) -> MethodSignature {
        t_method(s.as_ref()).unwrap().1
    }

    pub fn args<'a>(&'a self) -> &'a [JavaTy] {
        &self.args
    }

    pub fn ret<'a>(&'a self) -> Option<&'a JavaTy> {
        self.ret.as_ref()
    }

    pub fn args_string(&self) -> String {
        let mut result = String::new();
        for arg in &self.args {
            match arg.to_str() {
                Cow::Borrowed(s) => result.push_str(s),
                Cow::Owned(s) => result.push_str(&s),
            }
        }

        result.shrink_to_fit();
        result
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::JavaTy::*;


    #[test]
    fn test_signature_parse() {
        assert_eq!(MethodSignature::from_utf8("()"),
                   MethodSignature {
                       args: vec![],
                       ret: None,
                   });
        assert_eq!(MethodSignature::from_utf8("()I"),
                   MethodSignature {
                       args: vec![],
                       ret: Some(Int),
                   });
        assert_eq!(MethodSignature::from_utf8("(I)"),
                   MethodSignature {
                       args: vec![Int],
                       ret: None,
                   });
        assert_eq!(MethodSignature::from_utf8("(ILjava/lang/String;[I)J"),
                   MethodSignature {
                       args: vec![Int,
                                  Class("java/lang/String".to_string()),
                                  Array("I".to_string())],
                       ret: Some(Long),
                   });
    }


    #[test]
    fn test_parser() {
        use nom::IResult::Done;

        macro_rules! t {
            ($ty: ident, $test: expr, $result: expr) => {
                assert_eq!($ty($test), Done(&b""[..], $result));
            }
        }

        t!(t_primitive, b"Z", Boolean);
        t!(t_primitive, b"B", Byte);
        t!(t_primitive, b"C", Char);
        t!(t_primitive, b"S", Short);
        t!(t_primitive, b"I", Int);
        t!(t_primitive, b"J", Long);
        t!(t_primitive, b"F", Float);
        t!(t_primitive, b"D", Double);

        t!(t_class,
           b"Ljava/lang/String;",
           Class("java/lang/String".to_string()));

        t!(t_array, b"[I", Array("I".to_string()));
        t!(t_array, b"[[I", Array("[I".to_string()));
        t!(t_array, b"[[[I", Array("[[I".to_string()));
        t!(t_array,
           b"[[[Ljava/lang/String;",
           Array("[[Ljava/lang/String;".to_string()));

        t!(t_arglist, b"()", vec![]);
        t!(t_arglist,
           b"(IIILjava/lang/String;Z)",
           vec![Int, Int, Int, Class("java/lang/String".to_string()), Boolean]);
    }
}


//
// parser implementation
//

named!(
    t_primitive<&[u8], JavaTy>,
    map!(
        alt!(char!('Z') | char!('B') | char!('C') | char!('S') |
             char!('I') | char!('J') | char!('F') | char!('D')),
        JavaTy::primitive_from_char
        ));

named!(
    t_class<&[u8], JavaTy>,
    map!(
        delimited!(char!('L'), is_not!(";"), char!(';')),
        JavaTy::class_from_u8
        ));

named!(
    t_array<&[u8], JavaTy>,
    do_parse!(
        char!('[') >>
        ty: alt!(t_primitive | t_class | t_array) >>
        (JavaTy::array_from_ty(ty))
    ));

named!(
    t_ty<&[u8], JavaTy>,
    alt!(t_primitive | t_class | t_array)
    );

named!(
    t_arglist< &[u8], Vec<JavaTy> >,
    do_parse!(
        char!('(') >>
        result: many0!(t_ty) >>
        char!(')') >>
        (result)
        ));

named!(
    t_method<&[u8], MethodSignature>,
    do_parse!(
        args: t_arglist >>
        ret: opt!(complete!(t_ty)) >>
        (MethodSignature {
            args: args,
            ret: ret,
        })
        ));
