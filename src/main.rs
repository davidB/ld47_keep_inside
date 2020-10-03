use bevy::{prelude::*, window::CursorMoved};
use bevy_input::gamepad::{Gamepad, GamepadButton, GamepadEvent, GamepadEventType};
use std::collections::HashSet;

fn main() {
    App::build()
        .add_default_plugins()
        .init_resource::<GamepadState>()
        .add_startup_system(setup.system())
        .add_startup_system(gamepad_connection_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(paddle_control_by_mouse_system.system())
        .add_system(paddle_control_by_gamepad_system.system())
        .run();
}

struct State {
    cursor_moved_event_reader: EventReader<CursorMoved>,
    // need to identify the main camera
    camera_e: Entity,
}

#[derive(Default)]
struct GamepadState {
    gamepad_event_reader: EventReader<GamepadEvent>,
    gamepads: HashSet<Gamepad>,
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
    commands.insert_resource(ClearColor(Color::rgb(
        232.0 / 255.0,
        233.0 / 255.0,
        235.0 / 255.0,
    )));
}

fn paddle_control_by_mouse_system(
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
        let mouse_angle = pos_wld.y().atan2(pos_wld.x());
        let rot = Quat::from_rotation_z(mouse_angle);
        for (_paddle, mut transform) in &mut query.iter() {
            eprintln!("rot via mouse:  {:?}", rot);
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

fn gamepad_connection_system(
    mut gamepad_manager: ResMut<GamepadState>,
    gamepad_event: Res<Events<GamepadEvent>>,
) {
    for event in gamepad_manager.gamepad_event_reader.iter(&gamepad_event) {
        match &event {
            GamepadEvent(gamepad, GamepadEventType::Connected) => {
                gamepad_manager.gamepads.insert(*gamepad);
                println!("Connected {:?}", gamepad);
            }
            GamepadEvent(gamepad, GamepadEventType::Disconnected) => {
                gamepad_manager.gamepads.remove(gamepad);
                println!("Disconnected {:?}", gamepad);
            }
        }
    }
}

fn paddle_control_by_gamepad_system(
    gamepad_manager: Res<GamepadState>,
    axes: Res<Axis<GamepadAxis>>,
    mut query: Query<(&Paddle, &mut Transform)>,
) {
    for gamepad in gamepad_manager.gamepads.iter() {
        let maybe_x = axes
            .get(&GamepadAxis(*gamepad, GamepadAxisType::LeftStickX))
            //.filter(|value| (value - 1.0f32).abs() > 0.01f32 && (value + 1.0f32).abs() > 0.01f32)
            ;
        let maybe_y = axes
            .get(&GamepadAxis(*gamepad, GamepadAxisType::LeftStickY))
            //.filter(|value| (value - 1.0f32).abs() > 0.01f32 && (value + 1.0f32).abs() > 0.01f32)
            ;
        if let Some((x, y)) = maybe_x.zip(maybe_y) {
            // ignore if x and y are in the dead zone
            if x.abs() > 0.03f32 && y.abs() > 0.03f32 {
                let angle = y.atan2(x);
                let rot = Quat::from_rotation_z(angle);
                for (_paddle, mut transform) in &mut query.iter() {
                    eprintln!("rot via gamepad:  {:?}", rot);
                    transform.set_rotation(rot);
                }
            }
        }
    }
}
