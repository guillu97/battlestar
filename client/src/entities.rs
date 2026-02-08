use crate::components::{Asteroid, Player, Ship, Thruster, ThrusterOwner, Velocity};
use bevy::color::palettes::css::{BLACK, DARK_CYAN, GRAY};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_asset::RenderAssetUsages;
use bevy_prototype_lyon::prelude::*;

// ── Ship ───────────────────────────────────────────────────────────────

impl Ship {
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        position: Vec3,
    ) -> Entity {
        let ship_shape = shapes::RegularPolygon {
            sides: 3,
            feature: shapes::RegularPolygonFeature::Radius(25.0),
            ..Default::default()
        };

        let ship_entity = commands
            .spawn((
                ShapeBuilder::with(&ship_shape)
                    .fill(Fill::color(DARK_CYAN))
                    .stroke(Stroke::new(BLACK, 2.0f32))
                    .build(),
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

// ── Asteroid ───────────────────────────────────────────────────────────

impl Asteroid {
    pub fn spawn(commands: &mut Commands, position: Vec3, radius: f32) -> Entity {
        let asteroid_shape = shapes::Circle {
            radius,
            ..Default::default()
        };

        let entity = commands
            .spawn((
                ShapeBuilder::with(&asteroid_shape)
                    .fill(Fill::color(GRAY))
                    .stroke(Stroke::new(BLACK, 1.0 as f32))
                    .build(),
                Transform::from_translation(position),
                Asteroid,
            ))
            .id();

        entity
    }
}