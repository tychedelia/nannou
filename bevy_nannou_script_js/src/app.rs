use bevy::prelude::info;
// NOTE: this example requires the `console` feature to run correctly.
use boa_engine::object::builtins::JsArray;
use boa_engine::value::Numeric;
use boa_engine::{
    class::{Class, ClassBuilder},
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    property::Attribute,
    Context, JsArgs, JsData, JsResult, JsString, JsValue, Source,
};
use boa_gc::{Finalize, Trace};
use boa_runtime::Console;

#[derive(Debug, Trace, Finalize, JsData)]
pub struct JsApp {
    pub elapsed_seconds: f32,
    pub mouse: (f32, f32),
    pub window_rect: (f32, f32),
}

impl JsApp {
    fn elapsed_seconds(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(app) = object.downcast_ref::<JsApp>() {
                return Ok(JsValue::from(Numeric::from(app.elapsed_seconds)));
            }
        }
        Err(JsNativeError::typ()
            .with_message("'this' is not a JsApp object")
            .into())
    }

    fn mouse(this: &JsValue, _: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(app) = object.downcast_ref::<JsApp>() {
                let array = JsArray::new(ctx);
                array.push(JsValue::from(app.mouse.0), ctx)?;
                array.push(JsValue::from(app.mouse.1), ctx)?;
                return Ok(JsValue::from(array));
            }
        }
        Err(JsNativeError::typ()
            .with_message("'this' is not a JsApp object")
            .into())
    }

    fn window_rect(this: &JsValue, _: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(app) = object.downcast_ref::<JsApp>() {
                let array = JsArray::new(ctx);
                array.push(JsValue::from(app.window_rect.0), ctx)?;
                array.push(JsValue::from(app.window_rect.1), ctx)?;
                return Ok(JsValue::from(array));
            }
        }
        Err(JsNativeError::typ()
            .with_message("'this' is not a JsApp object")
            .into())
    }
}

impl Class for JsApp {
    const NAME: &'static str = "App";
    const LENGTH: usize = 0;

    fn data_constructor(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Err(JsNativeError::typ()
            .with_message("'App' cannot be constructed!")
            .into())
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class.method(
            js_string!("elapsedSeconds"),
            0,
            NativeFunction::from_fn_ptr(Self::elapsed_seconds),
        );
        class.method(
            js_string!("mouse"),
            0,
            NativeFunction::from_fn_ptr(Self::mouse),
        );
        class.method(
            js_string!("windowRect"),
            0,
            NativeFunction::from_fn_ptr(Self::window_rect),
        );

        Ok(())
    }
}
