use std::f32::consts::FRAC_PI_2;

use bevy::{
    color::palettes::tailwind::*,
    input::mouse::AccumulatedMouseMotion,
    picking::pointer::PointerInteraction,
    prelude::*,
    scene::{SceneInstance, SceneInstanceReady},
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_inspector_egui::{bevy_egui::EguiContext, quick::WorldInspectorPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Chess".to_string(),
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }));

        app.register_type::<Board>();

        app.add_plugins((MeshPickingPlugin, WorldInspectorPlugin::default()));

        app.add_systems(Startup, (setup_camera, setup_lights, setup_board));
        app.add_systems(Update, (/*move_camera,*/draw_mesh_intersections));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 1.5, 1.5))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
}

fn setup_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Board;

fn setup_board(mut commands: Commands, asset_server: Res<AssetServer>) {
    let board = SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("ChessBoard.glb")));

    commands
        .spawn((board, Name::new("Board"), Board))
        .observe(on_board_added);
}

fn on_board_added(
    trigger: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh3ds: Query<&Mesh3d>,
) {
    // Set up the materials.
    let white_matl = materials.add(Color::WHITE);
    let ground_matl = materials.add(Color::from(GRAY_300));
    let hover_matl = materials.add(Color::from(CYAN_300));
    let pressed_matl = materials.add(Color::from(YELLOW_300));

    let entity = trigger.entity();
    for child in children.iter_descendants(entity) {
        if mesh3ds.get(child).is_ok() {
            commands
                .entity(child)
                // .observe(update_material_on::<Pointer<Over>>(hover_matl.clone()))
                // .observe(update_material_on::<Pointer<Out>>(white_matl.clone()))
                // .observe(update_material_on::<Pointer<Down>>(pressed_matl.clone()))
                // .observe(update_material_on::<Pointer<Up>>(hover_matl.clone()))
                .observe(rotate_on_drag);
        }
    }
}

/// Returns an observer that updates the entity's material to the one specified.
fn update_material_on<E>(
    new_material: Handle<StandardMaterial>,
) -> impl Fn(Trigger<E>, Query<&mut MeshMaterial3d<StandardMaterial>>) {
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |trigger, mut query| {
        if let Ok(mut material) = query.get_mut(trigger.entity()) {
            material.0 = new_material.clone();
        }
    }
}

/// A system that draws hit indicators for every pointer.
fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }
}

/// An observer to rotate an entity when it is dragged
fn rotate_on_drag(drag: Trigger<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    let mut transform = transforms.get_mut(drag.entity()).unwrap();
    transform.rotate_y(drag.delta.x * 0.02);
}

fn move_camera(
    mouse_input: Res<AccumulatedMouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut egui_context: Query<&mut EguiContext>,
) {
    let mut egui_context = egui_context.get_single_mut().unwrap();

    if mouse_button_input.just_released(MouseButton::Middle) {
        if let Ok(mut window) = window.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }

    if mouse_button_input.just_pressed(MouseButton::Middle)
        && !egui_context.get_mut().wants_pointer_input()
    {
        if let Ok(mut window) = window.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        }
    }

    if let Ok(window) = window.get_single_mut() {
        if window.cursor_options.grab_mode != CursorGrabMode::Locked {
            return;
        }
    }

    for mut transform in &mut camera {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            info!("W pressed");
            direction += *transform.forward();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            info!("S pressed");
            direction -= *transform.forward();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            info!("A pressed");
            direction -= *transform.right();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            info!("D pressed");
            direction += *transform.right();
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            info!("E pressed");
            direction += Vec3::Y;
        }
        if keyboard_input.pressed(KeyCode::KeyQ) {
            info!("Q pressed");
            direction -= Vec3::Y;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * 5.0 * time.delta_secs();
        }

        let delta = mouse_input.delta;
        if delta != Vec2::ZERO {
            // Note that we are not multiplying by delta_time here.
            // The reason is that for mouse movement, we already get the full movement that happened since the last frame.
            // This means that if we multiply by delta_time, we will get a smaller rotation than intended by the user.
            // This situation is reversed when reading e.g. analog input from a gamepad however, where the same rules
            // as for keyboard input apply. Such an input should be multiplied by delta_time to get the intended rotation
            // independent of the framerate.
            let delta_yaw = -delta.x * 0.003;
            let delta_pitch = -delta.y * 0.002;

            let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
            let yaw = yaw + delta_yaw;

            // If the pitch was ±¹⁄₂ π, the camera would look straight up or down.
            // When the user wants to move the camera back to the horizon, which way should the camera face?
            // The camera has no way of knowing what direction was "forward" before landing in that extreme position,
            // so the direction picked will for all intents and purposes be arbitrary.
            // Another issue is that for mathematical reasons, the yaw will effectively be flipped when the pitch is at the extremes.
            // To not run into these issues, we clamp the pitch to a safe range.
            const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
            let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        }
    }
}
