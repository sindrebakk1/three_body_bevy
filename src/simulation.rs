use std::time::Duration;
use bevy::ecs::world::Command;
use bevy::input::common_conditions::input_just_pressed;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;

const G: f64 = 11.334e-12;

// STATE
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, States)]
pub enum TrailState {
    Hide,
    #[default]
    Show,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, States)]
pub enum  SimulationState {
    #[default]
    Stopped,
    Running,
}

// COMPONENTS
#[derive(Bundle)]
struct BodyBundle {
    body: Body,
    position: Position,
    velocity: Velocity,
    mass: Mass,
    acceleration: Acceleration,
    mesh: PbrBundle,
    config: BodyConfig,
}

#[derive(Component)]
struct Body;

#[derive(Component)]
struct Position(DVec3);

#[derive(Component)]
struct Velocity(DVec3);

#[derive(Component)]
struct Mass(f64);

#[derive(Component)]
struct Acceleration(DVec3);

#[derive(Bundle)]
struct TrailBundle {
    trail: Trail,
    decay: TrailDecay,
    color: TrailColor,
    mesh: PbrBundle,
}

#[derive(Component, Default)]
struct Trail {
    max_length: usize,
    points: Vec<Vec3>,
}

#[derive(Component)]
struct TrailDecay(Duration);

#[derive(Component)]
struct TrailColor(Color);

#[derive(Component)]
struct TrailRef(Entity);

// RESOURCES
#[derive(Clone, Resource)]
pub struct Config {
    pub initial_bodies: Vec<BodyConfig>,
    pub timestep: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            initial_bodies: vec![],
            timestep: 1.,
        }
    }
}

#[derive(Clone, Component)]
pub struct BodyConfig {
    pub radius: f64,
    pub mass: f64,
    pub position: DVec3,
    pub velocity: DVec3,
    pub color: Option<LinearRgba>,
    pub trail_color: Option<LinearRgba>,
    pub trail_length: usize,
}

impl Default for BodyConfig {
    fn default() -> Self {
        Self {
            radius: 1.,
            mass: 1.,
            position: DVec3::ZERO,
            velocity: DVec3::ZERO,
            color: None,
            trail_color: None,
            trail_length: 100,
        }
    }
}

#[derive(Resource)]
pub struct BodyMesh(Handle<Mesh>);

// COMMANDS
struct SpawnBodyCommand {
    // you can have some parameters
    body: BodyConfig,
}

impl Command for SpawnBodyCommand {
    fn apply(self, world: &mut World) {
        // Retrieve and store the necessary resources in local variables
        let body_mesh = world.get_resource::<BodyMesh>().unwrap().0.clone();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        let body_color = self.body.color.clone()
            .unwrap_or(LinearRgba::rgb(150., 150., 150.));

        let body_material = materials.add(StandardMaterial {
            emissive: body_color,
            ..default()
        });

        world.spawn(BodyBundle {
            body: Body,
            position: Position(self.body.position),
            velocity: Velocity(self.body.velocity),
            acceleration: Acceleration(DVec3::ZERO),
            mass: Mass(self.body.mass),
            mesh: PbrBundle {
                mesh: body_mesh,
                material: body_material,
                transform: Transform {
                    translation: self.body.position.as_vec3(),
                    scale: Vec3::splat(self.body.radius as f32),
                    ..default()
                },
                ..default()
            },
            config: self.body.clone(),
        });
    }
}

pub trait SpawnBodyCommandExt {
    // define a method that we will be able to call on `commands`
    fn spawn_body(&mut self, body: &BodyConfig);
}

// implement our trait for Bevy's `Commands`
impl<'w, 's> SpawnBodyCommandExt for Commands<'w, 's> {
    fn spawn_body(&mut self, body: &BodyConfig) {
        self.add(SpawnBodyCommand {
            body: body.clone(),
        });
    }
}

// PLUGIN
pub(crate) struct GravityPlugin {
    config: Config,
}

impl GravityPlugin {
    pub fn new(cfg: Config) -> Self {
        Self { config: cfg }
    }
}

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimulationState>()
            .init_state::<TrailState>()
            .insert_resource(self.config.clone())
            .add_systems(Startup, (setup, spawn_initial_bodies).chain())
            .add_systems(
                FixedUpdate,
                (gravity, update_body)
                    .run_if(in_state(SimulationState::Running))
                    .chain()
            )
            .add_systems(
                Update,
                (update_trail, draw_trail)
                    .run_if(in_state(TrailState::Show))
                    .run_if(in_state(SimulationState::Running))
                    .after(update_body)
                    .chain()
            )
            .add_systems(Update,(
                toggle_simulation,
                toggle_trail,
                spawn_on_click.run_if(input_just_pressed(MouseButton::Left))
            ));
    }
}

// SYSTEMS
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(
        BodyMesh(meshes.add(Sphere::new(1.0).mesh().ico(3).unwrap()))
    )
}

fn spawn_initial_bodies(
    mut commands: Commands,
    config: Res<Config>,
) {
    for body in config.initial_bodies.iter() {
        commands.spawn_body(body);
    }
}

fn spawn_on_click(
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    cursor: Res<crate::cursor::CursorCoords>
) {
    if input.just_pressed(MouseButton::Left) {
       commands.spawn_body(&BodyConfig {
           radius: 0.2,
           mass: 0.2,
           position: DVec3::from((cursor.0.as_dvec2(),0.)),
           velocity: DVec3::ZERO,
           color: Some(LinearRgba::rgb(5., 5., 5.)),
           trail_color: Some(LinearRgba::new(1., 1., 1., 0.4)),
           trail_length: 20,
       });
    }
}

fn gravity(mut query: Query<(&Mass, &GlobalTransform, &mut Acceleration), With<Body>>) {
    let mut iter = query.iter_combinations_mut();
    while let Some(
        [
           (m1, t1, mut a1),
           (m2, t2, mut a2)
       ]
    ) = iter.fetch_next() {
        let delta = t2.translation().as_dvec3() - t1.translation().as_dvec3();
        let distance_sq = delta.length_squared();

        if distance_sq == 0.0 {
            continue;
        }

        let f = G / distance_sq;
        let force_unit_mass = delta * f;
        a1.0 += force_unit_mass * m2.0;
        a2.0 -= force_unit_mass * m1.0;
    }
}

fn update_body(
    time: Res<Time>,
    mut query: Query<(&mut Acceleration, &mut Transform, &mut Position, &mut Velocity), With<Body>>,
    config: Res<Config>,
) {
    let dt = time.delta_seconds_f64() * config.timestep;
    for (
        mut a,
        mut t,
        mut p,
        mut v
    ) in query.iter_mut() {
        v.0 += a.0 * dt;
        p.0 += v.0 * dt;
        a.0 = DVec3::ZERO;
        t.translation = p.0.as_vec3();
    }
}

fn update_trail(
    mut query: Query<(&Position, &TrailRef), With<Body>>,
    mut trail_query: Query<&mut Trail, With<Trail>>,
) {
    for (pos, trail_entity) in query.iter_mut() {
        if let Ok(mut trail) = trail_query.get_mut(trail_entity.0) {
            if trail.points.len() >= trail.max_length {
                trail.points.remove(0);
            }
            trail.points.push(pos.0.as_vec3());
        }
    }
}

fn draw_trail(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(Entity, &BodyConfig, &GlobalTransform, Option<&TrailRef>), With<Body>>,
    mut trail_entity_query: Query<(&Trail, &Handle<Mesh>, &Handle<StandardMaterial>), With<Trail>>,
) {
    for (
        body, config, transform, trail
    ) in query.iter_mut() {
        if let Some(trail_entity) = trail {
            if let Ok(
                (trail, trail_mesh_handle, _trail_material_handle)
            ) = trail_entity_query.get_mut(trail_entity.0) {
                let trail_mesh = meshes.get_mut(trail_mesh_handle).unwrap();
                let positions: Vec<[f32; 3]> = trail.points
                    .iter()
                    .map(|p| p.to_array())
                    .collect();
                trail_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                continue;
            } else {
                commands.entity(trail_entity.0).despawn_recursive();
            }
        }
        let trail_color = config.trail_color.clone()
            .unwrap_or(config.color.clone()
                .unwrap_or(LinearRgba::rgb(150., 150., 150.)));
        let trail_material_handle = materials.add(StandardMaterial {
            emissive: trail_color,
            ..default()
        });

        // Create the trail mesh
        let mut trail_mesh = Mesh::new(
            PrimitiveTopology::LineStrip,
            RenderAssetUsages::default()
        );
        let trail_positions: Vec<Vec3> = vec![config.position.as_vec3()];
        trail_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, trail_positions.clone());
        let trail_mesh_handle = meshes.add(trail_mesh);

        let trail = commands.spawn(TrailBundle {
            trail: Trail {
                max_length: config.trail_length,
                points: trail_positions.clone(),
            },
            decay: TrailDecay(Duration::new(1, 0)),
            color: TrailColor(trail_color.into()),
            mesh: PbrBundle {
                mesh: trail_mesh_handle,
                material: trail_material_handle,
                global_transform: *transform,
                ..default()
            }
        }).id();

        commands.get_entity(body).unwrap().insert(TrailRef(trail));
    }
}

fn toggle_simulation(
    state: Res<State<SimulationState>>,
    mut next_state: ResMut<NextState<SimulationState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        match state.get() {
            SimulationState::Running => next_state.set(SimulationState::Stopped),
            SimulationState::Stopped => next_state.set(SimulationState::Running),
        }
    }
}

fn toggle_trail(
    state: Res<State<TrailState>>,
    mut next_state: ResMut<NextState<TrailState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyT) {
        match state.get() {
            TrailState::Show => next_state.set(TrailState::Hide),
            TrailState::Hide => next_state.set(TrailState::Show),
        }
    }
}
