use bevy::{prelude::*, render::camera::Projection};
use leafwing_input_manager::prelude::*;
use std::fmt::Debug;

#[derive(Component)]
pub struct DebugCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub move_sens: f32,
    pub look_sens: f32,
    pub zoom_sens: f32,
    pub upside_down: bool,
}

impl Default for DebugCamera {
    fn default() -> Self {
        DebugCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            move_sens: 0.005,
            look_sens: 0.005,
            zoom_sens: 0.1,
            upside_down: false,
        }
    }
}

pub struct DebugCameraPlugin;
impl Plugin for DebugCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera)
            .add_state(CameraState::FreeFloat)
            .add_plugin(InputManagerPlugin::<CameraAction>::default())
            .add_plugin(InputManagerPlugin::<CameraMovement>::default())
            .add_system(update_camera_state)
            .add_system_set(
                SystemSet::on_update(CameraState::Locked).with_system(update_camera_pos),
            )
            .add_system_set(SystemSet::on_update(CameraState::Fps).with_system(update_camera_rot))
            .add_system_set(
                SystemSet::on_update(CameraState::Editor)
                    .with_system(update_camera_rot)
                    .with_system(update_camera_pos)
                    .with_system(update_camera_pan),
            )
            .add_system_set(
                SystemSet::on_update(CameraState::FreeFloat)
                    .with_system(update_camera_zoom)
                    .with_system(update_camera_pos)
                    .with_system(update_camera_rot)
                    .with_system(update_camera_pan),
            );
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraState {
    FreeFloat, // Tranlation, Rotation
    Locked,    // Transltaion only
    Fps,       // Rotation only
    Editor,    // Trigger to move
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq)]
pub enum CameraMovement {
    Left,
    Right,
    Back,
    Forward,
    Up,
    Down,
}

impl CameraMovement {
    pub fn into_vec(self) -> Vec3 {
        match self {
            CameraMovement::Up => Vec3::Y,
            CameraMovement::Down => Vec3::NEG_Y,
            CameraMovement::Right => Vec3::X,
            CameraMovement::Left => Vec3::NEG_X,
            CameraMovement::Back => Vec3::Z,
            CameraMovement::Forward => Vec3::NEG_Z,
        }
    }
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq)]
pub enum CameraAction {
    Rotate,
    MoveTrigger,
    Pan,
    PanTrigger,
    Zoom,
    SensTrigger,
    FreeFloatToggle,
}

fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(-2.0, 2.5, 5.0);
    let radius = translation.length();

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(DebugCamera {
            radius,
            ..Default::default()
        })
        .insert(InputManagerBundle::<CameraAction> {
            input_map: InputMap::default()
                .insert(DualAxis::mouse_motion(), CameraAction::Pan)
                .insert(DualAxis::mouse_wheel(), CameraAction::Zoom)
                .insert(MouseButton::Right, CameraAction::MoveTrigger)
                .insert(MouseButton::Middle, CameraAction::PanTrigger)
                .insert(KeyCode::LShift, CameraAction::SensTrigger)
                .insert(KeyCode::C, CameraAction::FreeFloatToggle)
                .build(),
            action_state: ActionState::default(),
        })
        .insert(InputManagerBundle::<CameraMovement> {
            input_map: InputMap::default()
                .insert(KeyCode::W, CameraMovement::Forward)
                .insert(KeyCode::A, CameraMovement::Left)
                .insert(KeyCode::S, CameraMovement::Back)
                .insert(KeyCode::D, CameraMovement::Right)
                .insert(KeyCode::Space, CameraMovement::Up)
                .insert(KeyCode::LControl, CameraMovement::Down)
                .build(),
            action_state: ActionState::default(),
        });
}

fn update_camera_state(
    mut q: Query<(&mut DebugCamera, &ActionState<CameraAction>)>,
    mut state: ResMut<State<CameraState>>,
) {
    let (mut camera, actions) = q.single_mut();

    if actions.just_pressed(CameraAction::SensTrigger) {
        camera.move_sens *= 5.0;
    };

    if actions.just_released(CameraAction::SensTrigger) {
        camera.move_sens *= 0.2;
    };

    if actions.just_pressed(CameraAction::FreeFloatToggle) {
        match *state.current() {
            CameraState::FreeFloat => state.set(CameraState::Editor).unwrap(),
            _ => state.set(CameraState::FreeFloat).unwrap(),
        };
    };
}

fn update_camera_pan(mut q: Query<(&mut Transform, &DebugCamera, &ActionState<CameraAction>)>) {
    let (mut transform, camera, actions) = q.single_mut();
    let pan = actions.axis_pair(CameraAction::Pan).unwrap();

    if actions.pressed(CameraAction::PanTrigger) {
        let dx = transform.rotation * Vec3::X * camera.move_sens * pan.x();
        let dy = transform.rotation * Vec3::Y * camera.move_sens * pan.y();
        transform.translation = transform.translation - dx + dy;
    }
}

fn update_camera_zoom(mut q: Query<(&mut Projection, &DebugCamera, &ActionState<CameraAction>)>) {
    let (mut projection, camera, actions) = q.single_mut();
    let zoom = actions.axis_pair(CameraAction::Zoom).unwrap();
    if zoom.length_squared() == 0.0 {
        return;
    }

    if let Projection::Perspective(projection) = projection.as_mut() {
        projection.fov += -zoom.y() * camera.look_sens;
    }
}

fn update_camera_rot(
    mut q: Query<(&mut Transform, &DebugCamera, &ActionState<CameraAction>)>,
    state: Res<State<CameraState>>,
) {
    let (mut transform, camera, actions) = q.single_mut();
    let motion = actions.axis_pair(CameraAction::Pan).unwrap();
    let triggered = actions.pressed(CameraAction::MoveTrigger);

    if *state.current() == CameraState::FreeFloat || triggered {
        transform.rotation =
            Quat::from_rotation_y(-motion.x() * camera.look_sens) * transform.rotation;
        transform.rotation *= Quat::from_rotation_x(-motion.y() * camera.look_sens);
    }
}

fn update_camera_pos(
    mut q: Query<(
        &mut Transform,
        &DebugCamera,
        &ActionState<CameraMovement>,
        &ActionState<CameraAction>,
    )>,
    state: Res<State<CameraState>>,
) {
    let (mut transform, camera, movement, actions) = q.single_mut();
    let triggered = actions.pressed(CameraAction::MoveTrigger);

    if (*state.current() == CameraState::FreeFloat) || triggered {
        movement.get_pressed().iter().for_each(|movement| {
            let mut direction = movement.into_vec();

            // Apply up and down movements on global axis
            if *movement != CameraMovement::Up && *movement != CameraMovement::Down {
                direction = transform.rotation * direction;
            }

            transform.translation += direction * camera.move_sens;
        });
    }
}
