use std::time::Duration;

use bevy::{core_pipeline::bloom::BloomSettings, prelude::*};
use bevy_kira_audio::prelude::{Audio, *};

pub struct SpaceFloaty;

impl Plugin for SpaceFloaty {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_game)
            .add_system(gamepad_connections)
            .add_system(player_input)
            .add_system(player_move)
            .add_system(gravity)
            .add_system(engine_flame)
            .add_system(camera_track)
            .add_system(spinner);
    }
}

#[derive(Resource)]
struct GamepadData {
    a: Option<Gamepad>,
    b: Option<Gamepad>,
}

enum GamepadID {
    A,
    B,
}

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut wins: Query<&mut Window>,
    audio: Res<Audio>,
) {
    // Camera
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_scale(Vec3::ONE * 2.0),
            camera: Camera {
                hdr: true,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::Custom(
                    Color::rgb(0.06, 0.0, 0.06),
                ),
            },
            ..default()
        },
        BloomSettings {
            ..Default::default()
        },
        MainCamera,
    ));

    commands.insert_resource(GamepadData { a: None, b: None });

    // Earth
    let earth = (
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(200.0, 200.0)),
                ..Default::default()
            },
            texture: asset_server.load("textures/earth.png"),
            transform: Transform::from_xyz(-600.0, 0.0, 0.0),
            ..Default::default()
        },
        GravityProducer { strength: 200000.0 },
    );
    commands.spawn(earth);

    let giza = (
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(250.0, 250.0)),
                ..Default::default()
            },
            texture: asset_server.load("textures/giza.png"),
            transform: Transform::from_xyz(600.0, 0.0, 0.0),
            ..Default::default()
        },
        GravityProducer { strength: 200000.0 },
    );
    commands.spawn(giza);

    const ROCKET_SCALE: f32 = 80.0;
    const ASTRO_BABY_SCALE: f32 = 80.0;

    let rocket_engine_audio = audio
        .play(asset_server.load("sounds/rocket_engine.ogg"))
        .looped()
        .with_volume(0f64)
        .handle();

    let reddy_the_rocket = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(ROCKET_SCALE, ROCKET_SCALE)),
                    ..default()
                },
                texture: asset_server.load("textures/reddy_the_rocket.png"),
                transform: Transform::from_xyz(0.0, 300.0, -1.0),
                ..default()
            },
            PlayerController::new(6.0, 0.1, GamepadID::A, Some(rocket_engine_audio)),
            GravityEffected,
            CameraTrack(0.02),
        ))
        .id();

    let reddy_the_rocket_flame = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(ROCKET_SCALE, ROCKET_SCALE)),
                ..Default::default()
            },
            texture: asset_server.load("textures/reddy_the_rocket_flame.png"),
            transform: Transform::from_xyz(0.0, -ROCKET_SCALE, 0.0),
            ..Default::default()
        })
        .id();
    commands
        .entity(reddy_the_rocket)
        .push_children(&[reddy_the_rocket_flame]);

    let _astro_baby = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(ASTRO_BABY_SCALE, ASTRO_BABY_SCALE)),
                    ..default()
                },
                texture: asset_server.load("textures/astro_baby.png"),
                transform: Transform::from_xyz(0.0, -300.0, -1.0),
                ..default()
            },
            PlayerController::new(6.0, 0.1, GamepadID::B, None),
            GravityEffected,
        ))
        .id();

    let win = wins.single_mut();
    let star_texture = asset_server.load("textures/star.png");

    for _ in 0..100 {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..Default::default()
                },
                texture: star_texture.clone(),
                transform: Transform::from_xyz(
                    rand::random::<f32>() * win.resolution.width() * 2.0 - win.resolution.width(),
                    rand::random::<f32>() * win.resolution.height() * 2.0 - win.resolution.height(),
                    0.0,
                ),
                ..Default::default()
            },
            Spinner((rand::random::<f32>() - 0.5) * 0.01),
        ));
    }

    let _dua = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(120.0, 120.0)),
                ..Default::default()
            },
            texture: asset_server.load("textures/dua.png"),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        Spinner(-0.01),
    ));
}

#[derive(Component)]
struct PlayerController {
    vel: Vec3,
    acc: f32,
    rot: f32,
    thrust_on: bool,
    gamepad: GamepadID,
    sound: Option<Handle<AudioInstance>>,
}

impl PlayerController {
    fn new(acc: f32, rot: f32, gamepad: GamepadID, sound: Option<Handle<AudioInstance>>) -> Self {
        Self {
            vel: Vec3::new(0.0, 0.0, 0.0),
            acc,
            rot,
            thrust_on: false,
            gamepad,
            sound,
        }
    }
}

fn player_input(
    mut query: Query<(&mut Transform, &mut PlayerController)>,
    axes: Res<Axis<GamepadAxis>>,
    button_axes: Res<Axis<GamepadButton>>,
    gamepaddata: Res<GamepadData>,
    keys: Res<Input<KeyCode>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    for (mut t, mut pc) in query.iter_mut() {
        let gamepad = match pc.gamepad {
            GamepadID::A => gamepaddata.a,
            GamepadID::B => gamepaddata.b,
        };
        let controls: (f32, f32) = match gamepad {
            Some(gamepad) => player_gamepad_input(gamepad, &axes, &button_axes),
            None => player_kb_input(&keys),
        };

        t.rotate_z(-pc.rot * controls.0);

        let force = controls.1 * pc.acc * t.up();

        pc.thrust_on = force.length_squared() > 0.0;

        if let Some(sound) = &pc.sound {
            if let Some(sound) = audio_instances.get_mut(&sound) {
                sound.set_volume(
                    controls.1 as f64,
                    AudioTween::linear(Duration::new(0, 1000000)),
                );
            }
        }

        pc.vel += force;
    }
}

fn player_gamepad_input(
    gamepad: Gamepad,
    axes: &Res<Axis<GamepadAxis>>,
    button_axes: &Res<Axis<GamepadButton>>,
) -> (f32, f32) {
    let lsx = axes.get(GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::LeftStickX,
    });

    let lsy = axes.get(GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::LeftStickY,
    });

    let rt2 = button_axes.get(GamepadButton {
        gamepad,
        button_type: GamepadButtonType::RightTrigger2,
    });

    (
        lsx.map(|f| if f.abs() > 0.05 { f } else { 0.0 })
            .unwrap_or(0.0),
        rt2.map(|f| if f.abs() > 0.05 { f } else { 0.0 })
            .or(lsy.map(|f| if f.abs() > 0.05 { f } else { 0.0 }))
            .unwrap_or(0.0),
    )
}

fn player_kb_input(keys: &Res<Input<KeyCode>>) -> (f32, f32) {
    (
        (keys.pressed(KeyCode::D) as i32 - keys.pressed(KeyCode::A) as i32) as f32,
        keys.pressed(KeyCode::W) as i32 as f32,
    )
}

fn player_move(time: Res<Time>, mut query: Query<(&mut Transform, &PlayerController)>) {
    for (mut t, pc) in query.iter_mut() {
        t.translation += pc.vel * time.delta_seconds();
        t.translation.z = -1.0;
    }
}

#[derive(Component)]
struct GravityProducer {
    strength: f32,
}

impl GravityProducer {
    fn vec3_to_from_2d(to: &Vec3, from: &Vec3) -> Vec3 {
        Vec3::new(to.x - from.x, to.y - from.y, 0.0)
    }
}

#[derive(Component)]
struct GravityEffected;

fn gravity(
    mut effecteds: Query<(&Transform, &mut PlayerController), With<GravityEffected>>,
    produces: Query<(&Transform, &GravityProducer), Without<GravityEffected>>,
) {
    for (e_t, mut e_pc) in effecteds.iter_mut() {
        let mut force = Vec3::default();

        for (p_t, p_gp) in produces.iter() {
            let towards = GravityProducer::vec3_to_from_2d(&p_t.translation, &e_t.translation);

            let dforce = towards.length_squared().recip() * p_gp.strength * towards.normalize();

            // NOTE: Hard coded cap to stop excessive flinging.
            let dforce = dforce.clamp_length_max(60.0);

            force += dforce;
        }

        e_pc.vel += force;
    }
}

fn engine_flame(mut query: Query<(&Parent, &mut Visibility)>, p_query: Query<&PlayerController>) {
    for (p, mut vis) in query.iter_mut() {
        let Ok(pc) = p_query.get(**p) else {
            continue;
        };

        *vis = match pc.thrust_on {
            true => Visibility::Inherited,
            false => Visibility::Hidden,
        };
    }
}

fn gamepad_connections(
    mut gamepaddata: ResMut<GamepadData>,
    mut gamepad_evr: EventReader<bevy::input::gamepad::GamepadEvent>,
) {
    for ev in gamepad_evr.iter() {
        match ev {
            bevy::input::gamepad::GamepadEvent::Connection(conn_event) => {
                match &conn_event.connection {
                    bevy::input::gamepad::GamepadConnection::Connected(_info) => {
                        if gamepaddata.a.is_none() {
                            let _ = gamepaddata.a.insert(conn_event.gamepad);
                        } else if gamepaddata.b.is_none() {
                            let _ = gamepaddata.b.insert(conn_event.gamepad);
                        };
                    }
                    bevy::input::gamepad::GamepadConnection::Disconnected => {
                        gamepaddata.a = gamepaddata.a.filter(|g| g.id != conn_event.gamepad.id);
                        gamepaddata.b = gamepaddata.b.filter(|g| g.id != conn_event.gamepad.id);
                    }
                }
            }
            _ => (),
        }
    }
}

#[derive(Component)]
struct CameraTrack(f32);

#[derive(Component)]
struct MainCamera;

fn camera_track(
    object: Query<(&Transform, &CameraTrack), Without<MainCamera>>,
    mut cameras: Query<&mut Transform, With<MainCamera>>,
) {
    let object = object.single();

    for mut cam_t in cameras.iter_mut() {
        let (t, f) = object;
        let new_cam_t = {
            let mut t = cam_t.translation.lerp(t.translation, f.0);
            t.z = cam_t.translation.z;
            t
        };

        cam_t.translation = new_cam_t;
    }
}

#[derive(Component)]
struct Spinner(f32);

fn spinner(mut query: Query<(&mut Transform, &Spinner)>) {
    for (mut t, s) in query.iter_mut() {
        t.rotate_z(s.0);
    }
}
