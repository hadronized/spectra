pub use cgmath::{Matrix4, Vector2, Vector3, Vector4, Quaternion};
pub use num_traits::{One, Zero};

pub use scale::*;

// some useful aliases
pub type V2<T> = Vector2<T>;
pub type V3<T> = Vector3<T>;
pub type V4<T> = Vector4<T>;
pub type M44<T> = Matrix4<T>;
pub type Quat<T> = Quaternion<T>;
