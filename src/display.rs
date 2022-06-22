use winit::{window::Window, dpi::PhysicalSize};
use wgpu::*;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT / u8::BITS as usize;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}
impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                }
            ]
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 32.0] },
    Vertex { position: [1.0, -1.0, 0.0], tex_coords: [64.0, 32.0] },
    Vertex { position: [1.0, 1.0, 0.0], tex_coords: [64.0, 0.0] },
    Vertex { position: [-1.0, 1.0, 0.0], tex_coords: [0.0, 0.0] },
];
const INDICES: &[u16] = &[
    0, 1, 2,
    0, 2, 3,
];

pub struct Display {
    display: [u8; DISPLAY_SIZE],
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub(super) size: PhysicalSize<u32>,
    texture: Texture,
    texture_bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}
impl Display {
    pub async fn new(window: &Window) -> Self {
        let display = [0; DISPLAY_SIZE];

        let size = window.inner_size();
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                features: Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::default()
                },
                label: None,
            },
            None,
        ).await.unwrap();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let texture = device.create_texture(
            &TextureDescriptor {
                label: Some("texture"),
                size: Extent3d {
                    width: DISPLAY_SIZE as u32,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D1,
                format: TextureFormat::R8Uint,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            }
        );
        queue.write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &display,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(DISPLAY_SIZE as u32),
                rows_per_image: std::num::NonZeroU32::new(1),
            },
            Extent3d {
                width: DISPLAY_SIZE as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D1,
                        sample_type: TextureSampleType::Uint,
                    },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
        let texture_bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture_view),
                    },
                ],
                label: Some("texture_bind_group"),
            }
        );

        let shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        use util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some("vertex_buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some("index_buffer"),
                contents: bytemuck::cast_slice(&INDICES),
                usage: BufferUsages::INDEX,
            }
        ); 

        Self {
            display,
            surface,
            device,
            queue,
            config,
            size,
            texture,
            texture_bind_group,
            render_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    pub fn update(&mut self) {
        self.queue.write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &self.display,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(DISPLAY_SIZE as u32),
                rows_per_image: std::num::NonZeroU32::new(1),
            },
            Extent3d {
                width: DISPLAY_SIZE as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let output_view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut render_encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

        {
            let mut render_pass = render_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { 
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

    self.queue.submit(std::iter::once(render_encoder.finish()));
    output.present();

    Ok(())
    }
    pub fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut flip = false;
        let x = x % DISPLAY_WIDTH;
        let y = y % DISPLAY_HEIGHT;
        for dy in 0..sprite.len().min(DISPLAY_HEIGHT - y) {
            /*if x % 8 == 0 {
                self.display[((y + dy) * DISPLAY_WIDTH + x) / 8] ^= sprite[dy];
            } else {
                self.display[((y + dy) * DISPLAY_WIDTH + x) / 8] ^= sprite[dy] << (x % 8);
                if x + 8 < DISPLAY_WIDTH { 
                    self.display[((y + dy) * DISPLAY_WIDTH + x + 8 - 1) / 8] ^= sprite[dy] >> (x % 8);
                }
            }*/
            for dx in 0..8 {
                if (sprite[dy] & (1 << (7 - dx))) != 0 {
                    let display_byte = &mut self.display[((y + dy) * DISPLAY_WIDTH + (x + dx)) / 8];
                    if *display_byte & (1 << ((x + dx) % 8)) != 0 {
                        flip = true;
                    }
                    *display_byte ^= 1 << ((x + dx) % 8);

                }
            }
        }
        flip
    }
    pub fn clear(&mut self) {
        self.display = [0; DISPLAY_SIZE];
    }
}
