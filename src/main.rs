// mod assets;

// use self::assets::AssetHandles;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    input::mouse::MouseButtonInput,
    prelude::*,
    render::camera::Camera,
    window::CursorMoved,
};
//use bevy_input::gamepad::{GamepadEvent, GamepadEventType};
use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        // Adds frame time diagnostics
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Adds a system that prints diagnostics to the console
        .add_plugin(PrintDiagnosticsPlugin::default())
        .add_event::<GameStateEvent>()
        .init_resource::<GamepadState>()
        .add_resource(Scoreboard { score: 0, best: 0 })
        .add_startup_system(setup.system())
        .add_startup_system(setup_ui.system())
        .add_startup_system(gamepad_connection_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(paddle_control_by_mouse_system.system())
        .add_system(paddle_control_by_gamepad_system.system())
        .add_system(ball_movement_system.system())
        .add_system(ball_collision_system.system())
        .add_system(start_system.system())
        .add_system(start_control_system.system())
        .add_system(scoreboard_system.system())
        .run();
    Ok(())
}

struct State {
    mouse_button_event_reader: EventReader<MouseButtonInput>,
    cursor_moved_event_reader: EventReader<CursorMoved>,
    // need to identify the main camera
    camera_e: Entity,
    game_state_event_reader: EventReader<GameStateEvent>,
}

#[derive(Default)]
struct GamepadState {
    gamepad_event_reader: EventReader<GamepadEvent>,
    gamepads: HashSet<Gamepad>,
}

const RADIUS_EXTERN: f32 = 285.0;
const RADIUS_INTERN: f32 = 108.0;

struct Paddle {
    radius_origin: f32,
    half_height: f32,
    half_width: f32,
}

struct Ball {
    velocity: Vec3,
}

struct Scoreboard {
    score: usize,
    best: usize,
}
struct ScoreText {}
struct ScoreBestText {}
enum GameStateEvent {
    Start,
}

fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let font_score_handle = asset_server.load("Eduardo-Barrasa.ttf");
    let font_text_handle = asset_server.load("FiraMono-Medium.ttf");
    commands
        .spawn(UiCameraComponents::default())
        // scoreboard
        .spawn(NodeComponents {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            draw: Draw {
                is_transparent: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextComponents {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text {
                        value: "0".to_string(),
                        font: font_score_handle,
                        style: TextStyle {
                            font_size: 100.0,
                            color: Color::rgb_u8(0x00, 0xAA, 0xAA),
                        },
                    },
                    ..Default::default()
                })
                .with(ScoreText {});
        })
        .spawn(TextComponents {
            text: Text {
                font: font_text_handle.clone(),
                value: "Click or Button (A) on Gamepad\nto spawn a ball and to start".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8),
                    font_size: 20.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .spawn(TextComponents {
            text: Text {
                font: font_text_handle,
                value: "Best: 0".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8),
                    font_size: 20.0,
                },
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    right: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(ScoreBestText {});
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let camera = Camera2dComponents::default();
    let camera_e = commands.spawn(camera).current_entity().unwrap();
    let paddle_asset = asset_server.load("paddle.png");
    let paddle_material = materials.add(paddle_asset.into());
    commands
        .spawn(SpriteComponents {
            material: paddle_material.clone(),
            ..Default::default()
        })
        .with(Paddle {
            radius_origin: RADIUS_EXTERN,
            half_height: 5.0,
            half_width: 90.0,
        })
        .spawn(SpriteComponents {
            material: paddle_material,
            ..Default::default()
        })
        .with(Paddle {
            radius_origin: RADIUS_INTERN,
            half_height: 2.0,
            half_width: 20.0,
        });
    commands.insert_resource(State {
        mouse_button_event_reader: Default::default(),
        cursor_moved_event_reader: Default::default(),
        camera_e,
        game_state_event_reader: Default::default(),
    });
    commands.insert_resource(ClearColor(Color::rgb(
        232.0 / 255.0,
        233.0 / 255.0,
        235.0 / 255.0,
    )));
}

fn start_system(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut state: ResMut<State>,
    game_state_events: Res<Events<GameStateEvent>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query_balls: Query<(Entity, &Ball)>,
) {
    let ball_asset = asset_server.load("ball.png");
    let ball_material = materials.add(ball_asset.into());
    for ev in state.game_state_event_reader.iter(&game_state_events) {
        match ev {
            GameStateEvent::Start => {
                scoreboard.best = scoreboard.best.max(scoreboard.score);
                scoreboard.score = 0;
                // remove existing balls
                for (entity, _) in query_balls.iter() {
                    commands.despawn(entity);
                }
                // ball
                commands
                    .spawn(SpriteComponents {
                        material: ball_material.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            10.0,
                            -(RADIUS_EXTERN + RADIUS_INTERN) / 2.0,
                            1.0,
                        )),
                        ..Default::default()
                    })
                    .with(Ball {
                        velocity: 410.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
                    });
            }
        }
    }
}

fn start_control_system(
    mut state: ResMut<State>,
    mouse_button_input_events: Res<Events<MouseButtonInput>>,
    mut game_state_events: ResMut<Events<GameStateEvent>>,
    gamepad_manager: ResMut<GamepadState>,
    gamepad_inputs: Res<Input<GamepadButton>>,
) {
    for _event in state
        .mouse_button_event_reader
        .iter(&mouse_button_input_events)
    {
        // eprintln!("{:?}", event);
        game_state_events.send(GameStateEvent::Start)
    }
    for gamepad in gamepad_manager.gamepads.iter() {
        if gamepad_inputs.just_released(GamepadButton(*gamepad, GamepadButtonType::South)) {
            // eprintln!(
            //     "Released {:?}",
            //     GamepadButton(*gamepad, GamepadButtonType::South)
            // );
            game_state_events.send(GameStateEvent::Start)
        }
    }
}

fn paddle_control_by_mouse_system(
    mut state: ResMut<State>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    wnds: Res<Windows>,
    mut query: Query<(&Paddle, &mut Transform)>,
    // query to get camera components
    q_camera: Query<(&Camera, &Transform)>,
) {
    if let Ok((_, camera_transform)) = q_camera.get(state.camera_e) {
        for ev in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
            let pos_wld = find_mouse_position(ev, &wnds, &camera_transform);
            let mouse_angle = pos_wld.y().atan2(pos_wld.x());
            let rot = Quat::from_rotation_z(mouse_angle);
            for (_paddle, mut transform) in query.iter_mut() {
                //eprintln!("rot via mouse:  {:?} {:?}", rot, pos_wld);
                transform.rotation = rot;
            }
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
    let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

    // the default orthographic projection is in pixels from the center;
    // just undo the translation
    let p = ev.position - size / 2.0;

    // apply the camera transform
    camera_transform.compute_matrix() * p.extend(0.0).extend(1.0)
}

fn gamepad_connection_system(
    mut gamepad_manager: ResMut<GamepadState>,
    gamepad_event: Res<Events<GamepadEvent>>,
) {
    for event in gamepad_manager.gamepad_event_reader.iter(&gamepad_event) {
        match &event {
            GamepadEvent(gamepad, GamepadEventType::Connected) => {
                gamepad_manager.gamepads.insert(*gamepad);
                //eprintln!("Connected {:?}", gamepad);
            }
            GamepadEvent(gamepad, GamepadEventType::Disconnected) => {
                gamepad_manager.gamepads.remove(gamepad);
                //eprintln!("Disconnected {:?}", gamepad);
            }
            _ => (),
        }
    }
}

fn paddle_control_by_gamepad_system(
    gamepad_manager: Res<GamepadState>,
    axes: Res<Axis<GamepadAxis>>,
    mut query: Query<(&Paddle, &mut Transform)>,
) {
    for gamepad in gamepad_manager.gamepads.iter().cloned() {
        let maybe_x = axes
            .get(GamepadAxis(gamepad, GamepadAxisType::LeftStickX))
            //.filter(|value| (value - 1.0f32).abs() > 0.01f32 && (value + 1.0f32).abs() > 0.01f32)
            ;
        let maybe_y = axes
            .get(GamepadAxis(gamepad, GamepadAxisType::LeftStickY))
            //.filter(|value| (value - 1.0f32).abs() > 0.01f32 && (value + 1.0f32).abs() > 0.01f32)
            ;
        if let Some((x, y)) = maybe_x.zip(maybe_y) {
            // ignore if x and y are in the dead zone
            if x.abs() > 0.03f32 && y.abs() > 0.03f32 {
                let angle = y.atan2(x);
                let rot = Quat::from_rotation_z(angle);
                for (_paddle, mut transform) in query.iter_mut() {
                    // eprintln!("rot via gamepad:  {:?}", rot);
                    transform.rotation = rot;
                }
            }
        }
    }
}

fn ball_movement_system(time: Res<Time>, mut ball_query: Query<(&Ball, &mut Transform)>) {
    // clamp the timestep to stop the ball from escaping when the game starts
    let delta_seconds = f32::min(1.0, time.delta_seconds);

    for (ball, mut transform) in ball_query.iter_mut() {
        transform.translation += ball.velocity * delta_seconds;
    }
}

fn find_ball_paddle_collision_point(
    ball_translation_current: &Vec3,
    ball_translation_previous: &Vec3,
    paddle_transform: &Transform,
    paddle: &Paddle,
) -> Option<Vec3> {
    let current_o_dist = ball_translation_current.length();
    let previous_o_dist = ball_translation_previous.length();
    let paddle_o_dist = paddle.radius_origin;
    let maybe_collision_o_dist =
        if previous_o_dist < paddle_o_dist && paddle_o_dist < current_o_dist {
            Some(paddle_o_dist - paddle.half_height)
        } else if previous_o_dist > paddle_o_dist && paddle_o_dist > current_o_dist {
            Some(paddle_o_dist + paddle.half_height)
        } else {
            None
        };
    maybe_collision_o_dist.and_then(|collision_o_dist| {
        let ratio = (current_o_dist - collision_o_dist) / (current_o_dist - previous_o_dist);
        let collision_point = *ball_translation_previous
            + ((*ball_translation_current - *ball_translation_previous).normalize() * ratio);
        let paddle_translation = paddle_transform
            .compute_matrix()
            .transform_vector3(Vec3::new(paddle.radius_origin, 0.0, 0.0));
        if (collision_point - paddle_translation).length_squared()
            <= paddle.half_width * paddle.half_width
        {
            Some(collision_point)
        } else {
            None
        }
    })
}

fn ball_collision_system(
    time: Res<Time>,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<(&mut Ball, &mut Transform)>,
    paddle_query: Query<(&Paddle, &Transform)>,
) {
    let delta_seconds = f32::min(1.0, time.delta_seconds);
    for (mut ball, mut transform) in ball_query.iter_mut() {
        let ball_translation_previous = transform.translation - ball.velocity * delta_seconds;
        for (paddle, paddle_transform) in paddle_query.iter() {
            let maybe_collision_point = find_ball_paddle_collision_point(
                &transform.translation,
                &ball_translation_previous,
                paddle_transform,
                paddle,
            );
            if let Some(collision_point) = maybe_collision_point {
                transform.translation = collision_point;
                // reflect on axix origin / current position
                let o_angle = collision_point.y().atan2(collision_point.x());
                let dest = compute_reflection(o_angle, collision_point - ball.velocity);
                ball.velocity = dest - collision_point;
                scoreboard.score += 1;
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

fn compute_reflection(x_axis_angle: f32, position: Vec3) -> Vec3 {
    let mat_rotate_space = Mat3::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), x_axis_angle);
    let mat_mirror_x = Mat3::from_cols_array(&[1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0]);
    mat_rotate_space * mat_mirror_x * mat_rotate_space.inverse() * position
}

fn scoreboard_system(
    scoreboard: Res<Scoreboard>,
    mut query_scoretext: Query<(&mut Text, &ScoreText)>,
    mut query_scorebesttext: Query<(&mut Text, &ScoreBestText)>,
) {
    for (mut text, _) in query_scoretext.iter_mut() {
        text.value = format!("{}", scoreboard.score);
    }
    for (mut text, _) in query_scorebesttext.iter_mut() {
        text.value = format!("Best: {}", scoreboard.best);
    }
}
