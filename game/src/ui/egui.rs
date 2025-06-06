macro_rules! id {
    () => {
        bevy_egui::egui::Id::new(format!("{}:{}:{}", file!(), line!(), column!()))
    };
}

pub(crate) use id;
