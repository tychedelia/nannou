//! A shader that renders a mesh multiple times in one draw call.

use crate::render::RenderShaderModelInstances;
use crate::{
    draw::{drawing::Drawing, primitive::Primitive, Draw, DrawCommand},
    render::{
        queue_shader_model, DrawShaderModel, PreparedShaderModel, ShaderModel, ShaderModelMesh,
    },
};
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::system::{lifetimeless::*, SystemParamItem},
    pbr::{RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_instances::ExtractedInstances,
        mesh::{allocator::MeshAllocator, RenderMesh, RenderMeshBufferInfo},
        render_asset::{prepare_assets, RenderAsset, RenderAssets},
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        Render, RenderApp, RenderSet,
    },
};
use rayon::prelude::*;
use std::{hash::Hash, marker::PhantomData};

pub struct Indirect<'a, SM>
where
    SM: ShaderModel + Default,
{
    draw: &'a Draw<SM>,
    primitive_index: Option<usize>,
    indirect_buffer: Option<Handle<ShaderStorageBuffer>>,
    vertex_buffer: Option<Handle<ShaderStorageBuffer>>,
}

impl<'a, SM> Drop for Indirect<'a, SM>
where
    SM: ShaderModel + Default,
{
    fn drop(&mut self) {
        if let Some((index, ssbo)) = self.primitive_index.take().zip(self.indirect_buffer.take()) {
            let vertex_buffer = self.vertex_buffer.take();
            self.insert_indirect_draw_command(index, ssbo, vertex_buffer);
        }
    }
}

pub fn new<SM>(draw: &Draw<SM>) -> Indirect<SM>
where
    SM: ShaderModel + Default,
{
    Indirect {
        draw,
        primitive_index: None,
        indirect_buffer: None,
        vertex_buffer: None,
    }
}

impl<'a, SM> Indirect<'a, SM>
where
    SM: ShaderModel + Default,
{
    pub fn primitive<T>(mut self, drawing: Drawing<T, SM>) -> Indirect<'a, SM>
    where
        T: Into<Primitive>,
    {
        self.draw
            .state
            .write()
            .unwrap()
            .ignored_drawings
            .insert(drawing.index);
        self.primitive_index = Some(drawing.index);
        self
    }

    pub fn buffer(mut self, ssbo: Handle<ShaderStorageBuffer>) -> Indirect<'a, SM> {
        self.indirect_buffer = Some(ssbo);
        self
    }

    fn insert_indirect_draw_command(
        &self,
        index: usize,
        indirect_buffer: Handle<ShaderStorageBuffer>,
        vertex_buffer: Option<Handle<ShaderStorageBuffer>>,
    ) {
        let mut state = self.draw.state.write().unwrap();
        let primitive = state.drawing.remove(&index).unwrap();
        state.draw_commands.push(Some(DrawCommand::Indirect(
            primitive,
            indirect_buffer,
            vertex_buffer,
        )));
    }
}

#[derive(Component, ExtractComponent, Clone)]
pub struct IndirectMesh;

#[derive(Component, ExtractComponent, Clone)]
pub struct IndirectBuffer(pub Handle<ShaderStorageBuffer>);

#[derive(Component, ExtractComponent, Clone)]
pub struct IndirectVertexBuffer(pub Option<Handle<ShaderStorageBuffer>>);

pub struct IndirectShaderModelPlugin<SM>(PhantomData<SM>);

impl<SM> Default for IndirectShaderModelPlugin<SM>
where
    SM: Default,
{
    fn default() -> Self {
        IndirectShaderModelPlugin(PhantomData)
    }
}

impl<SM> Plugin for IndirectShaderModelPlugin<SM>
where
    SM: ShaderModel,
    SM::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawIndirectShaderModel<SM>>()
            .add_systems(
                Render,
                queue_shader_model::<SM, With<IndirectMesh>, DrawIndirectShaderModel<SM>>
                    .after(prepare_assets::<PreparedShaderModel<SM>>)
                    .in_set(RenderSet::QueueMeshes),
            );
    }
}

type DrawIndirectShaderModel<SM> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetShaderModelBindGroup<SM, 2>,
    DrawMeshIndirect,
);

struct SetShaderModelBindGroup<SM: ShaderModel, const I: usize>(PhantomData<SM>);
impl<P: PhaseItem, SM: ShaderModel, const I: usize> RenderCommand<P>
    for SetShaderModelBindGroup<SM, I>
{
    type Param = (
        SRes<RenderAssets<PreparedShaderModel<SM>>>,
        SRes<RenderShaderModelInstances<SM>>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        (models, instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let models = models.into_inner();
        let instances = instances.into_inner();

        let Some(asset_id) = instances.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some(model) = models.get(*asset_id) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &model.bind_group, &[]);
        RenderCommandResult::Success
    }
}

struct DrawMeshIndirect;
impl<P: PhaseItem> RenderCommand<P> for DrawMeshIndirect {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
        SRes<RenderAssets<GpuShaderStorageBuffer>>,
    );
    type ViewQuery = ();
    type ItemQuery = (Read<IndirectBuffer>, Read<IndirectVertexBuffer>);

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        item_q: Option<(&'w IndirectBuffer, &'w IndirectVertexBuffer)>,
        (meshes, render_mesh_instances, mesh_allocator, ssbos): SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();
        let meshes = meshes.into_inner();
        let ssbos = ssbos.into_inner();

        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.main_entity())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(gpu_mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Skip;
        };
        let Some((indirect_buffer, vertex_buffer)) = item_q else {
            return RenderCommandResult::Skip;
        };
        let Some(indirect_buffer) = ssbos.get(&indirect_buffer.0) else {
            return RenderCommandResult::Skip;
        };

        let vertex_buffer = match &vertex_buffer.0 {
            Some(vertex_buffer) => match ssbos.get(vertex_buffer) {
                Some(vertex_buffer) => vertex_buffer.buffer.slice(..),
                None => return RenderCommandResult::Skip,
            },
            None => match mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id) {
                Some(vertex_buffer_slice) => vertex_buffer_slice.buffer.slice(..),
                None => return RenderCommandResult::Skip,
            },
        };

        pass.set_vertex_buffer(0, vertex_buffer);

        match &gpu_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed { index_format, .. } => {
                let Some(index_buffer_slice) =
                    mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
                else {
                    return RenderCommandResult::Skip;
                };

                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), 0, *index_format);
                pass.draw_indexed_indirect(&indirect_buffer.buffer, 0);
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw_indirect(&indirect_buffer.buffer, 0);
            }
        }
        RenderCommandResult::Success
    }
}
