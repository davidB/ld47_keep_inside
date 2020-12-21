// disable console opening on windows
#![windows_subsystem = "windows"]

use bevy::{
    // diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    prelude::*,
    render::camera::Camera,
    window::CursorMoved,
    window::WindowMode,
};
use bevy_easings::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_6, PI};
//use bevy_input::gamepad::{GamepadEvent, GamepadEventType};
#[cfg(target_arch = "wasm32")]
use bevy_webgl2;

#[bevy_main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::build();
    app.add_resource(Msaa { samples: 4 })
        .add_resource(WindowDescriptor {
            title: "Keep Inside".to_string(),
            width: 800.,
            height: 600.,
            vsync: true,
            resizable: true,
            mode: WindowMode::Windowed,
            //WindowMode::Fullscreen { use_size: true },
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EasingsPlugin)
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
        .add_system(update_paddle_transform.system())
        .add_system(hit_to_fx.system())
        .add_system(update_paddle_fx.system())
        .add_system(custom_ease_system::<ImpactFx>.system())
        .add_system(start_system.system())
        .add_system(start_control_system.system())
        .add_system(hit_as_score.system())
        .add_system(scoreboard_system.system());
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    // // Adds frame time diagnostics
    // app.add_plugin(FrameTimeDiagnosticsPlugin::default())
    // // Adds a system that prints diagnostics to the console
    // .add_plugin(PrintDiagnosticsPlugin::default());

    app.run();
    Ok(())
}

struct State {
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
    angle_origin: f32,
    angle_speed: f32,
    half_surface_angle: f32,
    half_height: f32,
}

impl Paddle {
    fn set_angle(&mut self, angle: f32) {
        let previous_angle = self.angle_origin;
        let new_angle = positive_angle(angle);
        let angle_delta = new_angle - previous_angle;
        self.angle_origin = new_angle;
        self.angle_speed = if angle_delta > PI {
            new_angle - (previous_angle + 2.0 * PI)
        } else if angle_delta < -PI {
            new_angle + 2.0 * PI - previous_angle
        } else {
            angle_delta
        };
    }
}

struct Hit {
    direction: Vec3,
}
#[derive(Default, Clone, Debug)]
struct ImpactFx {
    mvt: Vec3,
}

impl Lerp for ImpactFx {
    type Scalar = f32;

    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        ImpactFx {
            mvt: self.mvt.lerp(other.mvt.clone(), *scalar),
        }
    }
}

struct Ball {
    mvt_dir: Vec3,
    velocity_indicator: i32,
    radius: f32,
}

impl Ball {
    //TODO try a quadratic, log or bezier curve
    fn velocity(&self) -> f32 {
        410.0 + 10.0 * self.velocity_indicator as f32
    }
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
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let font_score_handle = asset_server.load("Eduardo-Barrasa.ttf");
    let font_text_handle = asset_server.load("FiraMono-Medium.ttf");
    commands
        .spawn(CameraUiBundle::default())
        // scoreboard
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            draw: Draw {
                // is_transparent: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text {
                        value: "0".to_string(),
                        font: font_score_handle,
                        style: TextStyle {
                            font_size: 100.0,
                            color: Color::rgb_u8(0x00, 0xAA, 0xAA),
                            alignment: TextAlignment::default(),
                        },
                    },
                    ..Default::default()
                })
                .with(ScoreText {});
        })
        .spawn(TextBundle {
            text: Text {
                font: font_text_handle.clone(),
                value: "Click or Button (A) on Gamepad\nto spawn a ball and to start".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8),
                    font_size: 20.0,
                    alignment: TextAlignment::default(),
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
        .spawn(TextBundle {
            text: Text {
                font: font_text_handle,
                value: "Best: 0".to_string(),
                style: TextStyle {
                    color: Color::rgb(0.2, 0.2, 0.8),
                    font_size: 20.0,
                    alignment: TextAlignment::default(),
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

fn add_paddle(
    commands: &mut Commands,
    //asset_server: Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    radius: f32,
    height: f32,
    surface_angle: f32,
) {
    let mut builder = PathBuilder::new();
    builder.move_to(point(radius, 0.01));
    builder.arc(
        point(0.0, 0.0),
        radius,
        radius,
        surface_angle, // use negative angles for a clockwise arc.
        0.0,
    );
    // Calling `PathBuilder::build` will return a `Path` ready to be used to create
    // Bevy entities.
    let path = builder.build();
    let paddle_material = materials.add(Color::rgb(0.1, 0.4, 0.5).into());
    let circle_material = materials.add(Color::rgba(0.5, 0.4, 0.1, 0.8).into());
    commands
        .spawn(path.stroke(
            paddle_material,
            meshes,
            Vec3::new(0.0, 0.0, 0.0),
            &StrokeOptions::default().with_line_width(height), //.with_line_cap(LineCap::Round)
                                                               //.with_line_join(LineJoin::Round)
        ))
        .with(Paddle {
            radius_origin: radius,
            half_surface_angle: surface_angle / 2.0,
            half_height: height / 2.0,
            angle_origin: 0.0,
            angle_speed: 0.0,
        })
        .spawn(primitive(
            circle_material,
            meshes,
            ShapeType::Circle(radius),
            TessellationMode::Stroke(&StrokeOptions::default().with_line_width(1.0)),
            Vec3::zero().into(),
        ));
}

fn setup(
    commands: &mut Commands,
    //asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let camera = Camera2dBundle::default();
    let camera_e = commands.spawn(camera).current_entity().unwrap();
    add_paddle(
        commands,
        &mut meshes,
        &mut materials,
        RADIUS_EXTERN,
        12.0,
        FRAC_PI_6,
    );
    add_paddle(
        commands,
        &mut meshes,
        &mut materials,
        RADIUS_INTERN,
        4.0,
        FRAC_PI_6,
    );
    commands.insert_resource(State {
        cursor_moved_event_reader: Default::default(),
        camera_e,
        game_state_event_reader: Default::default(),
    });
    commands.insert_resource(ClearColor(Color::rgba(
        232.0 / 255.0,
        233.0 / 255.0,
        235.0 / 255.0,
        1.0,
    )));
}

fn start_system(
    commands: &mut Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut state: ResMut<State>,
    game_state_events: Res<Events<GameStateEvent>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query_balls: Query<(Entity, &Ball)>,
) {
    let material = materials.add(Color::rgb(0.8, 0.0, 0.0).into());
    let radius = 5.0;
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
                    // .spawn(SpriteBundle {
                    //     material: ball_material.clone(),
                    //     transform: Transform::from_translation(Vec3::new(
                    //         10.0,
                    //         -(RADIUS_EXTERN + RADIUS_INTERN) / 2.0,
                    //         1.0,
                    //     )),
                    //     ..Default::default()
                    // })
                    .spawn(primitive(
                        material.clone(),
                        &mut meshes,
                        ShapeType::Circle(radius),
                        TessellationMode::Fill(&FillOptions::default()),
                        Vec3::new(10.0, -(RADIUS_EXTERN + RADIUS_INTERN) / 2.0, 1.0).into(),
                    ))
                    .with(Ball {
                        velocity_indicator: 0,
                        mvt_dir: Vec3::new(0.5, -0.5, 0.0).normalize(),
                        radius,
                    });
            }
        }
    }
}

fn start_control_system(
    mouse_button_input: Res<Input<MouseButton>>,
    mut game_state_events: ResMut<Events<GameStateEvent>>,
    gamepad_manager: ResMut<GamepadState>,
    gamepad_inputs: Res<Input<GamepadButton>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
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
    mut query: Query<&mut Paddle>,
    // query to get camera Bundle
    q_camera: Query<(&Camera, &Transform)>,
) {
    if let Ok((_, camera_transform)) = q_camera.get(state.camera_e) {
        for ev in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
            let pos_wld = find_mouse_position(ev, &wnds, &camera_transform);
            for mut paddle in query.iter_mut() {
                let mouse_angle = pos_wld.y.atan2(pos_wld.x);
                paddle.set_angle(mouse_angle);
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
    mut query: Query<&mut Paddle>,
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
                for mut paddle in query.iter_mut() {
                    // eprintln!("rot via gamepad:  {:?}", rot);
                    paddle.set_angle(angle);
                }
            }
        }
    }
}

fn find_ball_paddle_collision_point(
    ball_translation_current: &Vec3,
    ball_translation_previous: &Vec3,
    ball: &Ball,
    paddle: &Paddle,
) -> Option<(Vec3, f32)> {
    let current_o_dist = ball_translation_current.length();
    let previous_o_dist = ball_translation_previous.length();
    let mvt_dir = (current_o_dist - previous_o_dist).signum();
    let range = paddle.half_height + ball.radius;
    let paddle_o_dist = paddle.radius_origin - mvt_dir * range;
    let maybe_collision_o_dist =
        if previous_o_dist < paddle_o_dist && paddle_o_dist <= current_o_dist {
            Some(paddle_o_dist)
        } else if previous_o_dist > paddle_o_dist && paddle_o_dist >= current_o_dist {
            Some(paddle_o_dist)
        } else {
            None
        };
    maybe_collision_o_dist.and_then(|collision_o_dist| {
        let ratio = (collision_o_dist - previous_o_dist) / (current_o_dist - previous_o_dist);
        let collision_point = *ball_translation_previous
            + ((*ball_translation_current - *ball_translation_previous).normalize() * ratio);
        let collision_rot = positive_angle(collision_point.y.atan2(collision_point.x));
        let paddle_rot = paddle.angle_origin;
        if (collision_rot - paddle_rot).abs() <= paddle.half_surface_angle {
            Some((collision_point, ratio))
        } else {
            None
        }
    })
}

fn positive_angle(angle: f32) -> f32 {
    let a = (angle + (2.0 * PI)) % (2.0 * PI);
    a
}

fn ball_movement_system(
    commands: &mut Commands,
    time: Res<Time>,
    mut ball_query: Query<(&mut Ball, &mut Transform)>,
    paddle_query: Query<(Entity, &Paddle)>,
) {
    // clamp the timestep to stop the ball from escaping when the game starts
    let delta_seconds = f32::max(1.0 / 60.0, f32::min(1.0, time.delta_seconds()));

    for (mut ball, mut transform) in ball_query.iter_mut() {
        let ball_translation_previous = transform.translation;
        transform.translation += (ball.velocity() * delta_seconds) * ball.mvt_dir;
        for (entity, paddle) in paddle_query.iter() {
            commands.remove_one::<Hit>(entity);
            let maybe_collision_point = find_ball_paddle_collision_point(
                &transform.translation,
                &ball_translation_previous,
                &ball,
                paddle,
            );
            if let Some((collision_point, ratio)) = maybe_collision_point {
                commands.insert_one(
                    entity,
                    Hit {
                        direction: ball.mvt_dir,
                    },
                );
                let normal_surface =
                    Vec3::new(-collision_point.x, -collision_point.y, 0.0).normalize();
                let speed_impact = 1.0 * paddle.angle_speed / (delta_seconds * 2.0 * PI);
                let mirror = normal_surface
                    + Vec3::new(
                        -normal_surface.y * speed_impact,
                        normal_surface.x * speed_impact,
                        0.0,
                    );
                let mvt_dir = reflect_2d(ball.mvt_dir, mirror.normalize());
                ball.mvt_dir = mvt_dir;
                ball.velocity_indicator += 1;
                transform.translation = collision_point
                    + ((1.0 - ratio) * (ball.velocity() * delta_seconds)) * ball.mvt_dir;
            }
        }
    }
}

fn reflect_2d(v: Vec3, n: Vec3) -> Vec3 {
    let d = v.x * n.x + v.y * n.y; //dot(v, n)
    Vec3::new(v.x - 2.0 * d * n.x, v.y - 2.0 * d * n.y, 0.0)
}

fn hit_as_score(mut scoreboard: ResMut<Scoreboard>, paddle_query: Query<(&Paddle, &Hit)>) {
    for (_paddle, _hit) in paddle_query.iter() {
        scoreboard.score += 1;
    }
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

fn update_paddle_transform(mut paddle_query: Query<(&Paddle, &mut Transform)>) {
    for (paddle, mut paddle_transform) in paddle_query.iter_mut() {
        paddle_transform.rotation =
            Quat::from_rotation_z(paddle.angle_origin - paddle.half_surface_angle);
    }
}

fn update_paddle_fx(mut paddle_query: Query<(&Paddle, &ImpactFx, &mut Transform)>) {
    for (_paddle, impact, mut paddle_transform) in paddle_query.iter_mut() {
        paddle_transform.translation = impact.mvt;
        dbg!(paddle_transform.translation);
    }
}

fn hit_to_fx(commands: &mut Commands, paddle_query: Query<(Entity, &Paddle, &Hit)>) {
    for (entity, _paddle, hit) in paddle_query.iter() {
        let impact = ImpactFx { mvt: Vec3::zero() };
        let e_impact = impact
            .clone()
            .ease_to(
                ImpactFx {
                    mvt: hit.direction * 20.0,
                },
                EaseFunction::BounceOut,
                EasingType::Once {
                    duration: std::time::Duration::from_millis(30),
                },
            )
            .ease_to(
                impact.clone(),
                EaseFunction::BounceOut,
                EasingType::Once {
                    duration: std::time::Duration::from_millis(20),
                },
            );
        dbg!("add impact");
        commands.insert_one(entity, impact);
        commands.insert_one(entity, e_impact);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_angle() {
        assert_eq!(positive_angle(0.0), 0.0);
        assert_eq!(positive_angle(2.0 * PI), 0.0);
        assert_eq!(positive_angle(PI), PI);
        assert_eq!(positive_angle(-PI), PI);
        assert_eq!(positive_angle(-0.5 * PI), 1.5 * PI);
    }
}
