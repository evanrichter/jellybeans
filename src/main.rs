use std::f32::consts::PI;

use bevy::{
    color::palettes::css::*,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                key_input,
                stop_shape_mouse,
                draw_stopper,
                fps,
                score_color,
                (rotate, update_score).chain(),
            ),
        )
        .insert_resource(Stopper::Under(0))
        .run();
}

fn rotate(
    time: Res<Time>,
    stopper: Res<Stopper>,
    mut transforms: Query<(&mut Transform, &ShapeIndex), Without<ScoreText>>,
) {
    let d = time.delta_secs();

    for (mut t, i) in &mut transforms {
        // "stop" shapes that are lower than stopper
        let level = match *stopper {
            Stopper::Done => NUM_SHAPES as u8,
            Stopper::Under(n) => n,
        };
        if i.0 < level {
            continue;
        }

        let diff = 1.1_f32.powf(i.0 as f32) * 120.0 * d / 40.0;
        let diff = diff * if i.0 % 2 == 0 { -1.0 } else { 1.0 };
        t.rotate_z(diff);
    }
}

fn update_score(
    stopper: Res<Stopper>,
    transforms: Query<(&Transform, &ShapeIndex), With<Shape>>,
    mut full_score: Query<&mut TextSpan, With<ScoreText>>,
    mut ind_score: Query<(&mut Text2d, &ShapeIndex), With<ScoreText>>,
) {
    let ideal = Quat::from_rotation_z(PI / 2.0);
    let ideal2 = Quat::from_rotation_z(3.0 * PI / 2.0);

    let mut scores = [0.0; NUM_SHAPES];

    for (t, i) in &transforms {
        let angle1 = t.rotation.angle_between(ideal);
        let angle2 = t.rotation.angle_between(ideal2);
        let angle = f32::min(angle1, angle2) * 180.0 / PI;
        let angle = 90.0 - angle;
        scores[i.0 as usize] = angle * 1.1_f32.powf(i.0 as f32);
    }

    for mut span in &mut full_score {
        let score: f32 = scores.iter().sum();
        let score = score.round_ties_even();
        **span = format!("{score:.0}");
    }

    for (mut t, i) in &mut ind_score {
        if let Stopper::Under(s) = *stopper {
            if i.0 > s {
                continue;
            }
        }

        let score = scores[i.0 as usize] as u32;
        **t = format!("{score:2.0}");
    }
}

const X_EXTENT: f32 = 950.;
const NUM_SHAPES: usize = 10;

#[derive(Component)]
struct ShapeIndex(u8);

#[derive(Component)]
struct Shape;

#[derive(Component)]
struct Picker;

#[derive(Resource, PartialEq, Eq)]
enum Stopper {
    Done,
    Under(u8),
}

impl Stopper {
    fn next(&mut self) {
        const N: u8 = NUM_SHAPES as u8 - 1;
        *self = match self {
            Self::Done => Self::Done,
            Self::Under(N) => Self::Done,
            Self::Under(n) => Self::Under(*n + 1),
        };
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct FpsText;

fn key_input(
    mut stopper: ResMut<Stopper>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ind_score: Query<&mut Text2d, (With<ScoreText>, With<ShapeIndex>)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        stopper.next();
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        *stopper = Stopper::Under(0);

        for mut t in &mut ind_score {
            **t = String::default();
        }

        return;
    }
}

fn stop_shape_mouse(mut stopper: ResMut<Stopper>, mouse: Res<ButtonInput<MouseButton>>) {
    if mouse.just_pressed(MouseButton::Left) {
        stopper.next();
        return;
    }
}

fn draw_stopper(stopper: Res<Stopper>, mut transforms: Query<(&mut Transform, &Picker)>) {
    for (mut p, _) in &mut transforms {
        p.translation.x = match *stopper {
            Stopper::Done => 4000.0,
            Stopper::Under(i) => shape_pos_x(i),
        };
    }
}

fn shape_pos_x(i: u8) -> f32 {
    -X_EXTENT / 2. + i as f32 / (NUM_SHAPES - 1) as f32 * X_EXTENT
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let shape = meshes.add(Capsule2d::new(25.0, 50.0));

    for i in 0..NUM_SHAPES {
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / NUM_SHAPES as f32, 0.95, 0.7);

        commands.spawn((
            Mesh2d(shape.clone()),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(shape_pos_x(i as u8), 0.0, 0.0),
            ShapeIndex(i as u8),
            Shape,
        ));
    }

    // spawn the stopper indicator
    let color = Color::WHITE;
    commands.spawn((
        Mesh2d(meshes.add(Capsule2d::new(15.0, 45.0))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(shape_pos_x(0), -100.0, -10.0)
            .with_rotation(Quat::from_rotation_z(PI / 2.0)),
        Picker,
    ));

    commands.spawn((
        Text::new("Press space or click to a jelly bean!\nPress R to reset\n\nTry to line them all up as flat as you can!"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(X_EXTENT as f32),
            display: Display::Grid,
            grid_auto_flow: GridAutoFlow::Row,
            ..default()
        })
        .with_children(|p| {
            p.spawn(Text::new("Score: ")).with_child((
                TextColor(PLUM.into()),
                TextSpan::default(),
                ScoreText,
            ));
            p.spawn(Text::new("FPS: ")).with_child((
                TextColor(GOLD.into()),
                TextSpan::default(),
                FpsText,
            ));
        });

    for i in 0..NUM_SHAPES {
        commands.spawn((
            ScoreText,
            Text2d::default(),
            ShapeIndex(i as u8),
            Transform::from_xyz(shape_pos_x(i as u8), 80.0, 0.0),
        ));
    }
}

fn fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut TextSpan, With<FpsText>>) {
    for mut span in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                **span = format!("{value:.0}");
            }
        }
    }
}

fn score_color(
    stopper: Res<Stopper>,
    mut query: Query<&mut TextColor, (With<ScoreText>, Without<ShapeIndex>)>,
) {
    if matches!(*stopper, Stopper::Done) {
        for mut color in &mut query {
            **color = HOT_PINK.into();
        }
    }
}
