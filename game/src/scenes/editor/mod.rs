mod ui;

use crate::game::execution::run::run;
use std::fs;
use crate::game::level;
use crate::scenes::Scene;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{in_state, Commands, EventReader, IntoSystemConfigs, NextState, OnEnter, OnExit, Res, ResMut, State};
use bevy::window::FileDragAndDrop;
use crate::game::execution::execution_state::ExecutionState;
use crate::game::level::{reset, CurrentLevel};

pub(super) struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ui::Code>()
            .add_systems(Update, ui::render.run_if(in_state(Scene::Editor)))
            .add_systems(Update, file_drop)
            .add_systems(OnEnter(Scene::Editor), level::spawn)
            .add_systems(OnExit(Scene::Editor), level::despawn)
            .add_systems(OnEnter(ExecutionState::Stopped), reset.run_if(in_state(Scene::Editor)))
            .add_systems(OnExit(ExecutionState::Stopped), |mut commands: Commands, code: Res<ui::Code>| commands.run_system_cached_with(run, code.0.clone()));
    }
}

fn file_drop(
    mut commands: Commands,
    mut events: EventReader<FileDragAndDrop>,
    mut next_scene: ResMut<NextState<Scene>>,
    mut current_level: ResMut<CurrentLevel>,
    mut code: ResMut<ui::Code>,
    mut next_execution: ResMut<NextState<ExecutionState>>,
    scene: Res<State<Scene>>,
) {
    for event in events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            let level = level::parse(&fs::read_to_string(path_buf).unwrap());
            *current_level = CurrentLevel(level);
            if *scene == Scene::Editor {
                code.0.clear();
                commands.run_system_cached(level::reset);
                next_execution.set(ExecutionState::Stopped);
            } else {
                next_scene.set(Scene::Editor);
            }
        }
    }
}
