use std::f32::consts::PI;

use crate::resources::{primitives::Primitive, vertex::ColorVertex};

pub struct Sphere;

impl Sphere {
    const SECTORS: i32 = 32;
    const STACKS: i32 = 32;
}

impl Primitive for Sphere {
    type Vertex = ColorVertex;

    fn create_vertices() -> Vec<Self::Vertex> {
        let mut vertices = Vec::new();
        let redius = 0.5f32;

        for i in 0..=Self::STACKS {
            let stack_angle = PI / 2.0 - (i as f32) * PI / Self::STACKS as f32;

            let xy = redius * stack_angle.cos();
            let z = redius * stack_angle.sin();

            for j in 0..=Self::SECTORS {
                let sector_angle = (j as f32) * 2.0 * PI / Self::SECTORS as f32;

                let x = xy * sector_angle.cos();
                let y = xy * sector_angle.sin();

                vertices.push(Self::Vertex {
                    position: [x, y, z],
                    color: [(x + 0.5), (y + 0.5), (z + 0.5)],
                });
            }
        }
        vertices
    }

    fn create_indices() -> Option<Vec<u16>> {
        let mut indecies = Vec::new();

        for i in 0..Self::STACKS {
            let k1 = i * (Self::SECTORS + 1);
            let k2 = k1 + Self::SECTORS + 1;
            for j in 0..Self::SECTORS {
                if i != 0 {
                    indecies.push((k1 + j) as u16);
                    indecies.push((k2 + j) as u16);
                    indecies.push((k1 + j + 1) as u16);
                }

                if i != Self::STACKS - 1 {
                    indecies.push((k1 + j + 1) as u16);
                    indecies.push((k2 + j) as u16);
                    indecies.push((k2 + j + 1) as u16);
                }
            }
        }

        Some(indecies)
    }
}
