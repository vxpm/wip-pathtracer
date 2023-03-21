use crate::{material::Material, shape::Shape};
use std::sync::Arc;

/// An object in space. It's basically a [`Shape`] with an
/// associated [`Material`].
pub struct Object {
    pub shape: Shape,
    pub material: Arc<dyn Material>,
}
