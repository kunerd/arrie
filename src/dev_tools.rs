use bevy::{
    dev_tools::ui_debug_overlay::{DebugUiPlugin, UiDebugOptions},
    input::common_conditions::input_just_pressed,
    prelude::*,
};

pub fn plugin(app: &mut App) {
    let toggle_system = toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY));

    // Toggle the debug overlay for UI.
    app.add_plugins(DebugUiPlugin);
    app.add_systems(Update, toggle_system);
}

const TOGGLE_KEY: KeyCode = KeyCode::KeyD;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
