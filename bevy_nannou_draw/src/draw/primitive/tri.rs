use bevy::prelude::*;
use lyon::tessellation::StrokeOptions;

use nannou_core::geom;

use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use crate::render::ShaderModel;

/// Properties related to drawing a **Tri**.
#[derive(Clone, Debug)]
pub struct Tri {
    tri: geom::Tri<Vec2>,
    dimensions: dimension::Properties,
    polygon: PolygonInit,
}

/// The drawing context for a `Tri`.
pub type DrawingTri<'a, SM> = Drawing<'a, Tri, SM>;

// Tri-specific methods.

impl Tri {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.stroke_color(color)
    }

    /// Use the given three points as the vertices (corners) of the triangle.
    pub fn points<P>(mut self, a: P, b: P, c: P) -> Self
    where
        P: Into<Vec2>,
    {
        let a = a.into();
        let b = b.into();
        let c = c.into();
        self.tri = geom::Tri([a, b, c]);
        self
    }
}

// Drawing methods.

impl<'a, SM> DrawingTri<'a, SM>
where
    SM: ShaderModel + Default,
{
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Use the given points as the vertices (corners) of the triangle.
    pub fn points<P>(self, a: P, b: P, c: P) -> Self
    where
        P: Into<Vec2>,
    {
        self.map_ty(|ty| ty.points(a, b, c))
    }
}

// Trait implementations.

impl draw::render::RenderPrimitive for Tri {
    fn render_primitive(self, ctxt: draw::render::RenderContext, mesh: &mut Mesh) {
        let Tri {
            mut tri,
            dimensions,
            polygon,
        } = self;
        let (maybe_x, maybe_y, _maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        // If dimensions were specified, scale the points to those dimensions.
        if maybe_x.is_some() || maybe_y.is_some() {
            let cuboid = tri.bounding_rect();
            let centroid = tri.centroid();
            let x_scale = maybe_x.map(|x| x / cuboid.w()).unwrap_or(1.0);
            let y_scale = maybe_y.map(|y| y / cuboid.h()).unwrap_or(1.0);
            let scale = Vec2::new(x_scale, y_scale);
            let (a, b, c) = tri.into();
            let translate = |v: Vec2| centroid + ((v - centroid) * scale);
            let new_a = translate(a);
            let new_b = translate(b);
            let new_c = translate(c);
            tri = geom::Tri([new_a, new_b, new_c]);
        }

        let tex_coords = [
            Vec2::new(0.0, 0.0), // Vertex A
            Vec2::new(1.0, 0.0), // Vertex B
            Vec2::new(0.5, 1.0), // Vertex C
        ];

        let points = tri.vertices().zip(tex_coords.iter().copied());

        polygon::render_points_themed(
            polygon.opts,
            true,
            points,
            ctxt,
            &draw::theme::Primitive::Tri,
            mesh,
        );
    }
}

impl From<geom::Tri<Vec2>> for Tri {
    fn from(tri: geom::Tri<Vec2>) -> Self {
        let dimensions = <_>::default();
        let polygon = <_>::default();
        Tri {
            tri,
            dimensions,
            polygon,
        }
    }
}

impl Default for Tri {
    fn default() -> Self {
        // Create a triangle pointing towards 0.0 radians.
        let fifty = 50.0;
        let thirty_three = 33.0;
        let a = Vec2::new(-fifty, thirty_three);
        let b = Vec2::new(fifty, 0.0);
        let c = Vec2::new(-fifty, -thirty_three);
        Tri::from(geom::Tri([a, b, c]))
    }
}

impl SetOrientation for Tri {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl SetPosition for Tri {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.polygon)
    }
}

impl SetDimensions for Tri {
    fn properties(&mut self) -> &mut dimension::Properties {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl SetColor for Tri {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.polygon)
    }
}

impl SetStroke for Tri {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl SetPolygon for Tri {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversions.

impl From<Tri> for Primitive {
    fn from(prim: Tri) -> Self {
        Primitive::Tri(prim)
    }
}

impl Into<Option<Tri>> for Primitive {
    fn into(self) -> Option<Tri> {
        match self {
            Primitive::Tri(prim) => Some(prim),
            _ => None,
        }
    }
}
