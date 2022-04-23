use bevy::prelude::*;

#[derive(Component, Default)]
pub struct MousePosition(pub Vec2);

fn cursor_position_to_model_2d(window: &Window, position: Vec2) -> Vec2 {
    Vec2::new(
        position.x - 0.5 * window.width(),
        position.y - 0.5 * window.height(),
    )
}

fn update_mouse_position(
    windows: Res<Windows>,
    mut mouse_position: ResMut<MousePosition>,
    mut cursor_moved_events: EventReader<CursorMoved>,
) {
    if let Some(cursor_moved) = cursor_moved_events.iter().last().take() {
        mouse_position.0 =
            cursor_position_to_model_2d(windows.get_primary().unwrap(), cursor_moved.position);
    }
}

pub struct SimpleMouse;
impl Plugin for SimpleMouse {
    fn build(&self, app: &mut App) {
        app.init_resource::<MousePosition>()
            .add_system_to_stage(CoreStage::PreUpdate, update_mouse_position);
    }
}
