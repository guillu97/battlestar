use crate::components::{ MapBoundary, Player, Ship, Thruster, ThrusterOwner, Velocity};
use crate::constants::WORLD_LIMIT;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_asset::RenderAssetUsages;

// ── Ship ───────────────────────────────────────────────────────────────

impl Ship {
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        position: Vec3,
    ) -> Entity {
        let ship_entity = commands
            .spawn((
                Mesh2d(meshes.add(build_triangle_mesh(25.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.0, 0.545, 0.545)))),
                Transform::from_translation(position),
                Ship,
                Player,
                Velocity::default(),
            ))
            .id();

        let thruster_entity = commands
            .spawn((
                Mesh2d(meshes.add(build_thruster_mesh())),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
                Transform::from_translation(Vec3::new(0.0, -28.0, -0.1)),
                Thruster {
                    base_length: 12.0,
                    max_length: 60.0,
                    speed_factor: 0.25,
                },
                ThrusterOwner(ship_entity),
            ))
            .id();

        commands.entity(ship_entity).add_child(thruster_entity);
        ship_entity
    }
}

pub fn build_thruster_mesh() -> Mesh {
    let base_width = 8.0;
    let base_length = 12.0;

    let left = [-base_width * 0.5, 0.0, 0.0];
    let right = [base_width * 0.5, 0.0, 0.0];
    let tip = [0.0, -base_length, 0.0];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(), // important pour mises à jour CPU
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![left, right, tip]);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![
            [1.0, 1.0, 0.0, 1.0], // jaune
            [1.0, 1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0, 1.0], // rouge
        ],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
    mesh
}

// ── Map Boundary ───────────────────────────────────────────────────────

impl MapBoundary {
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Entity {
        let map_limit = WORLD_LIMIT;

        let entity = commands
            .spawn((
                Mesh2d(meshes.add(build_rectangle_outline_mesh(map_limit * 2.0, map_limit * 2.0, 3.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(1.0, 0.0, 0.0)))),
                Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                MapBoundary,
            ))
            .id();

        entity
    }
}

// ── Helper functions for creating meshes ────────────────────────────────

pub fn build_triangle_mesh(radius: f32) -> Mesh {
    use std::f32::consts::PI;
    
    // Create a triangle pointing up (0° is up)
    let angle_offset = -(PI + (PI/2.0)); // Rotate to point right initially
    let vertices = [
        [
            radius * (angle_offset + 0.0 * 2.0 * PI / 3.0).cos(),
            radius * (angle_offset + 0.0 * 2.0 * PI / 3.0).sin(),
            0.0,
        ],
        [
            radius * (angle_offset + 1.0 * 2.0 * PI / 3.0).cos(),
            radius * (angle_offset + 1.0 * 2.0 * PI / 3.0).sin(),
            0.0,
        ],
        [
            radius * (angle_offset + 2.0 * 2.0 * PI / 3.0).cos(),
            radius * (angle_offset + 2.0 * 2.0 * PI / 3.0).sin(),
            0.0,
        ],
    ];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![vertices[0], vertices[1], vertices[2]]);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
    mesh
}

pub fn build_circle_mesh(radius: f32, segments: usize) -> Mesh {
    use std::f32::consts::PI;
    
    let mut positions = vec![[0.0, 0.0, 0.0]]; // Center
    
    for i in 0..=segments {
        let angle = (i as f32) * 2.0 * PI / (segments as f32);
        positions.push([
            radius * angle.cos(),
            radius * angle.sin(),
            0.0,
        ]);
    }
    
    let mut indices = Vec::new();
    for i in 1..=segments {
        indices.push(0u32);
        indices.push(i as u32);
        indices.push((i % segments) as u32 + 1);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn build_rectangle_outline_mesh(width: f32, height: f32, thickness: f32) -> Mesh {
    let half_w = width / 2.0;
    let half_h = height / 2.0;
    let half_t = thickness / 2.0;

    // Create 4 rectangles for the outline (top, bottom, left, right)
    let positions = vec![
        // Top bar
        [-half_w, half_h - half_t, 0.0],
        [half_w, half_h - half_t, 0.0],
        [half_w, half_h + half_t, 0.0],
        [-half_w, half_h + half_t, 0.0],
        // Bottom bar
        [-half_w, -half_h - half_t, 0.0],
        [half_w, -half_h - half_t, 0.0],
        [half_w, -half_h + half_t, 0.0],
        [-half_w, -half_h + half_t, 0.0],
        // Left bar
        [-half_w - half_t, -half_h, 0.0],
        [-half_w + half_t, -half_h, 0.0],
        [-half_w + half_t, half_h, 0.0],
        [-half_w - half_t, half_h, 0.0],
        // Right bar
        [half_w - half_t, -half_h, 0.0],
        [half_w + half_t, -half_h, 0.0],
        [half_w + half_t, half_h, 0.0],
        [half_w - half_t, half_h, 0.0],
    ];

    let indices = vec![
        // Top
        0, 1, 2, 0, 2, 3,
        // Bottom
        4, 5, 6, 4, 6, 7,
        // Left
        8, 9, 10, 8, 10, 11,
        // Right
        12, 13, 14, 12, 14, 15,
    ];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}