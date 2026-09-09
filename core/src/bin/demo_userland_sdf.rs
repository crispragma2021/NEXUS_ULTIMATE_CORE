// ============================================================
// DEMO USERLAND SDF - GPU Accelerated System Monitor
// ============================================================
// Renderiza métricas del sistema (CPU, RAM) directamente en la
// GPU usando Signed Distance Fields (SDF) y WGPU/Winit 0.30.
// Estética OMEGA: Dark mode, resplandor neón cian/esmeralda.
// ============================================================

use nexus_ultimate_core::memoria::persistence::DatabaseManager;
use std::sync::Arc;
use sysinfo::System;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
};

// Uniforms estructurados para enviar al shader (exactamente 16 bytes)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    cpu_usage: f32,
    ram_usage: f32,
    net_packets: f32, // Sustituye _padding
}

// Estructura de la aplicación WGPU + Winit
struct SDFApp {
    instance: wgpu::Instance,
    state: Option<AppState>,
    sys: System,
    start_time: std::time::Instant,
    net_packets_counter: Arc<std::sync::atomic::AtomicU32>,
    net_visual: f32,
}

struct AppState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl SDFApp {
    fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let mut sys = System::new_all();
        sys.refresh_all();

        let net_packets_counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let net_packets_clone = net_packets_counter.clone();

        // Lanzar hilo de telemetría eBPF
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("⚠️ [eBPF BRIDGE] Error al inicializar runtime: {:?}", e);
                    return;
                }
            };

            rt.block_on(async {
                let db_path = "sqlite:nexus_intelligence.db";
                if let Ok(db) = DatabaseManager::new(db_path).await {
                    let mut last_total = 0u64;
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                        if let Some(ref pool) = db.pool {
                            let query_res =
                                sqlx::query("SELECT SUM(packet_count) FROM traffic_stats")
                                    .fetch_one(&**pool)
                                    .await;
                            if let Ok(row) = query_res {
                                use sqlx::Row;
                                let current_total: i64 = row.try_get(0).unwrap_or(0);
                                let current_total = current_total.max(0) as u64;
                                if last_total == 0 {
                                    last_total = current_total;
                                }
                                let diff = current_total.saturating_sub(last_total);
                                last_total = current_total;
                                net_packets_clone
                                    .store(diff as u32, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    }
                }
            });
        });

        Self {
            instance,
            state: None,
            sys,
            start_time: std::time::Instant::now(),
            net_packets_counter,
            net_visual: 0.0,
        }
    }
}

// Implementación del ApplicationHandler para Winit 0.30
impl ApplicationHandler for SDFApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        // Crear la ventana
        let window_attributes = Window::default_attributes()
            .with_title("🔱 NEXUS - SDF Userland Monitor")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        // Inicialización asíncrona de WGPU bloqueando el hilo de inicialización
        let app_state = pollster::block_on(async {
            let size = window.inner_size();

            // Crear superficie
            let surface = self.instance.create_surface(window.clone()).unwrap();

            // Solicitar adaptador
            let adapter = self
                .instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap();

            // Solicitar dispositivo y cola
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .unwrap();

            // Configurar superficie
            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&device, &config);

            // Cargar WGSL Shader
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("SDF Shader Module"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "sdf_shader.wgsl"
                ))),
            });

            // Crear buffer de uniforms
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Uniform Buffer"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Layout del bind group
            let uniform_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Uniform Bind Group Layout"),
                });

            // Crear el bind group
            let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
                label: Some("Uniform Bind Group"),
            });

            // Crear pipeline layout
            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

            // Crear Render Pipeline
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("SDF Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

            AppState {
                window,
                surface,
                device,
                queue,
                config,
                render_pipeline,
                uniform_buffer,
                uniform_bind_group,
            }
        });

        self.state = Some(app_state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size)
                if physical_size.width > 0 && physical_size.height > 0 =>
            {
                state.config.width = physical_size.width;
                state.config.height = physical_size.height;
                state.surface.configure(&state.device, &state.config);
                state.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                // Actualizar métricas del sistema
                self.sys.refresh_cpu_usage();
                self.sys.refresh_memory();

                let cpus = self.sys.cpus();
                let cpu = if cpus.is_empty() {
                    0.0
                } else {
                    cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / (cpus.len() as f32 * 100.0)
                };
                let used_mem = self.sys.used_memory() as f32;
                let total_mem = self.sys.total_memory() as f32;
                let ram = used_mem / total_mem;

                // Leer actividad de red eBPF y aplicar filtro de amortiguación (decay)
                let raw_diff =
                    self.net_packets_counter
                        .load(std::sync::atomic::Ordering::Relaxed) as f32;
                if raw_diff > 0.0 {
                    // Impulso según la cantidad de paquetes, amortiguado
                    self.net_visual = (self.net_visual + raw_diff * 0.05).min(1.0);
                } else {
                    // Decaimiento continuo para animación orgánica y suave
                    self.net_visual = (self.net_visual - 0.02).max(0.0);
                }

                let uniforms = Uniforms {
                    time: self.start_time.elapsed().as_secs_f32(),
                    cpu_usage: cpu,
                    ram_usage: ram,
                    net_packets: self.net_visual,
                };

                // Actualizar buffer de uniforms en la GPU
                state
                    .queue
                    .write_buffer(&state.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

                // Obtener textura de la superficie
                let frame = match state.surface.get_current_texture() {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Error al obtener textura de superficie: {:?}", e);
                        return;
                    }
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    state
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                // Iniciar pase de renderizado
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("SDF Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.02,
                                    g: 0.02,
                                    b: 0.05,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    render_pass.set_pipeline(&state.render_pipeline);
                    render_pass.set_bind_group(0, &state.uniform_bind_group, &[]);
                    // Dibujar 3 vértices que forman un triángulo de pantalla completa
                    render_pass.draw(0..3, 0..1);
                }

                // Presentar frame
                state.queue.submit(std::iter::once(encoder.finish()));
                frame.present();

                // Continuar pidiendo redibujados para animación fluida a 60FPS
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = SDFApp::new();
    println!("🔱 [NEXUS SDF] Iniciando bucle de visualización acelerado por hardware...");
    event_loop.run_app(&mut app).unwrap();
}
