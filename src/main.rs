use std::ops::{Add, Div, Sub};
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::math::DVec3;
use crate::simulation::{BodyConfig, Config, GravityPlugin};

mod simulation;
mod cursor;

fn main() {
    let mut spawn_points = [
        DVec3::new(0., 0., 0.),
        DVec3::new(30., 0., 0.),
        DVec3::new(0., 40., 0.),
    ];
    spawn_points = center_coordinates(spawn_points);
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("3 Body Problem"),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(cursor::CursorPlugin)
        .add_plugins(GravityPlugin::new(
            Config {
                initial_bodies: vec![
                    BodyConfig {
                        radius: 1.,
                        mass: 1.,
                        position: spawn_points[0],
                        velocity: DVec3::new(0., 0., 0.),
                        color: Some(LinearRgba::rgb(130.99, 50.32, 20.0)),
                        trail_color: Some(
                            LinearRgba::new(1.399, 0.532, 0.2, 0.4)
                        ),
                        trail_length: 300,
                        ..default()
                    },
                    BodyConfig {
                        radius: 1.,
                        mass: 1.,
                        position: spawn_points[1],
                        velocity: DVec3::new(0., 0., 0.),
                        color: Some(LinearRgba::rgb(20.0, 130.99, 50.32)),
                        trail_color: Some(
                            LinearRgba::new(0.2, 1.399, 0.532, 0.4)
                        ),
                        trail_length: 300,
                        ..default()
                    },
                    BodyConfig {
                        radius: 1.,
                        mass: 1.,
                        position: spawn_points[2],
                        velocity: DVec3::new(0., 0., 0.),
                        color: Some(LinearRgba::rgb(50.32, 20.0, 130.99)),
                        trail_color: Some(
                            LinearRgba::new(0.532, 0.2, 1.399, 0.4)
                        ),
                        trail_length: 300,
                        ..default()
                    },
                ],
                timestep: (3.1536e7 / 12.) * 2., // 2 months / second
            },
        ))
        .add_systems(Startup, setup)
        .run();
}

fn center_coordinates(triangle_verts: [DVec3; 3]) -> [DVec3; 3] {
    let center = triangle_verts.iter()
        .fold(DVec3::ZERO, |acc, v| acc + *v) / 3.0;
    triangle_verts.map(|v| v - center)
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_xyz(0.0, 0.0, 5.)
                .looking_at(Vec3::default(), Vec3::Y),
            projection: OrthographicProjection {
                scale: 0.08,
                ..default()
            }.into(),
            ..default()
        },
        BloomSettings::NATURAL,
        cursor::MainCamera,
    ));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 1_500.,
            ..default()
        },
        ..default()
    });
}
