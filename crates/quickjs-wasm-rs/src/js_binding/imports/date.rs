use anyhow::Result;
use quickjs_wasm_sys::{JSContext, JSValue, JS_NewStringLen};
use std::os::raw::{c_char, c_int};
use super::super::{value::Value, context::Context};

pub fn add_to_context(context: &Context) -> Result<()> {
    let f = unsafe { context.new_callback(format_distance())? };
    let date_object = context.object_value()?;
    let global = context.global_object()?;
    date_object.set_property("fmt", f)?;

    global.set_property("date", date_object)?;

    Ok(())
}

static mut RET_AREA: [i64; 2] = [0; 2];

fn format_distance() -> impl FnMut(*mut JSContext, JSValue, c_int, *mut JSValue, c_int) -> JSValue
{

    move |ctx: *mut JSContext, _this: JSValue, argc: c_int, argv: *mut JSValue, _magic: c_int| {

        #[link(wasm_import_module = "date")]
        extern "C"  {
            #[cfg_attr(target_arch = "wasm32", link_name = "fmt")]
            #[cfg_attr(not(target_arch = "wasm32"), link_name = "date_fmt")]
            fn wit_import(_: i32, _:i32, _:i32, _:i32, _:i32);
        }

        unsafe {
            if argc != 2 {
                panic!("expected 2 argument, {} given", argc);
            }

            let arg0 = *argv.offset(0isize);
            let arg1 = *argv.offset(1isize);

            let str0 = Value::new(ctx, arg0).map(|v| v.as_str().unwrap().to_string()).unwrap();
            let ptr0 = str0.as_ptr() as i32;
            let len0 = str0.len() as i32;

            let str1 = Value::new(ctx, arg1).map(|v| v.as_str().unwrap().to_string()).unwrap();
            let ptr1 = str1.as_ptr() as i32;
            let len1 = str1.len() as i32;

            let retptr = RET_AREA.as_mut_ptr() as i32;
            wit_import(ptr0, len0, ptr1, len1, retptr);

            let retptr_len = *((retptr + 8) as *const i32) as usize;
            let ret_str = String::from_utf8(Vec::from_raw_parts(*((retptr + 0) as *const i32) as *mut _, retptr_len, retptr_len)).unwrap();
            JS_NewStringLen(ctx, ret_str.as_ptr() as *const c_char, ret_str.len() as _)
        }
    }
}
