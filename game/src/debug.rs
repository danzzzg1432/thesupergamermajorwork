use bevy::app::{App, Plugin, Startup, Update};
use bevy::color::{Color, Srgba};
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::math::{Isometry2d, Vec2};
use bevy::prelude::{in_state, AppExtStates, Camera, Commands, Component, EventReader, Gizmos, GlobalTransform, IntoSystemConfigs, KeyCode, NextState, OrthographicProjection, Query, Res, ResMut, Single, State, StateTransitionEvent, States, Text, Transform, Window, With, Node, Val};
use bevy::ui::PositionType;
use bevy::window::PrimaryWindow;
use crate::camera::{CameraExt, ControllableCamera2d};
use std::marker::ConstParamTy;

pub(crate) struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<DebugState>()
            .add_systems(Startup, DebugState::init)
            .add_systems(Update, DebugState::update_state)
            .add_systems(Update, DebugState::update_text)
            .add_systems(Update, transition)
            .add_systems(Update, grid_renderer.run_if(in_state(DebugState::Grid)));
    }
}

#[derive(Component, Debug)]
struct DebugTextLabel;

macro_rules! state {
    {$($(#[$attr:meta])* $key:ident => $var:ident),* $(,)?} => {
        #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, ConstParamTy)]
        enum DebugState {
           $($(#[$attr])* $var,)*
        }
        
        impl DebugState {
            fn init(
                mut commands: Commands,
            ) {
                commands.spawn((
                    DebugTextLabel,
                    Text::default(),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(12.),
                        left: Val::Px(12.),
                        ..Default::default()
                    },
                ));
                $(commands.spawn(DebugText::<{ Self::$var }>::default());)*
            }
            
            #[allow(non_snake_case)]
            fn update_text(
                mut label: Single<&mut Text, With<DebugTextLabel>>,
                state: Res<State<DebugState>>,
                $($var: Single<&DebugText::<{ Self::$var }>>,)*
            ) {
                label.0 = match **state { $(Self::$var => $var.0.clone(),)* };
            }

            fn update_state(
                mut next_state: ResMut<NextState<DebugState>>,
                keys: Res<ButtonInput<KeyCode>>,
            ) {
                $(
                    if keys.just_pressed(KeyCode::$key) {
                        next_state.set(Self::$var);
                    }
                )else*
            }
        }
    };
}

state! {
    #[default]
    F1 => Disabled,
    F2 => Grid,
}

#[derive(Component, Debug, Default)]
struct DebugText<const S: DebugState>(String);

fn transition(
    mut events: EventReader<StateTransitionEvent<DebugState>>,
) {
    for transition in events.read() {
        if let Some(state) = &transition.entered {
            info!("{state:?} renderer activated");
        }
    }
}

fn grid_renderer(
    mut gizmos: Gizmos,
    cameras: Query<(&Camera, &OrthographicProjection, &Transform, &GlobalTransform, Option<&ControllableCamera2d>)>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut text: Single<&mut DebugText<{ DebugState::Grid }>>,
) {
    for (camera, projection, transform, global_transform, controls) in cameras.iter() {
        if projection.scale < 0.5 {
            gizmos.grid_2d(
                Isometry2d::from_translation(transform.translation.round().truncate()),
                projection.area.size().as_uvec2() / 2 * 2 + 3,
                Vec2::splat(1.),
                Color::srgb(0.75, 0.75, 0.75),
            );
        }
        if let Some(controls) = controls {
            gizmos.rect_2d(
                Isometry2d::from_translation(controls.bounds.center()),
                controls.bounds.size(),
                Srgba::rgb(1., 0., 0.),
            );
        }
        if camera.has_cursor(&window) {
            let pos = camera.viewport_to_world_2d(global_transform, window.cursor_position().unwrap_or_default() - camera.logical_viewport_rect().unwrap_or_default().min).unwrap_or_default().round();
            gizmos.rect_2d(
                Isometry2d::from_translation(pos),
                Vec2::splat(1.),
                Srgba::rgb(0., 1., 0.),
            );
            text.0 = format!("Cursor: X={} Y={}", pos.x as i32, pos.y as i32);
        } else {
            text.0 = "Cursor: Not in viewport".into();
        }
    }
}
