use std::cell::RefMut;
pub use bevy::prelude::*;

pub trait SetMaterial<M: Material>: Sized {
    /// Provide a mutable reference to the **Material** for updating.
    fn material(&mut self) -> RefMut<'_, M>;
}