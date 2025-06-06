use crate::game::logging::Log;
use bevy::math::UVec2;
use bevy::prelude::{Camera, NextState, Res, ResMut, Resource, Single, State, With};
use bevy::render::camera::Viewport;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::{egui, EguiContexts};
use bevy_egui::egui::{Frame, Margin, ScrollArea, SidePanel, TopBottomPanel};
use egui_extras::syntax_highlighting;
use egui_extras::syntax_highlighting::code_view_ui;
use crate::game::execution::execution_state::ExecutionState;
use crate::game::execution::run::{reset_tick, STEPPER};
use crate::scenes::Scene;
use crate::ui::egui::id;

#[derive(Resource, Default)]
pub(super) struct Code(pub(crate) String);

#[allow(clippy::cast_sign_loss)]
pub(super) fn render(
    mut contexts: EguiContexts,
    mut code: ResMut<Code>,
    log: Res<Log>,
    execution: Res<State<ExecutionState>>,
    mut next_execution: ResMut<NextState<ExecutionState>>,
    mut next_scene: ResMut<NextState<Scene>>,
    mut camera: Single<&mut Camera>,
    window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let bottom = if execution.show_console() {
        TopBottomPanel::bottom(id!())
            .resizable(true)
            .show(contexts.ctx_mut(), |ui| {
                Frame::canvas(ui.style())
                    .show(ui, |ui| {
                        ScrollArea::both()
                            .stick_to_bottom(true)
                            .max_width(f32::INFINITY)
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                code_view_ui(
                                    ui,
                                    &syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style()),
                                    &log.0,
                                    "",
                                )
                            });
                    });
            }).response.rect.height() * window.scale_factor()
    } else { 0. };
    let left = SidePanel::left(id!())
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                if execution.can_run() && ui.button("Run").clicked() {
                    reset_tick();
                    next_execution.set(ExecutionState::Running);
                    STEPPER.wake();
                }
                if *execution == ExecutionState::Stopped && ui.button("Step").clicked() {
                    next_execution.set(ExecutionState::Stepping);
                    STEPPER.skip();
                    STEPPER.wake();
                } else if *execution == ExecutionState::Stepping {
                    if !STEPPER.is_waiting() {
                        ui.add_enabled_ui(false, |ui| ui.button("Step"));
                    } else if ui.button("Step").clicked() {
                        STEPPER.wake();
                    }
                }
                if execution.can_exit() && ui.button("Exit").clicked() {
                    next_scene.set(Scene::MainMenu);
                }
                if execution.can_stop() && ui.button("Stop").clicked() {
                    if *execution == ExecutionState::Finished {
                        next_execution.set(ExecutionState::Stopped);
                    } else {
                        next_execution.set(ExecutionState::Stopping);
                    }
                }
                if !execution.interactive() && execution.shutdown() {
                    ui.add_enabled_ui(false, |ui| ui.button("Stopping…"));
                }
            });
            Frame::canvas(ui.style())
                .show(ui, |ui| {
                    ScrollArea::both().show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut code.0)
                            .interactive(execution.interactive())
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_rows(10)
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .frame(false)
                            .margin(Margin::same(0))
                            .layouter(&mut |ui: &egui::Ui, string, _wrap_width| {
                                let mut layout_job = syntax_highlighting::highlight(
                                    ui.ctx(),
                                    ui.style(),
                                    &syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style()),
                                    string,
                                    "py",
                                );
                                layout_job.wrap.max_width = f32::INFINITY;
                                ui.fonts(|f| f.layout_job(layout_job))
                            })
                        );
                    });
                });
        }).response.rect.width() * window.scale_factor();
    let physical_position = UVec2::new(left as u32, 0).min(window.physical_size() - UVec2::splat(1));
    let physical_size = (window.physical_size() - physical_position - UVec2::new(0, bottom as u32)).max(UVec2::splat(1));
    camera.viewport = Some(Viewport { physical_position, physical_size, ..Default::default() });
}
