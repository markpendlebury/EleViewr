use anyhow::{anyhow, Result};
use clap::Parser;
use image::GenericImageView;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use wgpu::util::DeviceExt;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Parser, Debug)]
#[command(name = "eleviewr")]
#[command(author = "User")]
#[command(version = "0.1.0")]
#[command(about = "A lightweight image viewer for Wayland/Hyprland", long_about = None)]
struct Args {
    #[arg(help = "Image file to open (optional, defaults to current directory)")]
    image_path: Option<String>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    screen_aspect: f32,
    image_aspect: f32,
    scale_factor: f32,
    _padding: f32,
}

struct ImageViewer {
    images: Vec<PathBuf>,
    current_index: usize,
    image_texture: Option<wgpu::Texture>,
    texture_bind_group: Option<wgpu::BindGroup>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    current_image_size: Option<(u32, u32)>,
}

impl ImageViewer {
    fn load_images_in_directory(&mut self, path: &Path) -> Result<()> {
        self.images.clear();

        let search_dir = if path.is_file() {
            // If it's a file, use its parent directory
            path.parent().unwrap_or_else(|| Path::new("."))
        } else {
            // If it's a directory, use it directly
            path
        };

        let target_filename = if path.is_file() {
            path.file_name()
        } else {
            None
        };

        let mut current_set = false;

        for entry in WalkDir::new(search_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if entry_path.is_file() {
                let ext = entry_path
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();
                if ["jpg", "jpeg", "png", "gif", "webp", "tiff", "bmp"].contains(&ext.as_str()) {
                    if target_filename.is_some() && entry_path.file_name() == target_filename {
                        self.current_index = self.images.len();
                        current_set = true;
                    }
                    self.images.push(entry_path.to_path_buf());
                }
            }
        }

        // Sort images alphabetically
        self.images.sort();

        // If we didn't find the current image, reset to the first one
        if !current_set && !self.images.is_empty() {
            self.current_index = 0;
        }

        if self.images.is_empty() {
            return Err(anyhow!(
                "No image files found in directory: {}",
                search_dir.display()
            ));
        }

        Ok(())
    }

    fn load_image(&mut self) -> Result<(String, (u32, u32))> {
        if self.images.is_empty() {
            return Err(anyhow!("No images loaded"));
        }

        let img_path = &self.images[self.current_index];
        println!("Loading image: {}", img_path.display());

        let img = image::open(img_path)?;
        let dimensions = img.dimensions();
        let rgba = img.to_rgba8();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Image Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        // Create texture view and bind group
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });

        self.texture_bind_group = Some(bind_group);
        self.image_texture = Some(texture);
        self.current_image_size = Some(dimensions);

        // Update uniform buffer with new image dimensions
        let screen_width = self.config.width as f32;
        let screen_height = self.config.height as f32;
        let screen_aspect = screen_width / screen_height;

        let image_width = dimensions.0 as f32;
        let image_height = dimensions.1 as f32;
        let image_aspect = image_width / image_height;

        // Scale factor can be used for zoom operations (default to 1.0)
        let scale_factor = 1.0;

        let uniforms = Uniforms {
            screen_aspect,
            image_aspect,
            scale_factor,
            _padding: 0.0,
        };

        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Return the image name and dimensions
        let title = format!(
            "EleViewr - {}",
            img_path.file_name().unwrap_or_default().to_string_lossy()
        );

        Ok((title, dimensions))
    }

    fn next_image(&mut self) -> Result<(String, (u32, u32))> {
        if self.images.is_empty() {
            return Err(anyhow!("No images loaded"));
        }

        self.current_index = (self.current_index + 1) % self.images.len();
        self.load_image()
    }

    fn prev_image(&mut self) -> Result<(String, (u32, u32))> {
        if self.images.is_empty() {
            return Err(anyhow!("No images loaded"));
        }

        self.current_index = if self.current_index == 0 {
            self.images.len() - 1
        } else {
            self.current_index - 1
        };
        self.load_image()
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // If no path is provided, use the current directory
    let path = match &args.image_path {
        Some(path_str) => {
            let p = Path::new(path_str);
            if !p.exists() {
                return Err(anyhow!("File or directory not found: {}", path_str));
            }
            p.to_path_buf()
        }
        None => std::env::current_dir()?,
    };

    let event_loop = EventLoop::new();

    // We'll create the window, wgpu instance, and surface all in the main function
    let window = WindowBuilder::new()
        .with_title("EleViewr")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)?;

    // Save window id before we move window into closure
    let win_id = window.id();

    // Create wgpu instance
    let instance = wgpu::Instance::default();

    // Create surface for the window - this is unsafe in wgpu 0.17
    let surface = unsafe { instance.create_surface(&window) }?;

    // Create adapter
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))
    .ok_or_else(|| anyhow!("Failed to find appropriate adapter"))?;

    // Create device and queue
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))?;

    // Configure surface
    let size = window.inner_size();
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    // Create texture bind group layout and sampler
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Image Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    // Create shader and render pipeline
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
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
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    // Create uniform buffer for aspect ratio preservation
    let initial_uniforms = Uniforms {
        screen_aspect: size.width as f32 / size.height as f32,
        image_aspect: 1.0, // Default to square
        scale_factor: 1.0,
        _padding: 0.0,
    };

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[initial_uniforms]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Create uniform bind group
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &device
                        .create_texture(&wgpu::TextureDescriptor {
                            label: Some("Dummy Texture"),
                            size: wgpu::Extent3d {
                                width: 1,
                                height: 1,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            usage: wgpu::TextureUsages::TEXTURE_BINDING
                                | wgpu::TextureUsages::COPY_DST,
                            view_formats: &[],
                        })
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    });

    // Create the ImageViewer with the components we've initialized
    let viewer = Arc::new(Mutex::new(ImageViewer {
        images: Vec::new(),
        current_index: 0,
        image_texture: None,
        texture_bind_group: None,
        texture_bind_group_layout,
        device,
        queue,
        config,
        render_pipeline,
        sampler,
        uniform_buffer,
        uniform_bind_group,
        current_image_size: None,
    }));

    // Load images from directory and update window
    {
        let mut viewer_lock = viewer.lock().unwrap();
        viewer_lock.load_images_in_directory(&path)?;

        // Load the first image and get its details
        let (title, dimensions) = viewer_lock.load_image()?;

        // Update window title
        window.set_title(&title);

        // Set window size based on image dimensions, with a maximum size
        let max_width = 1920;
        let max_height = 1080;
        let (width, height) = if dimensions.0 > max_width || dimensions.1 > max_height {
            let ratio = dimensions.0 as f32 / dimensions.1 as f32;
            if ratio > max_width as f32 / max_height as f32 {
                (max_width, (max_width as f32 / ratio) as u32)
            } else {
                ((max_height as f32 * ratio) as u32, max_height)
            }
        } else {
            (dimensions.0, dimensions.1)
        };

        // Resize window to fit image
        if width > 0 && height > 0 {
            window.set_inner_size(winit::dpi::PhysicalSize::new(width, height));
        }
    }

    // Run the event loop - this doesn't return, so we need to do our initialization before this
    // The Ok(()) return is not reachable
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { window_id, event } => {
                if window_id == win_id {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(physical_size) => {
                            let mut viewer_lock = viewer.lock().unwrap();
                            if physical_size.width > 0 && physical_size.height > 0 {
                                viewer_lock.config.width = physical_size.width;
                                viewer_lock.config.height = physical_size.height;
                                surface.configure(&viewer_lock.device, &viewer_lock.config);

                                // Update uniform buffer with new screen aspect ratio
                                if let Some(dimensions) = viewer_lock.current_image_size {
                                    let screen_width = physical_size.width as f32;
                                    let screen_height = physical_size.height as f32;
                                    let screen_aspect = screen_width / screen_height;

                                    let image_width = dimensions.0 as f32;
                                    let image_height = dimensions.1 as f32;
                                    let image_aspect = image_width / image_height;

                                    let uniforms = Uniforms {
                                        screen_aspect,
                                        image_aspect,
                                        scale_factor: 1.0,
                                        _padding: 0.0,
                                    };

                                    viewer_lock.queue.write_buffer(
                                        &viewer_lock.uniform_buffer,
                                        0,
                                        bytemuck::cast_slice(&[uniforms]),
                                    );
                                }
                            }
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            if input.state == ElementState::Pressed {
                                match input.virtual_keycode {
                                    Some(winit::event::VirtualKeyCode::Escape) => {
                                        *control_flow = ControlFlow::Exit;
                                    }
                                    Some(winit::event::VirtualKeyCode::Right)
                                    | Some(winit::event::VirtualKeyCode::L) => {
                                        let mut viewer_lock = viewer.lock().unwrap();
                                        if let Ok((title, _dimensions)) = viewer_lock.next_image() {
                                            // We can't update the window title directly here
                                            // since the window is already moved into the event loop
                                            println!("Next image: {}", title);
                                        }
                                    }
                                    Some(winit::event::VirtualKeyCode::Left)
                                    | Some(winit::event::VirtualKeyCode::H) => {
                                        let mut viewer_lock = viewer.lock().unwrap();
                                        if let Ok((title, _dimensions)) = viewer_lock.prev_image() {
                                            // We can't update the window title directly here
                                            // since the window is already moved into the event loop
                                            println!("Previous image: {}", title);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Render the current frame
                let viewer_lock = viewer.lock().unwrap();
                let output = match surface.get_current_texture() {
                    Ok(output) => output,
                    Err(_) => return,
                };

                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    viewer_lock
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

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
                                    b: 0.1,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                    render_pass.set_pipeline(&viewer_lock.render_pipeline);

                    // Only draw the image if we have a bind group (i.e., an image loaded)
                    if let Some(bind_group) = &viewer_lock.texture_bind_group {
                        // Use the texture bind group for rendering
                        render_pass.set_bind_group(0, bind_group, &[]);
                        render_pass.draw(0..6, 0..1);
                    } else {
                        // Use the default uniform bind group if no image is loaded
                        render_pass.set_bind_group(0, &viewer_lock.uniform_bind_group, &[]);
                        render_pass.draw(0..6, 0..1);
                    }
                }

                viewer_lock.queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            _ => {}
        }
    });

    // This code is unreachable, but Rust doesn't know that and requires us to return a Result
    #[allow(unreachable_code)]
    Ok(())
}
