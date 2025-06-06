use bevy::app::{App, Plugin, Update};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::input::ButtonInput;
use bevy::math::{Rect, Vec3};
use bevy::prelude::{Camera, Camera2d, MouseButton, Single};
use bevy::prelude::{Component, OrthographicProjection, Query, Res, Transform, With};
use bevy::window::{PrimaryWindow, Window};

pub trait CameraExt {
    fn has_cursor(&self, window: &Window) -> bool;
}

impl CameraExt for Camera {
    fn has_cursor(&self, window: &Window) -> bool {
        let Some(viewport_rect) = self.physical_viewport_rect() else { return false; };
        let Some(cursor_pos) = window.physical_cursor_position() else { return false; };
        viewport_rect.contains(cursor_pos.as_uvec2())
    }
}

#[derive(Component, Debug, Default)]
#[require(Transform, Camera2d)]
pub struct ControllableCamera2d {
    panning: bool,
    pub bounds: Rect,
}

impl ControllableCamera2d {
    pub fn new(bounds: Rect) -> Self {
        Self { bounds, ..Default::default() }
    }
}

pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, controls);
    }
}

fn controls(
    mut cameras: Query<(&mut Transform, &Camera, &mut ControllableCamera2d, &mut OrthographicProjection)>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    for (mut transform, camera, mut controls, mut projection) in &mut cameras {
        if mouse.pressed(MouseButton::Left) && (controls.panning || mouse.just_pressed(MouseButton::Left) && camera.has_cursor(&window)) {
            controls.panning = true;
            transform.translation.x -= motion.delta.x * projection.scale;
            transform.translation.y += motion.delta.y * projection.scale;
        } else {
            controls.panning = false;
        }
        if scroll.delta.y != 0. && camera.has_cursor(&window) {
            let scale = (projection.scale * (1. + -scroll.delta.y.clamp(-3., 3.)/4.)).clamp(0.001, 0.1);
            let factor = scale / projection.scale;
            projection.scale = scale;
            let cursor = window.cursor_position().unwrap_or_default() - camera.logical_viewport_rect().unwrap_or_default().center();
            transform.translation += (cursor - cursor * factor).extend(0.) * projection.scale/factor * Vec3::new(1., -1., 1.);
        }
        transform.translation.x = transform.translation.x.clamp(controls.bounds.min.x, controls.bounds.max.x);
        transform.translation.y = transform.translation.y.clamp(controls.bounds.min.y, controls.bounds.max.y);
    }
}
