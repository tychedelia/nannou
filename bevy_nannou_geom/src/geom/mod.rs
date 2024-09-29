use crate::geom::properties::material::{Component, SetMaterial, World};
use crate::geom::properties::mesh::SetMesh;
use crate::geom::properties::transform::SetTransform;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::view::{NoFrustumCulling, RenderLayers};
use bevy_nannou_draw::render::ShaderModel;
use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use crate::geom::mesh::cube::Cube;

pub mod mesh;
pub mod properties;

#[derive(Clone)]
pub struct Geom<'w, SM> {
    window: Entity,
    resource_world: Rc<RefCell<UnsafeWorldCell<'w>>>,
    component_world: Rc<RefCell<UnsafeWorldCell<'w>>>,
    _shader_model: PhantomData<SM>,
}

impl<'w, SM> Geom<'w, SM>
where
    SM: ShaderModel + Material + Default,
{
    pub fn new(
        window: Entity,
        resource_world: Rc<RefCell<UnsafeWorldCell<'w>>>,
        component_world: Rc<RefCell<UnsafeWorldCell<'w>>>,
    ) -> Self {
        Self {
            window,
            resource_world,
            component_world,
            _shader_model: PhantomData,
        }
    }

    pub fn get<T>(&self, entity: Entity) -> Geometry<'w, 'w, T, SM>
    where
        T: Into<Mesh> + Component + Clone,
    {
        Geometry {
            entity,
            geom: GeomRef::Owned(self.clone()),
            _ty: PhantomData,
        }
    }

    pub(crate) fn resource_world(&self) -> std::cell::Ref<'_, World> {
        let world = self.resource_world.borrow();
        std::cell::Ref::map(world, |world| unsafe { world.world() })
    }

    pub(crate) fn resource_world_mut(&self) -> RefMut<'_, World> {
        let world = self.resource_world.borrow_mut();
        RefMut::map(world, |world| unsafe { world.world_mut() })
    }

    pub(crate) fn component_world(&self) -> std::cell::Ref<'_, World> {
        let world = self.component_world.borrow();
        std::cell::Ref::map(world, |world| unsafe { world.world() })
    }

    pub(crate) fn component_world_mut(&self) -> RefMut<'_, World> {
        let world = self.component_world.borrow_mut();
        RefMut::map(world, |world| unsafe { world.world_mut() })
    }

    pub fn cuboid(&self) -> Entity {
        self.a(Cube::default()).entity
    }

    fn a<'a, T>(&'a self, primitive: T) -> Geometry<'a, 'w, T, SM>
    where
        T: Into<Mesh> + Component + Clone,
    {
        let mut resource_world = self.resource_world_mut();
        let material = SM::default();
        let mut materials = resource_world.resource_mut::<Assets<SM>>();
        let material = materials.add(material);
        let mut meshes = resource_world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(primitive.clone().into());
        let render_layer = {
            let component_world = self.component_world();
            component_world
                .get::<RenderLayers>(self.window)
                .unwrap()
                .clone()
        };

        let entity = self
            .component_world_mut()
            .spawn((
                MaterialMeshBundle {
                    material,
                    mesh,
                    ..Default::default()
                },
                NoFrustumCulling,
                primitive,
                render_layer,
            ))
            .id();

        Geometry {
            entity,
            geom: GeomRef::Borrowed(self),
            _ty: PhantomData,
        }
    }
}

pub enum GeomRef<'a, 'w, SM> {
    Borrowed(&'a Geom<'w, SM>),
    Owned(Geom<'w, SM>),
}

impl<'a, 'w, SM> Deref for GeomRef<'a, 'w, SM> {
    type Target = Geom<'w, SM>;

    fn deref(&self) -> &Self::Target {
        match self {
            GeomRef::Borrowed(geom) => *geom,
            GeomRef::Owned(geom) => geom,
        }
    }
}

pub struct Geometry<'a, 'w, T, SM>
where
    SM: ShaderModel + Material + Default,
    T: Into<Mesh> + Component + Clone,
{
    entity: Entity,
    geom: GeomRef<'a, 'w, SM>,
    _ty: PhantomData<T>,
}

impl<'a, 'w, T, SM> Drop for Geometry<'a, 'w, T, SM>
where
    SM: ShaderModel + Material + Default,
    T: Into<Mesh> + Component + Clone,
{
    fn drop(&mut self) {
        if self.primitive_changed() {
            self.sync_primitive();
        }
    }
}

impl<'a, 'w, T, SM> Geometry<'a, 'w, T, SM>
where
    SM: ShaderModel + Material + Default,
    T: Into<Mesh> + Component + Clone,
{
    fn primitive(&self) -> RefMut<'_, T> {
        let component_world = self.geom.component_world_mut();
        RefMut::map(component_world, |world| {
            world.get_mut::<T>(self.entity).unwrap().into_inner()
        })
    }

    fn sync_primitive(&mut self) {
        let prim = self.primitive().clone();
        let mut mesh = self.mesh();
        *mesh = prim.into();
    }

    fn primitive_changed(&mut self) -> bool {
        let mut component_world = self.geom.component_world_mut();
        component_world.get_mut::<T>(self.entity).unwrap().is_changed()
    }
}

impl<'a, 'w, T, SM> SetTransform for Geometry<'a, 'w, T, SM>
where
    SM: ShaderModel + Material + Default,
    T: Into<Mesh> + Component + Clone,
{
    fn transform(&mut self) -> RefMut<'_, Transform> {
        let component_world = self.geom.component_world_mut();
        RefMut::map(component_world, |world| {
            world
                .get_mut::<Transform>(self.entity)
                .unwrap()
                .into_inner()
        })
    }
}

impl<'a, 'w, T, SM> SetMaterial<SM> for Geometry<'a, 'w, T, SM>
where
    SM: ShaderModel + Material + Default,
    T: Into<Mesh> + Component + Clone,
{
    fn material(&mut self) -> RefMut<'_, SM> {
        let resource_world = self.geom.resource_world_mut();
        let component_world = self.geom.component_world();
        let handle = component_world.get::<Handle<SM>>(self.entity).unwrap();
        let materials = RefMut::map(resource_world, |world| {
            world.resource_mut::<Assets<SM>>().into_inner()
        });
        RefMut::map(materials, |materials| materials.get_mut(handle).unwrap())
    }
}

impl<'a, 'w, T, SM> SetMesh for Geometry<'a, 'w, T, SM>
where
    SM: ShaderModel + Material + Default,
    T: Into<Mesh> + Component + Clone,
{
    fn mesh(&mut self) -> RefMut<'_, Mesh> {
        let resource_world = self.geom.resource_world_mut();
        let component_world = self.geom.component_world();
        let handle = component_world.get::<Handle<Mesh>>(self.entity).unwrap();
        let meshes = RefMut::map(resource_world, |world| {
            world.resource_mut::<Assets<Mesh>>().into_inner()
        });
        RefMut::map(meshes, |meshes| meshes.get_mut(handle).unwrap())
    }
}

impl<'a, 'w, T> Geometry<'a, 'w, T, StandardMaterial>
where
    T: Into<Mesh> + Component + Clone,
{
    pub fn base_color(mut self, color: Color) -> Self {
        self.material().base_color = color;
        self
    }

    pub fn base_color_texture(mut self, texture: Handle<Image>) -> Self {
        self.material().base_color_texture = Some(texture);
        self
    }

    pub fn emissive(mut self, color: Color) -> Self {
        self.material().emissive = color.to_linear();
        self
    }

    pub fn perceptual_roughness(mut self, roughness: f32) -> Self {
        self.material().perceptual_roughness = roughness;
        self
    }

    pub fn metallic(mut self, metallic: f32) -> Self {
        self.material().metallic = metallic;
        self
    }

    pub fn reflectance(mut self, reflectance: f32) -> Self {
        self.material().reflectance = reflectance;
        self
    }

    pub fn diffuse_transmission(mut self, transmission: f32) -> Self {
        self.material().diffuse_transmission = transmission;
        self
    }

    pub fn specular_transmission(mut self, transmission: f32) -> Self {
        self.material().specular_transmission = transmission;
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.material().thickness = thickness;
        self
    }

    pub fn ior(mut self, ior: f32) -> Self {
        self.material().ior = ior;
        self
    }

    pub fn attenuation_distance(mut self, distance: f32) -> Self {
        self.material().attenuation_distance = distance;
        self
    }

    pub fn attenuation_color(mut self, color: Color) -> Self {
        self.material().attenuation_color = color;
        self
    }
}
