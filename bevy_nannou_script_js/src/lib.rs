use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::app::JsApp;
use anyhow::{anyhow, Context as AnyhowContext};
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy::reflect;
use bevy::reflect::{GetTypeRegistration, ReflectMut, ReflectRef, TypeRegistry};
use boa_engine::object::builtins::{JsArray, JsMap};
use boa_engine::object::{Object, ObjectInitializer};
use boa_engine::value::TryFromJs;
use boa_engine::{
    class::{Class, ClassBuilder},
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    property::Attribute,
    Context, JsData, JsObject, JsResult, JsString, JsValue, Source,
};
use boa_gc::{Finalize, Trace};
use boa_runtime::Console;

use crate::asset::{Script, ScriptAssetPlugin};

mod app;
mod asset;

pub mod prelude {
    pub use crate::app::JsApp;
    pub use crate::asset::Script;
    pub use crate::run_script;
    pub use crate::RegisterScriptTypeExt;
    pub use crate::UpdateScript;
    pub use crate::UpdateScriptAssetLocation;
}

trait ReflectIntoJs {
    fn reflect_into_js(&self) -> JsValue;
}

trait ReflectFromJs {
    fn reflect_from_js(js_value: JsValue, ctx: &mut Context) -> Self;
}

type ReflectIntoJsFn = fn(&dyn Reflect) -> JsValue;
type ReflectFromJsFn = fn(JsValue, ctx: &mut Context) -> Box<dyn Reflect>;

macro_rules! register_type {
    ($type:ty, $type_registry:expr) => {
        register_type::<$type>(
            $type_registry,
            |v| JsValue::from(*v.downcast_ref::<$type>().expect("Unable to downcast")),
            |v, ctx| Box::new(<$type>::try_from_js(&v, ctx).expect("Could not convert")),
        );
    };
}

macro_rules! register_numeric_type {
    ($type:ty, $type_registry:expr) => {
        register_type::<$type>(
            $type_registry,
            |v| JsValue::from(*v.downcast_ref::<$type>().expect("Unable to downcast")),
            |v, ctx| {
                let numeric = v
                    .to_numeric(ctx)
                    .expect("Could not convert value to numeric");
                match numeric {
                    boa_engine::value::Numeric::Number(f) => Box::new(f as $type),
                    _ => panic!("Unsupported numeric type"),
                }
            },
        );
    };
}

fn register_type<T: 'static>(
    type_registry: &mut reflect::TypeRegistry,
    into_js_fn: ReflectIntoJsFn,
    from_js_fn: ReflectFromJsFn,
) {
    let t = type_registry
        .get_mut(TypeId::of::<T>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<T>()));
    t.insert(into_js_fn);
    t.insert(from_js_fn);
}

pub trait RegisterScriptTypeExt {
    fn register_script_type<
        T: 'static + Reflect + GetTypeRegistration + ReflectIntoJs + ReflectFromJs,
    >(
        &mut self,
    ) -> &mut Self;
}

impl RegisterScriptTypeExt for App {
    fn register_script_type<
        T: 'static + Reflect + GetTypeRegistration + ReflectIntoJs + ReflectFromJs,
    >(
        &mut self,
    ) -> &mut Self {
        {
            let type_registry = self.world_mut().resource_mut::<AppTypeRegistry>();
            let mut type_registry = type_registry.write();
            register_type::<T>(
                &mut type_registry,
                |v| {
                    let me = v.downcast_ref::<T>().expect("Unable to downcast");
                    T::reflect_into_js(me)
                },
                |v, ctx| Box::new(T::reflect_from_js(v, ctx)),
            );
        }
        self.register_type::<T>()
    }
}

pub struct ScriptJsPlugin;

impl Plugin for ScriptJsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_plugins(ScriptAssetPlugin);
    }

    fn finish(&self, app: &mut App) {
        let type_registry = app.world_mut().resource_mut::<AppTypeRegistry>();
        let mut type_registry = type_registry.write();
        register_type!(u8, &mut type_registry);
        register_type!(u16, &mut type_registry);
        register_type!(u32, &mut type_registry);
        register_type!(u64, &mut type_registry);
        register_type!(i8, &mut type_registry);
        register_type!(i16, &mut type_registry);
        register_type!(i32, &mut type_registry);
        register_type!(i64, &mut type_registry);
        register_numeric_type!(f32, &mut type_registry);
        register_numeric_type!(f64, &mut type_registry);
        register_type!(bool, &mut type_registry);
    }
}

#[derive(Resource)]
pub struct UpdateScriptAssetLocation(pub String);

#[derive(Resource)]
pub struct UpdateScript(pub Handle<Script>);

#[derive(Default, Deref, DerefMut)]
struct JsContext(Context);

trait WorldScope {
    fn with_world_scope<F, R>(&mut self, world: UnsafeWorldCell<'_>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}

impl WorldScope for JsContext {
    fn with_world_scope<F, R>(&mut self, world: UnsafeWorldCell<'_>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let holder = WorldHolder(unsafe { world.world_mut() });
        self.realm().host_defined_mut().insert(holder);
        let result = f(self);
        self.realm().host_defined_mut().remove::<WorldHolder>();
        result
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
struct WorldHolder(#[unsafe_ignore_trace] *mut World);

impl WorldHolder {
    fn world(&self) -> &World {
        unsafe { &*self.0 }
    }

    fn world_mut(&mut self) -> &mut World {
        unsafe { &mut *self.0 }
    }
}

fn setup(world: &mut World) {
    let mut ctx = Context::default();
    add_runtime(&mut ctx);
    world.insert_non_send_resource(JsContext(ctx));
}

fn add_runtime(context: &mut Context) {
    let console = Console::init(context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");

    context
        .register_global_class::<app::JsApp>()
        .expect("the App builtin shouldn't exist");
}

pub fn reflect_to_js_model(
    model: &dyn Reflect,
    type_registry: &TypeRegistry,
    ctx: &mut Context,
) -> JsValue {
    match model.reflect_ref() {
        ReflectRef::Struct(s) => {
            let pairs = (0..s.field_len())
                .map(|idx| {
                    let name = s.name_at(idx).expect("Unable to get field name");
                    let name = JsString::from(name);
                    let field = s.field_at(idx).expect("Unable to get field");
                    let value = reflect_to_js_model(field, type_registry, ctx);
                    (name, value)
                })
                .collect::<Vec<_>>();

            let mut js_obj = ObjectInitializer::new(ctx);
            for ((name, value)) in pairs.into_iter() {
                js_obj
                    .property(name, value, Attribute::all());
            }
            JsValue::from(js_obj.build())
        }
        ReflectRef::Value(v) => {
            if let Some(into_js) = type_registry.get_type_data::<ReflectIntoJsFn>(Any::type_id(v)) {
                into_js(v)
            } else {
                panic!(
                    "Unable to find ReflectIntoJsFn for type: {:?}",
                    Any::type_id(v)
                );
            }
        }
        _ => todo!("Other types of models"),
    }
}

fn write_js_model(
    model: ReflectMut,
    js_value: JsValue,
    type_registry: &TypeRegistry,
    ctx: &mut Context,
) -> anyhow::Result<()> {
    match js_value {
        JsValue::Object(obj) => {
            match model {
                ReflectMut::Struct(s) => {
                    let field_data: Vec<(String, JsValue, TypeId)> = (0..s.field_len())
                        .map(|idx| {
                            let name = s
                                .name_at(idx)
                                .expect("Unable to get field name")
                                .to_string();
                            let js_name = JsString::from(name.as_str());
                            let value = obj
                                .get(js_name, ctx)
                                .map_err(|e| anyhow!("Could not read field from js map"))
                                .expect("Could not read field from js map");
                            let field_type_id =
                                s.field_at(idx).expect("Unable to get field").type_id();
                            (name, value, field_type_id)
                        })
                        .collect();

                    // Now mutate the fields one by one.
                    for (name, value, field_type_id) in field_data {
                        if let Some(mut field) = s.field_mut(&name) {
                            let from_js = type_registry
                                .get_type_data::<ReflectFromJsFn>(field_type_id)
                                .context("Unable to find ReflectFromJs for type")?;
                            let value = from_js(value, ctx);
                            field.set(value).expect("Could not set field value");
                        }
                    }
                }
                _ => todo!("Other types of models"),
            }
        }
        _ => panic!("Unsupported type"),
    }
    Ok(())
}

pub fn run_script(world: &mut World, js_app: JsApp, model: &mut dyn Reflect) {
    let script = world.get_resource::<UpdateScript>().unwrap().0.clone();
    let Some(script) = world
        .get_resource::<Assets<Script>>()
        .expect("Script asset not loaded")
        .get(&script)
    else {
        return;
    };

    let script = script.code.clone();
    let script = format!("{script};update(app, model);");

    let mut ctx = world.remove_non_send_resource::<JsContext>().unwrap();
    {
        let type_registry = world.get_resource::<AppTypeRegistry>().unwrap().read();
        let js_model = reflect_to_js_model(model, &type_registry, &mut ctx);
        ctx.global_object()
            .set(JsString::from("model"), js_model, true, &mut ctx)
            .expect("Unable to set model in global object");

        let prototype = ctx
            .realm()
            .get_class::<JsApp>()
            .expect("Unable to get app class");
        let js_app = JsObject::from_proto_and_data(Some(prototype.prototype()), js_app);
        ctx.global_object()
            .set(JsString::from("app"), JsValue::from(js_app), true, &mut ctx)
            .expect("Unable to set app in global object");
        let result = ctx.eval(Source::from_bytes(&script));
        if let Ok(result) = result {
            match write_js_model(model.reflect_mut(), result, &type_registry, &mut ctx) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error running update script: {:?}", e);
                }
            };
        } else {
            error!("Error running update script: {:?}", result);
        };
    }
    world.insert_non_send_resource(ctx);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_type() {
        let mut type_registry = reflect::TypeRegistry::default();
        register_type!(u8, &mut type_registry);
        register_type!(u16, &mut type_registry);
        register_type!(u32, &mut type_registry);
        register_type!(u64, &mut type_registry);
        register_type!(i8, &mut type_registry);
        register_type!(i16, &mut type_registry);
        register_type!(i32, &mut type_registry);
        register_type!(i64, &mut type_registry);
        register_type!(bool, &mut type_registry);
    }
}
