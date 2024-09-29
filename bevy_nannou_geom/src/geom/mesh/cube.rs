use std::ops::DerefMut;
use bevy::prelude::*;
use bevy_nannou_draw::render::ShaderModel;
use crate::geom::Geometry;
use crate::geom::properties::mesh::SetMesh;

pub type CubeGeometry<'a, 'w, SM: ShaderModel> = Geometry<'a, 'w, Cube, SM>;

#[derive(Component, Deref, DerefMut, Default, Clone)]
pub struct Cube(pub Cuboid);

impl From<Cube> for Mesh {
    fn from(value: Cube) -> Self {
        value.0.into()
    }
}

impl <'a, 'w, SM> CubeGeometry<'a, 'w, SM>
    where SM: ShaderModel + Material + Default
{
    pub fn x_length(mut self, length: f32) -> Self {
        self.primitive().half_size.x = length / 2.0;
        self
    }

    pub fn y_length(mut self, length: f32) -> Self {
        self.primitive().half_size.y = length / 2.0;
        self
    }

    pub fn z_length(mut self, length: f32) -> Self {
        self.primitive().half_size.z = length / 2.0;
        self
    }

    pub fn length(mut self, length: f32) -> Self {
        self.primitive().half_size = Vec3::splat(length / 2.0);
        self
    }

    pub fn dimensions(mut self, dimensions: Vec3) -> Self {
        self.primitive().half_size = dimensions / 2.0;
        self
    }

    pub fn corners(mut self, point1: Vec3, point2: Vec3) -> Self {
        self.primitive().half_size = (point2 - point1).abs() / 2.0;
        self
    }
}

