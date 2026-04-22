use egui::{Context, SidePanel, CentralPanel, TextEdit, ScrollArea, Color32, RichText};

pub struct Gui {
    script_content: String,
    error_message: Option<String>,
    show_stats: bool,
    entity_count: usize,
    vertex_count: usize,
    triangle_count: usize,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            script_content: String::from(
                r#"let base = cube(width: 10.0, height: 2.0, depth: 10.0)
spawn(base)

let tower = cylinder(radius: 1.0, height: 5.0, segments: 16)
spawn(tower)

let top = sphere(radius: 1.5, segments: 24, rings: 12)
spawn(top)"#,
            ),
            error_message: None,
            show_stats: true,
            entity_count: 0,
            vertex_count: 0,
            triangle_count: 0,
        }
    }

    pub fn update(&mut self, ctx: &Context) -> Option<GuiEvent> {
        let mut event = None;

        SidePanel::left("editor_panel")
            .default_width(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading(RichText::new("VX Script Editor").size(18.0));
                ui.separator();

                ui.label("Write VX code to create 3D geometry:");
                ui.add_space(5.0);

                ScrollArea::vertical().show(ui, |ui| {
                    let response = ui.add(
                        TextEdit::multiline(&mut self.script_content)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(25)
                            .font(egui::TextStyle::Monospace),
                    );

                    if response.changed() {
                        self.error_message = None;
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button(RichText::new("▶ Execute Script").size(14.0)).clicked() {
                        event = Some(GuiEvent::ExecuteScript(self.script_content.clone()));
                    }

                    if ui.button(RichText::new("🗑 Clear Scene").size(14.0)).clicked() {
                        event = Some(GuiEvent::ClearScene);
                    }
                });

                if let Some(error) = &self.error_message {
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(255, 100, 100), RichText::new("Error:").strong());
                    ui.label(RichText::new(error).color(Color32::from_rgb(255, 150, 150)));
                }

                ui.separator();
                ui.checkbox(&mut self.show_stats, "Show Statistics");

                if self.show_stats {
                    ui.separator();
                    ui.group(|ui| {
                        ui.label(RichText::new("Scene Statistics").strong());
                        ui.label(format!("Entities: {}", self.entity_count));
                        ui.label(format!("Vertices: {}", self.vertex_count));
                        ui.label(format!("Triangles: {}", self.triangle_count));
                    });
                }

                ui.separator();
                ui.collapsing("VX Language Reference", |ui| {
                    ui.label(RichText::new("Primitives:").strong());
                    ui.label("  cube(width, height, depth)");
                    ui.label("  sphere(radius, segments, rings)");
                    ui.label("  cylinder(radius, height, segments)");
                    ui.label("  plane(width, depth)");
                    ui.add_space(5.0);
                    ui.label(RichText::new("Commands:").strong());
                    ui.label("  spawn(object) - Add to scene");
                    ui.add_space(5.0);
                    ui.label(RichText::new("Example:").strong());
                    ui.label("  let box = cube(width: 2.0, height: 2.0, depth: 2.0)");
                    ui.label("  spawn(box)");
                });
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(RichText::new("3D Viewport").size(20.0));
            });
            ui.separator();
            
            ui.group(|ui| {
                ui.label(RichText::new("Camera Controls:").strong());
                ui.label("  🖱 Left Mouse: Orbit camera");
                ui.label("  🖱 Middle Mouse: Pan camera");
                ui.label("  🖱 Scroll Wheel: Zoom in/out");
            });

            ui.add_space(20.0);
            
            if self.entity_count == 0 {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No geometry in scene").size(16.0).color(Color32::GRAY));
                    ui.label("Execute a script to create 3D objects");
                });
            }
        });

        event
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn update_stats(&mut self, entity_count: usize, vertex_count: usize, triangle_count: usize) {
        self.entity_count = entity_count;
        self.vertex_count = vertex_count;
        self.triangle_count = triangle_count;
    }

    pub fn get_script(&self) -> &str {
        &self.script_content
    }
}

pub enum GuiEvent {
    ExecuteScript(String),
    ClearScene,
}
