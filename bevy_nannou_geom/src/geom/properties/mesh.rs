use std::cell::RefMut;
pub use bevy::prelude::*;

pub trait SetMesh: Sized {
    /// Provide a mutable reference to the [`Mesh`] for updating.
    fn mesh(&mut self) -> RefMut<'_, Mesh>;
}