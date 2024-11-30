/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use qfg5reenigne::{
    threed::{
        model::{
            Vertex,
            ModelVertex,
        },
        model,
        camera,
        resources,
        texture,
    },
    qfg5resource::{
        qfg5anm, qfg5mdl,
    }
};

use anyhow::Result;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use cgmath::prelude::*;
use wgpu::util::DeviceExt;


struct Instance {
    // position: cgmath::Vector3<f32>,
    transform: cgmath::Matrix4<f32>,
    // rotation: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            //model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
            model: self.transform.into()
        }
    }
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

enum RenderWhat {
    MeshIndex(usize),
    AnmIndex(usize),
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    camera: camera::Camera,
    camera_controller: camera::CameraController,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    mdl: qfg5mdl::Qfg5Model,
    anm: qfg5anm::AnmDecoder,
    obj_model: model::Model,
    render_what: RenderWhat,
    // Font
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,
    text_buffer: glyphon::Buffer,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: &'a Window, mdl_fname: &str, anm_fname: &str) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
                memory_hints: Default::default(),
            },
            None, // Trace path
        ).await.unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        
        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let camera = camera::Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (4.0, 12.0, 42.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_controller = camera::CameraController::new(1.0);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
 
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
    
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");
            
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc(),
                    InstanceRaw::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Start with empty instances - will be updated by update_render()
        let instances = Vec::new();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let mdl = resources::load_binary(mdl_fname).await.unwrap();
        let mdl = qfg5mdl::Qfg5Model::new(&mdl).unwrap();

        let obj_model =
            resources::load_qfg5model(&mdl, &device, &queue, &texture_bind_group_layout)
            .await;
        if let Err(ref e) = obj_model {
            println!("ARGH {:?}", e);
        }
        let obj_model = obj_model.unwrap();
        let render_what = RenderWhat::MeshIndex(0);

        let anm = resources::load_binary(anm_fname).await.unwrap();
        let anm = qfg5anm::AnmDecoder::new(&anm).unwrap();

        // TODO is dit goed ??
        assert_eq!(anm.anims.len(), mdl.submeshes.len());

         // Set up text renderer
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(&device, &cache);
        let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let mut atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        let physical_width = (size.width as f64 * window.scale_factor()) as f32;
        let physical_height = (size.height as f64 * window.scale_factor()) as f32;

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        text_buffer.set_text(&mut font_system, "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
        text_buffer.shape_until_scroll(&mut font_system, false);

        let mut result = Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            depth_texture,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            instances,
            instance_buffer,
            mdl,
            anm,
            obj_model,
            render_what,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer,
        };
        result.update_render();
        result
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn process_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::Equal => {
                        if is_pressed {
                            match self.render_what {
                                RenderWhat::MeshIndex(mut index) => {
                                    index += 1;
                                    if index == self.obj_model.meshes.len() {
                                        index = 0;
                                    }
                                    self.render_what = RenderWhat::MeshIndex(index);
                                },
                                RenderWhat::AnmIndex(mut index) => {
                                    index += 1;
                                    if index == self.anm.anims.len() {
                                        index = 0;
                                    }
                                    self.render_what = RenderWhat::AnmIndex(index);
                                }
                            }
                            self.update_render();
                            true
                        } else {
                            false
                        }
                    },
                    KeyCode::Minus => {
                        if is_pressed {
                            match self.render_what {
                                RenderWhat::MeshIndex(mut index) => {
                                    if index > 0 {
                                        index -= 1;
                                    } else {
                                        index = self.obj_model.meshes.len() - 1;
                                    }
                                    self.render_what = RenderWhat::MeshIndex(index);
                                },
                                RenderWhat::AnmIndex(mut index) => {
                                    if index > 0 {
                                        index -= 1;
                                    } else {
                                        index = self.anm.anims.len() - 1;
                                    }
                                    self.render_what = RenderWhat::AnmIndex(index);
                                }
                            }
                            self.update_render();
                            true
                        } else {
                            false
                        }
                    },
                    KeyCode::Space => {
                        if is_pressed {
                            self.render_what = match self.render_what {
                                RenderWhat::MeshIndex(_) => RenderWhat::AnmIndex(0),
                                RenderWhat::AnmIndex(_) => RenderWhat::MeshIndex(0),
                            };
                            self.update_render();
                            true
                        } else {
                            false
                        }
                    },
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.process_input(event) || self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform])); 
    }

    fn update_render(&mut self) {
        let mut instances = Vec::new();
        let s = match self.render_what {
            RenderWhat::MeshIndex(index) => {
                let position = cgmath::Vector3{ x: 0.0, y: 0.0, z: 0.0 };
                let rotation = cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(-90.0));
                let transform = cgmath::Matrix4::from_translation(position) * cgmath::Matrix4::from(rotation);
                instances.push(Instance{ transform });

                let submesh = &self.mdl.submeshes[index];
                &format!("mesh: {} - {} - {}/{}", self.mdl.name, submesh.name, index, self.mdl.submeshes.len())
            },
            RenderWhat::AnmIndex(index) => {
                for (mesh_index, _) in self.mdl.submeshes.iter().enumerate() {
                    let block = &self.anm.anims[mesh_index].blocks[index];
                    /*
                    let x = cgmath::Vector4{ x: block.rotation[0], y: block.rotation[1], z: block.rotation[2], w: 0.0 };
                    let y = cgmath::Vector4{ x: block.rotation[3], y: block.rotation[4], z: block.rotation[5], w: 0.0 };
                    let z = cgmath::Vector4{ x: block.rotation[6], y: block.rotation[7], z: block.rotation[8], w: 0.0 };
                    */
                    let x = cgmath::Vector4{ x: block.rotation[0], y: block.rotation[3], z: block.rotation[6], w: 0.0 };
                    let y = cgmath::Vector4{ x: block.rotation[1], y: block.rotation[4], z: block.rotation[7], w: 0.0 };
                    let z = cgmath::Vector4{ x: block.rotation[2], y: block.rotation[5], z: block.rotation[8], w: 0.0 };
                    let w = cgmath::Vector4{ x: block.translation[0], y: block.translation[1], z: block.translation[2], w: 1.0 };
                    let transform = cgmath::Matrix4{ x, y, z, w };
                    instances.push(Instance{ transform });
                }
                &format!("animation: {} - {} - {}/{}", self.mdl.name, self.anm.name, index, self.anm.anims.len())
            }
        };
        self.text_buffer.set_text(&mut self.font_system, s, Attrs::new().family(Family::SansSerif), Shaping::Advanced);

        self.instances = instances;
        let instance_data = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        self.instance_buffer = instance_buffer;
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.viewport.update(
            &self.queue,
            Resolution {
                width: self.config.width,
                height: self.config.height,
            },
        );

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [TextArea {
                    buffer: &self.text_buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 600,
                        bottom: 160,
                    },
                    default_color: Color::rgb(255, 255, 255),
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .unwrap();

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                // This is what @location(0) in the fragment shader targets
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline);
            use qfg5reenigne::threed::model::DrawModel;
            match self.render_what {
                RenderWhat::MeshIndex(index) => {
                    render_pass.draw_single_mesh_instanced(&self.obj_model, index, 0..self.instances.len() as u32, &self.camera_bind_group);
                },
                RenderWhat::AnmIndex(_) => {
                    for (n, _) in self.instances.iter().enumerate() {
                        render_pass.draw_single_mesh_instanced(&self.obj_model, n, n as u32..(n + 1) as u32, &self.camera_bind_group);
                    }
                }
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
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

                self.text_renderer.render(&self.atlas, &self.viewport, &mut pass).unwrap();
            }
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.atlas.trim();
    
        Ok(())
    }
}

async fn run(mdl_fname: &str, anm_fname: &str) -> Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop)?;

    let mut state = State::new(&window, mdl_fname, anm_fname).await;

    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },
                    WindowEvent::RedrawRequested => {
                        // This tells winit that we want another frame after this one
                        state.window().request_redraw();
            
                        state.update();
                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if it's lost or outdated
                            Err(
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                            ) => state.resize(state.size),
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                log::error!("OutOfMemory");
                                control_flow.exit();
                            }
            
                            // This happens when the a frame takes too long to present
                            Err(wgpu::SurfaceError::Timeout) => {
                                log::warn!("Surface timeout")
                            }
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    })?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().into_iter().collect();
    if args.len() != 3 {
        println!("usage: {} file.mdl file.anm", args[0]);
    } else {
        pollster::block_on(run(&args[1], &args[2]))?;
    }
    Ok(())
}
