use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::hierarchy::{BuildChildren, ChildBuild, ChildBuilder};
use bevy::prelude::{AlignItems, Button, JustifyContent, Node, Text, TextFont, Val};
use crate::ui::button::InteractiveButton;

pub mod button;
pub mod egui;

pub(super) struct UiPlugins;

impl PluginGroup for UiPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(button::ButtonPlugin)
    }
}

pub trait ChildBuilderExt {
    fn button(&mut self, text: &str, button_interactive: InteractiveButton);
}

impl ChildBuilderExt for ChildBuilder<'_> {
    fn button(&mut self, text: &str, button_interactive: InteractiveButton) {
        self.spawn((
            Button,
            Node {
                width: Val::Percent(80.0),
                height: Val::Px(60.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            button_interactive.background,
            button_interactive,
        )).with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: 24.0,
                    ..Default::default()
                }
            ));
        });
    }
}
