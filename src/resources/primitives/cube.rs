use crate::resources::{primitives::Primitive, vertex::ColorVertex};

pub struct Cube;

impl Primitive for Cube {
    type Vertex = ColorVertex;

    fn create_vertices() -> Vec<Self::Vertex> {
        let s = 0.5f32;
        vec![
            // 前面 (Z+) - 赤系
            ColorVertex {
                position: [-s, -s, s],
                color: [1.0, 0.0, 0.0],
            }, // 0
            ColorVertex {
                position: [s, -s, s],
                color: [1.0, 0.2, 0.0],
            }, // 1
            ColorVertex {
                position: [s, s, s],
                color: [1.0, 0.4, 0.0],
            }, // 2
            ColorVertex {
                position: [-s, s, s],
                color: [1.0, 0.6, 0.0],
            }, // 3
            // 後面 (Z-) - 青系
            ColorVertex {
                position: [s, -s, -s],
                color: [0.0, 0.0, 1.0],
            }, // 4
            ColorVertex {
                position: [-s, -s, -s],
                color: [0.0, 0.2, 1.0],
            }, // 5
            ColorVertex {
                position: [-s, s, -s],
                color: [0.0, 0.4, 1.0],
            }, // 6
            ColorVertex {
                position: [s, s, -s],
                color: [0.0, 0.6, 1.0],
            }, // 7
            // 左面 (X-) - 緑系
            ColorVertex {
                position: [-s, -s, -s],
                color: [0.0, 1.0, 0.0],
            }, // 8
            ColorVertex {
                position: [-s, -s, s],
                color: [0.2, 1.0, 0.0],
            }, // 9
            ColorVertex {
                position: [-s, s, s],
                color: [0.4, 1.0, 0.0],
            }, // 10
            ColorVertex {
                position: [-s, s, -s],
                color: [0.6, 1.0, 0.0],
            }, // 11
            // 右面 (X+) - マゼンタ系
            ColorVertex {
                position: [s, -s, s],
                color: [1.0, 0.0, 1.0],
            }, // 12
            ColorVertex {
                position: [s, -s, -s],
                color: [1.0, 0.2, 1.0],
            }, // 13
            ColorVertex {
                position: [s, s, -s],
                color: [1.0, 0.4, 1.0],
            }, // 14
            ColorVertex {
                position: [s, s, s],
                color: [1.0, 0.6, 1.0],
            }, // 15
            // 上面 (Y+) - シアン系
            ColorVertex {
                position: [-s, s, s],
                color: [0.0, 1.0, 1.0],
            }, // 16
            ColorVertex {
                position: [s, s, s],
                color: [0.2, 1.0, 1.0],
            }, // 17
            ColorVertex {
                position: [s, s, -s],
                color: [0.4, 1.0, 1.0],
            }, // 18
            ColorVertex {
                position: [-s, s, -s],
                color: [0.6, 1.0, 1.0],
            }, // 19
            // 下面 (Y-) - 黄系
            ColorVertex {
                position: [-s, -s, -s],
                color: [1.0, 1.0, 0.0],
            }, // 20
            ColorVertex {
                position: [s, -s, -s],
                color: [1.0, 1.0, 0.2],
            }, // 21
            ColorVertex {
                position: [s, -s, s],
                color: [1.0, 1.0, 0.4],
            }, // 22
            ColorVertex {
                position: [-s, -s, s],
                color: [1.0, 1.0, 0.6],
            }, // 23
        ]
    }

    fn create_indices() -> Option<Vec<u16>> {
        Some(vec![
            // 前面 (Z+)
            0, 1, 2, 2, 3, 0, // 後面 (Z-)
            4, 5, 6, 6, 7, 4, // 左面 (X-)
            8, 9, 10, 10, 11, 8, // 右面 (X+)
            12, 13, 14, 14, 15, 12, // 上面 (Y+)
            16, 17, 18, 18, 19, 16, // 下面 (Y-)
            20, 21, 22, 22, 23, 20,
        ])
    }
}
