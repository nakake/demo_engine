use crate::resources::{primitives::Primitive, vertex::ColorVertex};

pub struct Triangle;

impl Primitive for Triangle {
    type Vertex = ColorVertex;

    fn create_vertices() -> Vec<Self::Vertex> {
        vec![
            ColorVertex {
                position: [0.0, 0.5, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            ColorVertex {
                position: [-0.5, -0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            ColorVertex {
                position: [0.5, -0.5, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ]
    }

    fn create_indices() -> Option<Vec<u16>> {
        None
    }
}
