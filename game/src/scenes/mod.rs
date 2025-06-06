use bevy::app::{App, Plugin};
use bevy::prelude::{AppExtStates, States};

mod main_menu;
mod editor;

pub(super) struct ScenesPlugin;

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Scene>()
            .enable_state_scoped_entities::<Scene>()
            .add_plugins(main_menu::MainMenuPlugin)
            .add_plugins(editor::EditorPlugin);
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scene {
    #[default]
    MainMenu,
    Editor,
}
