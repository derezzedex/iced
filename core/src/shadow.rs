use crate::{Color, Vector};

#[cfg(feature = "inspector")]
use crate::widget::operation::inspectable;

/// A shadow.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(
    feature = "inspector",
    derive(inspectable::Serialize, inspectable::Deserialize)
)]
pub struct Shadow {
    /// The color of the shadow.
    pub color: Color,

    /// The offset of the shadow.
    pub offset: Vector,

    /// The blur radius of the shadow.
    pub blur_radius: f32,
}
