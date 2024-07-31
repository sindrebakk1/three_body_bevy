use bevy::prelude::{App, Camera, Component, GlobalTransform, Plugin, Query, ResMut, Resource, Update, Vec2, Window, With};
use bevy::window::PrimaryWindow;

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
pub struct CursorCoords(pub(crate) Vec2);

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorCoords::default())
            .add_systems(Update, my_cursor_system);
    }
}

fn my_cursor_system(
    mut world_coordinates: ResMut<CursorCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(Option<&Camera>, Option<&GlobalTransform>), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    if let (Some(camera), Some(camera_transform)) = q_camera.single() {
        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        if let Some(world_position) = q_window.single().cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            world_coordinates.0 = world_position;
        }
    }
}