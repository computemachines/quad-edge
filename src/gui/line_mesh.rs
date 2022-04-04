use bevy::{
    core_pipeline::Transparent2d,
    ecs::system::lifetimeless::{Read, SQuery, SRes},
    prelude::*,
    reflect::{TypeUuid, Uuid},
    render::{
        mesh::GpuBufferInfo,
        render_asset::{RenderAsset, RenderAssets},
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline,
        },
        render_resource::{
            BlendState, ColorTargetState, ColorWrites, FragmentState, FrontFace, MultisampleState,
            PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, TextureFormat,
            VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode, SpecializedRenderPipelines, PipelineCache, SpecializedRenderPipeline,
        },
        texture::BevyDefault,
        view::VisibleEntities,
        RenderApp, RenderStage,
    },
    sprite::{
        Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    }, core::FloatOrd,
};

// example system

// fn add_lines(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
//     let mut lines = Mesh::new(PrimitiveTopology::LineList);
//     let mut v_pos = vec![[0., 0., 0.], [100., 100., 0.]];
//     lines.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);

//     let mut v_color = vec![[1., 0., 0., 1.], [0., 1., 0., 1.]];
//     lines.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

//     let indices: Vec<u32> = vec![0, 1];
//     lines.set_indices(Some(Indices::U32(indices)));

//     commands.spawn_bundle((
//         line_mesh::LineMesh::default(),
//         Mesh2dHandle(meshes.add(lines)),
//         Transform::default(),
//         GlobalTransform::default(),
//         Visibility::default(),
//         ComputedVisibility::default(),
//         AnimatedLine,
//     ));
// }



#[derive(Component, Default)]
pub struct LineMesh;

pub const LINES_MESH_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x97391c0926e9d409);
pub struct LineMeshPlugin;
impl Plugin for LineMeshPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            LINES_MESH_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("../../assets/shaders/line_mesh.wgsl")),
        );
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app.add_render_command::<Transparent2d, (
            SetItemPipeline,
            SetMesh2dViewBindGroup<0>,
            SetMesh2dBindGroup<1>,
            DrawLines2d,
        )>();
        render_app
            .init_resource::<LinesMeshPipeline>()
            .init_resource::<SpecializedRenderPipelines<LinesMeshPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_line_mesh2d)
            .add_system_to_stage(RenderStage::Queue, queue_line_mesh2d);
    }
}

fn extract_line_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Query<(Entity, &ComputedVisibility), With<LineMesh>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        values.push((entity, (LineMesh,)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

fn queue_line_mesh2d(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    line_mesh2d_pipeline: Res<LinesMeshPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<LinesMeshPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    line_mesh2d: Query<(&Mesh2dHandle, &Mesh2dUniform), With<LineMesh>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    if line_mesh2d.is_empty() {
        return;
    }
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_colored_mesh2d = transparent_draw_functions
            .read()
            .get_id::<(
                SetItemPipeline,
                SetMesh2dViewBindGroup<0>,
                SetMesh2dBindGroup<1>,
                DrawLines2d,
            )>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, mesh2d_uniform)) = line_mesh2d.get(*visible_entity) {
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id =
                    pipelines.specialize(&mut pipeline_cache, &line_mesh2d_pipeline, mesh2d_key);

                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_colored_mesh2d,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // in order to get correct transparency
                    sort_key: FloatOrd(mesh_z),
                    // This material is not batched
                    batch_range: None,
                });
            }
        }
    }
}

struct DrawLines2d;
impl EntityRenderCommand for DrawLines2d {
    type Param = (SRes<RenderAssets<Mesh>>, SQuery<Read<Mesh2dHandle>>);

    fn render<'w>(
        view: Entity,
        item: Entity,
        (meshes, mesh2d_query): bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let mesh_handle = &mesh2d_query.get(item).unwrap().0;
        if let Some(gpu_mesh) = meshes.into_inner().get(mesh_handle) {
            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..1);
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

struct LinesMeshPipeline {
    mesh2d_pipeline: Mesh2dPipeline,
}
impl FromWorld for LinesMeshPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}

impl SpecializedRenderPipeline for LinesMeshPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::RenderPipelineDescriptor {
        let vertex_attributes = vec![
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 16, // size of color attribute. bevy stores attributes in alphabetical order.
                shader_location: 0, // location of the position attribute
            },
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 1,
            },
        ];
        let vertex_array_stride = 28; // 3*4 + 4*4

        let vertex_state = VertexState {
            shader: LINES_MESH_SHADER_HANDLE.typed::<Shader>(),
            entry_point: "vs_main".into(),
            shader_defs: vec![],
            buffers: vec![VertexBufferLayout {
                array_stride: vertex_array_stride,
                step_mode: VertexStepMode::Vertex,
                attributes: vertex_attributes,
            }],
        };
        let fragment_state = FragmentState {
            shader: LINES_MESH_SHADER_HANDLE.typed::<Shader>(),
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
            label: Some("Line mesh pipeline".into()),
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
