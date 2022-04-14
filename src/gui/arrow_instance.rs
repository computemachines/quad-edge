use bevy::{
    core::{FloatOrd, Pod, Zeroable},
    core_pipeline::Transparent2d,
    ecs::system::lifetimeless::{Read, SQuery, SRes},
    prelude::*,
    reflect::{TypeUuid, Uuid},
    render::{
        mesh::{GpuBufferInfo, Indices},
        render_asset::{RenderAsset, RenderAssets},
        render_component::{ExtractComponent, ExtractComponentPlugin},
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline,
        },
        render_resource::{
            BlendState, Buffer, BufferInitDescriptor, BufferUsages, ColorTargetState, ColorWrites,
            FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineCache, RenderPipelineDescriptor, SpecializedPipeline,
            SpecializedPipelines, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
            VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::VisibleEntities,
        RenderApp, RenderStage,
    },
    sprite::{
        Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    },
};

pub const TWO_TRANSFORM_INTER_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x97391c0926e9d409);
pub struct ArrowPlugin;
impl Plugin for ArrowPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            TWO_TRANSFORM_INTER_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("../../assets/shaders/two_interp.wgsl")),
        );
        // app.add_plugin(ExtractComponentPlugin::<ArrowInstances>::default());
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app.add_render_command::<Transparent2d, (
            SetItemPipeline,
            SetMesh2dViewBindGroup<0>,
            SetMesh2dBindGroup<1>,
            DrawArrowInstanced,
        )>();
        render_app
            .init_resource::<ArrowInstancePipeline>()
            .init_resource::<SpecializedPipelines<ArrowInstancePipeline>>()
            .init_resource::<ArrowInstances>()
            .add_system_to_stage(RenderStage::Extract, extract_arrow_instances)
            .add_system_to_stage(RenderStage::Queue, queue_arrow_instances)
            .add_system_to_stage(RenderStage::Prepare, prepare_instance_buffers);
    }
}

#[derive(Bundle, Default)]
pub struct ArrowsBundle {
    pub mesh: Mesh2dHandle,

    pub local: Transform,
    pub global: GlobalTransform,
    pub visible: Visibility,
    pub computed_visibility: ComputedVisibility,

    pub instances: ArrowInstances,
}

#[derive(Component, Default, Debug, Clone)]
pub struct ArrowInstances(pub Vec<ArrowInstance>);

// Array of ArrowInstances are passed to GPU as an Instance Buffer
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct ArrowInstance {
    pub tail_global_transform: Vec3,
    pub head_global_transform: Vec3,
}

#[derive(Component)]
pub struct Arrow(pub Transform, pub Transform, pub Entity);

fn extract_arrow_instances(
    mut commands: Commands,
    arrows: Query<&Arrow>,
    mut query: Query<(&GlobalTransform, &mut ArrowInstances)>,
) {
    info!("extract arrow instances system");
    for (_, mut arrow_instance) in query.iter_mut() {
        info!("clearing arrow instances");
        arrow_instance.0.clear();
    }

    for Arrow(tail, head, arrow_entity) in arrows.iter() {
        info!("Extract Arrow into ArrowInstance for rendering");
        if let Ok((transform, mut arrow_instances)) = query.get_mut(*arrow_entity) {
            info!("Found arrow_type parent");
            arrow_instances.0.push(ArrowInstance {
                head_global_transform: transform.mul_transform(*head).translation,
                tail_global_transform: transform.mul_transform(*tail).translation,
            });
            commands.get_or_spawn(*arrow_entity).insert(QueueArrowInstanced);
        } else {
            warn!("Found arrow_type with broken parent");
        }
    }
}

#[derive(Component)]
struct QueueArrowInstanced;

fn queue_arrow_instances(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    arrow_instance_pipeline: Res<ArrowInstancePipeline>,
    mut pipelines: ResMut<SpecializedPipelines<ArrowInstancePipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    msaa: Res<Msaa>,
    arrow_instances: Query<(&Mesh2dHandle, &Mesh2dUniform), With<QueueArrowInstanced>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    info!("queue arrow instances system");
    // if arrow_instances.is_empty() {
    // return;
    // }
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        info!("queue view");
        let draw_arrow_instanced = transparent_draw_functions
            .read()
            .get_id::<(
                SetItemPipeline,
                SetMesh2dViewBindGroup<0>,
                SetMesh2dBindGroup<1>,
                DrawArrowInstanced,
            )>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            info!("queue visible entity");

            if let Ok((mesh2d_handle, mesh2d_uniform)) = arrow_instances.get(*visible_entity) {
                info!("entity is arrow_instances");

                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id =
                    pipelines.specialize(&mut pipeline_cache, &arrow_instance_pipeline, mesh2d_key);


                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_arrow_instanced,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // in order to get correct transparency
                    sort_key: FloatOrd(mesh_z),
                    // This material is not batched
                    batch_range: None,
                });
            } else {
                info!("entity is not arrow_instances");
            }
        }
    }
}

#[derive(Component, Debug)]
struct InstanceBuffer {
    buffer: Buffer,
    length: u32,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &ArrowInstances)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in query.iter() {
        info!("prepare instance buffer data");
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Instance buffer data"),
            contents: bytemuck::cast_slice(instance_data.0.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.0.len() as u32,
        });
    }
}

struct DrawArrowInstanced;
impl EntityRenderCommand for DrawArrowInstanced {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<Read<Mesh2dHandle>>,
        SQuery<Read<InstanceBuffer>>,
    );

    fn render<'w>(
        view: Entity,
        item: Entity,
        (meshes, mesh2d_query, instance_buffer_query): bevy::ecs::system::SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let mesh_handle = &mesh2d_query.get(item).unwrap().0;
        let instance_buffer = instance_buffer_query.get(item).unwrap();

        if let Some(gpu_mesh) = meshes.into_inner().get(mesh_handle) {
            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..instance_buffer.length);
                }
                GpuBufferInfo::NonIndexed { vertex_count } => {
                    pass.draw(0..*vertex_count, 0..1);
                }
            }
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

struct ArrowInstancePipeline {
    mesh2d_pipeline: Mesh2dPipeline,
}
impl FromWorld for ArrowInstancePipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}

impl SpecializedPipeline for ArrowInstancePipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::RenderPipelineDescriptor {
        let vertex_attributes = vec![
            VertexAttribute {
                // color
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 1,
            },
            VertexAttribute {
                //position
                format: VertexFormat::Float32x3,
                offset: 16, // size of color attribute. bevy stores MESH::*_ATTRIBUTES in alphabetical order.
                shader_location: 0, // location of the position attribute
            },
        ];
        let vertex_array_stride = 24; // 3*4 + 3*4

        let instance_vertex_attributes = vec![
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 2, // tail origin
            },
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: VertexFormat::Float32x3.size(),
                shader_location: 3, // head origin
            },
        ];

        let vertex_state = VertexState {
            shader: TWO_TRANSFORM_INTER_SHADER_HANDLE.typed::<Shader>(),
            entry_point: "vs_main".into(),
            shader_defs: vec![],
            buffers: vec![
                VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attributes,
                },
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<ArrowInstance>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: instance_vertex_attributes,
                },
            ],
        };
        let fragment_state = FragmentState {
            shader: TWO_TRANSFORM_INTER_SHADER_HANDLE.typed::<Shader>(),
            shader_defs: vec![],
            entry_point: "fs_main".into(),
            targets: vec![ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            }],
        };

        let bind_groups_layout = vec![
            self.mesh2d_pipeline.view_layout.clone(),
            self.mesh2d_pipeline.mesh_layout.clone(),
        ];

        RenderPipelineDescriptor {
            label: Some("arrow instance pipeline".into()),
            layout: Some(bind_groups_layout),
            vertex: vertex_state,
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList, //key.primitive_topology(),
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Line,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(fragment_state),
        }
    }
}
