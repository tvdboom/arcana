use bevy_egui::egui::{self, Color32, CornerRadius, Stroke, style::WidgetVisuals};

pub fn apply_custom_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Stone palette (less futuristic)
    let bg_color = Color32::from_rgb(34, 31, 28);
    let panel_bg = Color32::from_rgba_unmultiplied(58, 52, 45, 230);
    let primary_stone = Color32::from_rgb(164, 149, 122);
    let secondary_stone = Color32::from_rgb(124, 111, 92);
    let text_light = Color32::from_rgb(231, 224, 210);
    let text_dim = Color32::from_rgb(184, 174, 156);
    let border_color = Color32::from_rgb(86, 76, 63);

    style.visuals.window_corner_radius = CornerRadius::same(6);
    style.visuals.window_fill = panel_bg;
    style.visuals.window_stroke = Stroke::new(2.0, secondary_stone);
    style.visuals.panel_fill = bg_color;

    // Modify widget visuals (buttons, sliders, etc.)
    let set_widget_visuals = |visuals: &mut WidgetVisuals, fill: Color32, stroke: Color32, text: Color32| {
        visuals.bg_fill = fill;
        visuals.bg_stroke = Stroke::new(1.0, stroke);
        visuals.fg_stroke = Stroke::new(1.0, text);
        visuals.corner_radius = CornerRadius::same(3);
    };

    // Non-interactive widgets
    set_widget_visuals(&mut style.visuals.widgets.noninteractive, bg_color, border_color, text_dim);
    
    // Inactive widgets (default button state)
    set_widget_visuals(
        &mut style.visuals.widgets.inactive,
        Color32::from_rgb(72, 64, 54),
        border_color,
        text_light
    );

    // Hovered widgets
    set_widget_visuals(
        &mut style.visuals.widgets.hovered,
        Color32::from_rgb(86, 76, 63),
        primary_stone,
        text_light
    );

    // Active (clicked) widgets
    set_widget_visuals(
        &mut style.visuals.widgets.active,
        Color32::from_rgb(98, 88, 73),
        primary_stone,
        Color32::from_rgb(248, 242, 230)
    );

    // Open state
    style.visuals.widgets.open.bg_fill = Color32::from_rgb(66, 58, 49);
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, border_color);

    // Selection color
    style.visuals.selection.bg_fill = Color32::from_rgb(116, 103, 83);
    style.visuals.selection.stroke = Stroke::new(1.0, text_light);

    ctx.set_style(style);
}
