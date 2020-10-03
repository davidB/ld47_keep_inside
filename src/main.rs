use bevy::{prelude::*, window::CursorMoved};

fn main() {
    App::build()
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(paddle_mouse_control_system.system())
        .run();
}

struct State {
    cursor_moved_event_reader: EventReader<CursorMoved>,
    // need to identify the main camera
    camera_e: Entity,
}

struct Paddle {}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("assets/paddle.png").unwrap();
    let camera = Camera2dComponents::default();
    let camera_e = commands.spawn(camera).current_entity().unwrap();
    commands
        .spawn(SpriteComponents {
            material: materials.add(texture_handle.into()),
            ..Default::default()
        })
        .with(Paddle {});
    commands.insert_resource(State {
        cursor_moved_event_reader: Default::default(),
        camera_e,
    });
}

fn paddle_mouse_control_system(
    mut state: ResMut<State>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    wnds: Res<Windows>,
    mut query: Query<(&Paddle, &mut Transform)>,
    // query to get camera components
    q_camera: Query<&Transform>,
) {
    let camera_transform = q_camera.get::<Transform>(state.camera_e).unwrap();

    for ev in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
        let pos_wld = find_mouse_position(ev, &wnds, &camera_transform);
        eprintln!("World coords: {}/{}", pos_wld.x(), pos_wld.y());
        let mouse_angle = pos_wld.y().atan2(pos_wld.x());
        let rot = Quat::from_rotation_z(mouse_angle);
        for (_paddle, mut transform) in &mut query.iter() {
            transform.set_rotation(rot);
        }
    }
}

// see [Convert screen coordinates to world coordinates](https://github.com/jamadazi/bevy-cookbook/blob/master/bevy-cookbook.md#convert-screen-coordinates-to-world-coordinates)
fn find_mouse_position(
    ev: &CursorMoved,
    wnds: &Res<Windows>,
    camera_transform: &Transform,
) -> Vec4 {
    // get the size of the window that the event is for
    let wnd = wnds.get(ev.id).unwrap();
    let size = Vec2::new(wnd.width as f32, wnd.height as f32);

    // the default orthographic projection is in pixels from the center;
    // just undo the translation
    let p = ev.position - size / 2.0;

    // apply the camera transform
    *camera_transform.value() * p.extend(0.0).extend(1.0)
}
