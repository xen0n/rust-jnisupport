#![feature(proc_macro)]

extern crate jni;
extern crate jni_macros;

use jni_macros::jni_export;

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jint;


#[jni_export(class = "com.example.test", name = "testMethod", sig = "(I)I")]
fn test_method(_: JNIEnv, _: JClass, x: jint) -> jint {
    x + 1
}
