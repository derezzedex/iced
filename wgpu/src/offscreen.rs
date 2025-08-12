#[derive(Debug, Clone)]
pub struct Pipeline {
    texture: Option<wgpu::Texture>,
    view: Option<wgpu::TextureView>,
    bind_group: Option<wgpu::BindGroup>,
    texture_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    raw: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Pipeline {
        let sampler =
            device.create_sampler(&wgpu::SamplerDescriptor::default());

        let texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::offscreen texture layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::NonFiltering,
                        ),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: false,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu::offscreen pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[&texture_layout],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu triangle blit_shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shader/offscreen.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::offscreen pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(
                            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                        ),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options:
                        wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        Self {
            texture: None,
            view: None,
            bind_group: None,
            sampler,
            raw: pipeline,
            texture_layout,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        frame: &wgpu::TextureView,
    ) {
        let frame = frame.texture();

        if self.texture.as_ref().is_none_or(|tex| {
            tex.width() != frame.width() || tex.height() != frame.height()
        }) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("iced_wgpu.offscreen.source_texture"),
                size: frame.size(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: frame.format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view =
                texture.create_view(&wgpu::TextureViewDescriptor::default());

            let bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("iced_wgpu::offscreen texture_bind_group"),
                    layout: &self.texture_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(
                                &self.sampler,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                    ],
                });

            self.bind_group = Some(bind_group);
            self.view = Some(view);
            self.texture = Some(texture);
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
    ) {
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu::offscreen render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame,
                    depth_slice: None,
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

        render_pass.set_pipeline(&self.raw);
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..6, 0..1);
    }

    pub fn view(&self) -> &wgpu::TextureView {
        self.view.as_ref().unwrap()
    }
}
