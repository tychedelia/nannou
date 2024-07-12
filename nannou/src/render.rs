use crate::prelude::bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::ecs::query::QueryItem;
pub use bevy::prelude::*;
use bevy::render::render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode};
use bevy::render::renderer::RenderContext;
use bevy::render::view::ViewTarget;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub struct RenderApp<'w> {
    current_view: Option<Entity>,
    world: &'w World,
}

impl<'w> RenderApp<'w> {
    pub fn world(&self) -> &'w World {
        self.world
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct NannouRenderNodeLabel;

#[derive(Default)]
struct NannouRenderNode<M>(std::marker::PhantomData<M>);

impl<M> ViewNode for NannouRenderNode<M>
where
    M: Send + Sync + 'static,
{
    type ViewQuery = (&'static ViewTarget,);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        Ok(())
    }
}

macro_rules! define_view_node {
    ($node_name:ident, $label_name:ident) => {
        #[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
        struct $label_name;

        #[derive(Default)]
        struct $node_name<M>;

        impl bevy::render::render_graph::ViewNode for $node_name<M>
        where
            M: Send + Sync + 'static,
        {
            type ViewQuery = (&'static ViewTarget,);

            fn run(
                &self,
                _graph: &mut RenderGraphContext,
                _render_context: &mut RenderContext,
                (view_target): QueryItem<Self::ViewQuery>,
                world: &World,
            ) -> Result<(), NodeRunError> {
                Ok(())
            }
        }
    };
}
