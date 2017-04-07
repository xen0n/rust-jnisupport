#![feature(proc_macro)]

extern crate jni;
extern crate jni_macros;

use jni_macros::jni_export;

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jint;


#[jni_export(class = "com.example.Test", name = "testMethod1", sig = "(I)I")]
fn test_method_1(_: JNIEnv, _: JClass, x: jint) -> jint {
    x + 1
}


#[jni_export("com.example.Test.testMethod2", "(I)I")]
fn test_method_2(_: JNIEnv, _: JClass, x: jint) -> jint {
    x + 2
}
