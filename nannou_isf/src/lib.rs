use crate::asset::{Isf, IsfAssetPlugin, IsfHandle};
use crate::inputs::{IsfInputValue, IsfInputs};
use crate::render::{IsfRenderPlugin, IsfRenderTargets};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiPlugin, EguiSchedule};
use bevy_inspector_egui::bevy_inspector::ui_for_resource;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

mod asset;
mod inputs;
mod render;

pub use crate::asset::{IsfAssetLoaderError, wgsl_from_isf};

pub mod prelude {
    pub use crate::asset::Isf;
    pub use crate::inputs::{IsfInputValue, IsfInputs};
}

pub struct NannouIsfPlugin;

impl Plugin for NannouIsfPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }
        app.add_systems(First, asset_event_handler)
            .add_systems(Update, isf_inputs_ui)
            .add_plugins((
                IsfRenderPlugin,
                IsfAssetPlugin,
                ResourceInspectorPlugin::<IsfInputs>::default(),
                ExtractComponentPlugin::<IsfHandle>::default(),
                ExtractResourcePlugin::<IsfInputs>::default(),
                ExtractResourcePlugin::<IsfRenderTargets>::default(),
            ))
            .init_resource::<IsfRenderTargets>()
            .init_resource::<IsfInputs>()
            .register_type::<IsfInputs>()
            .register_type::<IsfInputValue>()
            .register_asset_reflect::<Image>();
    }
}

fn asset_event_handler(
    mut commands: Commands,
    mut ev_asset: MessageReader<AssetEvent<Isf>>,
    mut isf_inputs: ResMut<IsfInputs>,
    cameras: Query<Entity, With<Camera>>,
    mut assets: ResMut<Assets<Isf>>,
) {
    for ev in ev_asset.read() {
        match ev {
            AssetEvent::Added { .. } | AssetEvent::Modified { .. } | AssetEvent::Removed { .. } => {
                // TODO: handle these events
            }
            AssetEvent::LoadedWithDependencies { id } => {
                let isf = assets.get(*id).unwrap();
                *isf_inputs = IsfInputs::from_isf(&isf.isf);
                let handle = assets.get_strong_handle(*id).unwrap();
                for camera in cameras.iter() {
                    commands.entity(camera).insert(IsfHandle(handle.clone()));
                }
            }
            _ => {}
        }
    }
}

/// Draws the `IsfInputs` inspector on the egui context of the camera rendering the ISF.
fn isf_inputs_ui(world: &mut World) {
    let Some(mut egui_context) = world
        .query_filtered::<&EguiContext, (With<IsfHandle>, Without<EguiSchedule>)>()
        .iter(world)
        .next()
        .cloned()
    else {
        return;
    };
    egui::Window::new("IsfInputs")
        .default_size((0.0, 0.0))
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui_for_resource::<IsfInputs>(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}
