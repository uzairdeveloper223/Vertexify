mod math;
mod scene;
mod vx;
mod renderer;
mod gui;

use scene::Scene;
use renderer::Renderer;
use gui::{Gui, GuiEvent};
use winit::{
    event::*,
    event_loop::EventLoop,
    window::WindowBuilder,
};
use wgpu::util::DeviceExt;
use std::sync::Arc;

struct App {
    scene: Scene,
    gui: Gui,
    mouse_pressed: bool,
    middle_mouse_pressed: bool,
    last_mouse_pos: Option<(f32, f32)>,
}

impl App {
    fn new() -> Self {
        let mut scene = Scene::new();
        let gui = Gui::new();

        let script = gui.get_script();
        if let Err(e) = vx::execute_script(script, &mut scene) {
            eprintln!("Initial script error: {}", e);
        }

        Self {
            scene,
            gui,
            mouse_pressed: false,
            middle_mouse_pressed: false,
            last_mouse_pos: None,
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent, renderer: &mut Renderer, egui_consumed: bool) -> bool {
        if egui_consumed {
            return true;
        }

        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                match button {
                    MouseButton::Left => {
                        self.mouse_pressed = *state == ElementState::Pressed;
                        if *state == ElementState::Released {
                            self.last_mouse_pos = None;
                        }
                    }
                    MouseButton::Middle => {
                        self.middle_mouse_pressed = *state == ElementState::Pressed;
                        if *state == ElementState::Released {
                            self.last_mouse_pos = None;
                        }
                    }
                    _ => {}
                }
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                let current_pos = (position.x as f32, position.y as f32);

                if let Some(last_pos) = self.last_mouse_pos {
                    let delta_x = (current_pos.0 - last_pos.0) * 0.005;
                    let delta_y = (current_pos.1 - last_pos.1) * 0.005;

                    if self.mouse_pressed {
                        renderer.camera.orbit(delta_x, -delta_y);
                    } else if self.middle_mouse_pressed {
                        renderer.camera.pan(-delta_x * 2.0, delta_y * 2.0);
                    }
                }

                self.last_mouse_pos = Some(current_pos);
                false
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y * 0.5,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                };
                renderer.camera.zoom(scroll_delta);
                true
            }
            _ => false,
        }
    }

    fn handle_gui_event(&mut self, event: GuiEvent) {
        match event {
            GuiEvent::ExecuteScript(script) => {
                self.scene.clear();
                match vx::execute_script(&script, &mut self.scene) {
                    Ok(_) => {
                        self.gui.clear_error();
                        println!("Script executed: {} entities created", self.scene.entities().count());
                    }
                    Err(e) => {
                        self.gui.set_error(e);
                    }
                }
            }
            GuiEvent::ClearScene => {
                self.scene.clear();
                self.gui.clear_error();
                println!("Scene cleared");
            }
        }
    }

    fn update_stats(&mut self) {
        let mut vertex_count = 0;
        let mut triangle_count = 0;
        for entity in self.scene.entities() {
            if let Some(mesh) = &entity.mesh {
                vertex_count += mesh.vertices.len();
                triangle_count += mesh.indices.len() / 3;
            }
        }
        self.gui.update_stats(
            self.scene.entities().count(),
            vertex_count,
            triangle_count,
        );
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Vertexify - 3D Modeling with VX Script")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap());

    let mut app = App::new();
    let mut renderer = pollster::block_on(Renderer::new(window.clone()));

    let egui_ctx = egui::Context::default();
    let mut egui_state = egui_winit::State::new(
        egui_ctx,
        egui::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
    );

    let mut egui_renderer = egui_wgpu::Renderer::new(
        &renderer.device,
        renderer.config.format,
        None,
        1,
    );

    println!("Vertexify started successfully");
    println!("Initial scene: {} entities", app.scene.entities().count());

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => {
                let egui_response = egui_state.on_window_event(&window, &event);
                
                if !app.handle_window_event(&event, &mut renderer, egui_response.consumed) {
                    match event {
                        WindowEvent::CloseRequested => {
                            println!("Closing Vertexify");
                            elwt.exit();
                        }
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            app.update_stats();

                            let raw_input = egui_state.take_egui_input(&window);
                            let full_output = egui_state.egui_ctx().run(raw_input, |ctx| {
                                if let Some(event) = app.gui.update(ctx) {
                                    app.handle_gui_event(event);
                                }
                            });

                            egui_state.handle_platform_output(&window, full_output.platform_output);

                            match render(&window, &mut renderer, &app.scene, &mut egui_state, &mut egui_renderer) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    eprintln!("Out of memory!");
                                    elwt.exit();
                                }
                                Err(e) => eprintln!("Render error: {:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn render(
    window: &winit::window::Window,
    renderer: &mut Renderer,
    scene: &Scene,
    egui_state: &mut egui_winit::State,
    egui_renderer: &mut egui_wgpu::Renderer,
) -> Result<(), wgpu::SurfaceError> {
    let output = renderer.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let view_proj = renderer.camera.view_projection_matrix();
    let uniforms = renderer::Uniforms {
        view_proj: view_proj.to_cols_array_2d(),
    };

    renderer.queue.write_buffer(
        &renderer.uniform_buffer,
        0,
        bytemuck::cast_slice(&[uniforms]),
    );

    let mut encoder = renderer.device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        },
    );

    let mut buffers = Vec::new();

    for entity in scene.entities() {
        if !entity.visible {
            continue;
        }

        if let Some(mesh) = &entity.mesh {
            let vertices: Vec<renderer::GpuVertex> =
                mesh.vertices.iter().map(|v| v.into()).collect();

            let vertex_buffer = renderer.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );

            let index_buffer = renderer.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                },
            );

            buffers.push((vertex_buffer, index_buffer, mesh.indices.len()));
        }
    }

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.15,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &renderer.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&renderer.render_pipeline);
        render_pass.set_bind_group(0, &renderer.uniform_bind_group, &[]);

        for (vertex_buffer, index_buffer, index_count) in &buffers {
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..*index_count as u32, 0, 0..1);
        }
    }

    let screen_descriptor = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [renderer.config.width, renderer.config.height],
        pixels_per_point: window.scale_factor() as f32,
    };

    let full_output = egui_state.egui_ctx().end_frame();
    let paint_jobs = egui_state.egui_ctx().tessellate(full_output.shapes, full_output.pixels_per_point);

    for (id, image_delta) in &full_output.textures_delta.set {
        egui_renderer.update_texture(
            &renderer.device,
            &renderer.queue,
            *id,
            image_delta,
        );
    }

    egui_renderer.update_buffers(
        &renderer.device,
        &renderer.queue,
        &mut encoder,
        &paint_jobs,
        &screen_descriptor,
    );

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Egui Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        egui_renderer.render(
            &mut render_pass,
            &paint_jobs,
            &screen_descriptor,
        );
    }

    for id in &full_output.textures_delta.free {
        egui_renderer.free_texture(id);
    }

    renderer.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
