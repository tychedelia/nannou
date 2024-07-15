use boa_engine::{
    class::{Class, ClassBuilder},
    Context,
    error::JsNativeError,
    js_string
    ,
    JsData, JsResult, JsValue, native_function::NativeFunction,
};
use boa_engine::object::builtins::JsArray;
use boa_engine::value::Numeric;
use boa_gc::{Finalize, Trace};

#[derive(Debug, Trace, Finalize, JsData)]
pub struct JsApp {
    pub time: f32,
    pub mouse: (f32, f32),
    pub window_rect: (f32, f32),
}

impl JsApp {
    fn time(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(app) = object.downcast_ref::<JsApp>() {
                return Ok(JsValue::from(Numeric::from(app.time)));
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
            js_string!("time"),
            0,
            NativeFunction::from_fn_ptr(Self::time),
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
