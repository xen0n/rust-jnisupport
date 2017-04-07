#![feature(proc_macro)]

extern crate jni_macros;

use jni_macros::jni_export;


#[jni_export(class = "com.example.test", name = "testMethod", sig = "(I)I")]
fn test_method(x: i32) -> i32 {
    x + 1
}
