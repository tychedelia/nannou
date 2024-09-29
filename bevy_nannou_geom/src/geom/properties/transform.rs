use bevy::prelude::*;
use bevy_nannou_draw::draw::properties::spatial::orientation::Properties;
use std::cell::RefMut;

pub trait SetTransform: Sized {
    /// Provide a mutable reference to the **Transform** for updating.
    fn transform(&mut self) -> RefMut<Transform>;

    /// Specify the translation of the transform.
    /// Build with the given **Absolute** **Position** along the *x* axis.
    fn x(mut self, x: f32) -> Self {
        self.transform().translation.x = x;
        self
    }

    /// Build with the given **Absolute** **Position** along the *y* axis.
    fn y(mut self, y: f32) -> Self {
        self.transform().translation.y = y;
        self
    }

    /// Build with the given **Absolute** **Position** along the *z* axis.
    fn z(mut self, z: f32) -> Self {
        self.transform().translation.z = z;
        self
    }

    /// Set the **Position** with some two-dimensional point.
    fn xy(self, p: Vec2) -> Self {
        self.x(p.x).y(p.y)
    }

    /// Set the **Position** with some three-dimensional point.
    fn xyz(self, p: Vec3) -> Self {
        self.x(p.x).y(p.y).z(p.z)
    }

    /// Set the **Position** with *x* *y* coordinates.
    fn x_y(self, x: f32, y: f32) -> Self {
        self.xy([x, y].into())
    }

    /// Set the **Position** with *x* *y* *z* coordinates.
    fn x_y_z(self, x: f32, y: f32, z: f32) -> Self {
        self.xyz([x, y, z].into())
    }

    /// Specify the orientation around the *x* axis as an absolute value in radians.
    fn x_radians(mut self, x: f32) -> Self {
        self.transform().rotation = Quat::from_rotation_x(x);
        self
    }

    /// Specify the orientation around the *y* axis as an absolute value in radians.
    fn y_radians(mut self, y: f32) -> Self {
        self.transform().rotation = Quat::from_rotation_y(y);
        self
    }

    /// Specify the orientation around the *z* axis as an absolute value in radians.
    fn z_radians(mut self, z: f32) -> Self {
        self.transform().rotation = Quat::from_rotation_z(z);
        self
    }

    /// Specify the orientation around the *x* axis as an absolute value in degrees.
    fn x_degrees(self, x: f32) -> Self {
        self.x_radians(x.to_radians())
    }

    /// Specify the orientation around the *y* axis as an absolute value in degrees.
    fn y_degrees(self, y: f32) -> Self {
        self.y_radians(y.to_radians())
    }

    /// Specify the orientation around the *z* axis as an absolute value in degrees.
    fn z_degrees(self, z: f32) -> Self {
        self.z_radians(z.to_radians())
    }

    /// Specify the orientation around the *x* axis as a number of turns around the axis.
    fn x_turns(self, x: f32) -> Self {
        self.x_radians(x * std::f32::consts::TAU)
    }

    /// Specify the orientation around the *y* axis as a number of turns around the axis.
    fn y_turns(self, y: f32) -> Self {
        self.y_radians(y * std::f32::consts::TAU)
    }

    /// Specify the orientation around the *z* axis as a number of turns around the axis.
    fn z_turns(self, z: f32) -> Self {
        self.z_radians(z * std::f32::consts::TAU)
    }

    /// Specify the orientation along each axis with the given **Vector** of radians.
    ///
    /// This has the same affect as calling `self.x_radians(v.x).y_radians(v.y).z_radians(v.z)`.
    fn radians(self, v: Vec3) -> Self {
        self.x_radians(v.x).y_radians(v.y).z_radians(v.z)
    }

    /// Specify the orientation along each axis with the given **Vector** of degrees.
    ///
    /// This has the same affect as calling `self.x_degrees(v.x).y_degrees(v.y).z_degrees(v.z)`.
    fn degrees(self, v: Vec3) -> Self {
        self.x_degrees(v.x).y_degrees(v.y).z_degrees(v.z)
    }

    /// Specify the orientation along each axis with the given **Vector** of "turns".
    ///
    /// This has the same affect as calling `self.x_turns(v.x).y_turns(v.y).z_turns(v.z)`.
    fn turns(self, v: Vec3) -> Self {
        self.x_turns(v.x).y_turns(v.y).z_turns(v.z)
    }

    /// Specify the orientation with the given euler orientation in radians.
    fn euler(self, e: Vec3) -> Self {
        self.radians(e)
    }

    /// Specify the orientation with the given **Quaternion**.
    fn quaternion(mut self, q: Quat) -> Self {
        self.transform().rotation = q;
        self
    }

    // Higher level methods.

    /// Specify the "pitch" of the orientation in radians.
    ///
    /// This has the same effect as calling `x_radians`.
    fn pitch(self, pitch: f32) -> Self {
        self.x_radians(pitch)
    }

    /// Specify the "yaw" of the orientation in radians.
    ///
    /// This has the same effect as calling `y_radians`.
    fn yaw(self, yaw: f32) -> Self {
        self.y_radians(yaw)
    }

    /// Specify the "roll" of the orientation in radians.
    ///
    /// This has the same effect as calling `z_radians`.
    fn roll(self, roll: f32) -> Self {
        self.z_radians(roll)
    }

    /// Assuming we're looking at a 2D plane, positive values cause a clockwise rotation where the
    /// given value is specified in radians.
    ///
    /// This is equivalent to calling the `z_radians` or `roll` methods.
    fn rotate(self, radians: f32) -> Self {
        self.z_radians(radians)
    }
}
