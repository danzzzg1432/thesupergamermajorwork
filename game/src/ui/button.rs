use bevy::app::{App, Plugin, Update};
use bevy::ecs::system::SystemId;
use bevy::prelude::{Button, Changed, Commands, Component, Query, With};
use bevy::ui::{BackgroundColor, Interaction};

pub(super) struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

#[derive(Component)]
pub struct InteractiveButton {
    pub on_click: SystemId,
    pub background: BackgroundColor,
    pub background_hover: BackgroundColor,
    pub background_click: BackgroundColor,
}

pub(super) fn update(
    mut commands: Commands,
    mut query: Query<(&Interaction, &InteractiveButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, button, mut background) in &mut query {
        match interaction {
            Interaction::Pressed => {
                *background = button.background_click;
                commands.run_system(button.on_click);
            }
            Interaction::Hovered => {
                *background = button.background_hover;
            }
            Interaction::None => {
                *background = button.background;
            }
        }
    }
}
