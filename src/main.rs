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
        .add_system(ball_movement_system.system())
        .add_system(ball_collision_system.system())
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
struct Ball {
    velocity: Vec3,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let camera = Camera2dComponents::default();
    let camera_e = commands.spawn(camera).current_entity().unwrap();
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/paddle.png").unwrap().into()),
            ..Default::default()
        })
        .with(Paddle {})
        // ball
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/ball.png").unwrap().into()),
            transform: Transform::from_translation(Vec3::new(10.0, -RADIUS_EXTERN + 50.0, 1.0)),
            ..Default::default()
        })
        .with(Ball {
            velocity: 400.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
        });
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
            eprintln!("rot via mouse:  {:?} {:?}", rot, pos_wld);
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

fn ball_movement_system(time: Res<Time>, mut ball_query: Query<(&Ball, &mut Transform)>) {
    // clamp the timestep to stop the ball from escaping when the game starts
    let delta_seconds = f32::min(0.2, time.delta_seconds);

    for (ball, mut transform) in &mut ball_query.iter() {
        transform.translate(ball.velocity * delta_seconds);
    }
}
const RADIUS_EXTERN: f32 = 280.0;
const RADIUS_PADDLE_EXTERN: f32 = 100.0;
const RADIUS_INTERN: f32 = 108.0;
const RADIUS_PADDLE_INTERN: f32 = 40.0;

fn ball_collision_system(
    mut ball_query: Query<(&mut Ball, &mut Transform)>,
    mut paddle_query: Query<(&Paddle, &Transform)>,
) {
    for (mut ball, mut transform) in &mut ball_query.iter() {
        let o_dist = transform.translation().length();

        if o_dist >= RADIUS_EXTERN || o_dist <= RADIUS_INTERN {
            for (_, paddle_transform) in &mut paddle_query.iter() {
                let paddle_extern =
                    paddle_transform
                        .value()
                        .transform_vector3(Vec3::new(RADIUS_EXTERN, 0.0, 0.0));
                let paddle_dist = (paddle_extern - transform.translation()).length();
                let collide = (o_dist >= RADIUS_EXTERN && paddle_dist <= RADIUS_PADDLE_EXTERN)
                    || (o_dist <= RADIUS_INTERN && paddle_dist <= RADIUS_PADDLE_INTERN);
                if collide {
                    // FIXME a workaround relocate the ball else strange behaior
                    let n = transform.translation().normalize();
                    let dist = if o_dist >= RADIUS_EXTERN {
                        RADIUS_EXTERN - 2.0
                    } else {
                        RADIUS_INTERN + 2.0
                    };
                    transform.set_translation(n * dist);
                    // reflect on axix origin / current position
                    let o_angle = transform
                        .translation()
                        .y()
                        .atan2(transform.translation().x());
                    let dest = compute_reflection(o_angle, transform.translation() - ball.velocity);
                    ball.velocity = dest - transform.translation();
                    // ball.velocity = compute_reflection(o_angle, ball.velocity);
                    // eprintln!(
                    //     "out of the zone {:?} > {:?} new velocity {:?} {:?}",
                    //     o_dist,
                    //     ball.o_dist,
                    //     ball.velocity,
                    //     ball.velocity.length()
                    // );
                }
            }
        }
    }
}

fn compute_reflection(x_axis_angle: f32, position: Vec3) -> Vec3 {
    let mat_rotate_space = Mat3::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), x_axis_angle);
    let mat_mirror_x = Mat3::from_cols_array(&[1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0]);
    mat_rotate_space * mat_mirror_x * mat_rotate_space.inverse() * position
}
