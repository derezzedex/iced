use crate::program;
use crate::Transformation;
use glow::HasContext;
use iced_graphics::layer;
use iced_native::Rectangle;

const MAX_INSTANCES: usize = 100_000;

#[derive(Debug)]
pub struct Pipeline {
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    vertex_buffer: <glow::Context as HasContext>::Buffer,
    index_buffer: <glow::Context as HasContext>::Buffer,
    transform_location: <glow::Context as HasContext>::UniformLocation,
    scale_location: <glow::Context as HasContext>::UniformLocation,
    screen_height_location: <glow::Context as HasContext>::UniformLocation,
    current_transform: Transformation,
    current_scale: f32,
    current_target_height: u32,
}

impl Pipeline {
    pub fn new(gl: &glow::Context) -> Pipeline {
        let program = unsafe {
            program::create(
                gl,
                &[
                    (glow::VERTEX_SHADER, include_str!("shader/quad.vert")),
                    (glow::FRAGMENT_SHADER, include_str!("shader/quad.frag")),
                ],
                Some(&[
                    (0, "i_Pos"),
                    (1, "i_Scale"),
                    (2, "i_Color"),
                    (3, "i_BorderColor"),
                    (4, "i_BorderRadius"),
                    (5, "i_BorderWidth"),
                    (6, "i_id"),
                ]),
            )
        };

        let transform_location =
            unsafe { gl.get_uniform_location(program, "u_Transform") }
                .expect("Get transform location");

        let scale_location =
            unsafe { gl.get_uniform_location(program, "u_Scale") }
                .expect("Get scale location");

        let screen_height_location =
            unsafe { gl.get_uniform_location(program, "u_ScreenHeight") }
                .expect("Get target height location");

        unsafe {
            gl.use_program(Some(program));

            let matrix: [f32; 16] = Transformation::identity().into();
            gl.uniform_matrix_4_f32_slice(
                Some(&transform_location),
                false,
                &matrix,
            );

            gl.uniform_1_f32(Some(&scale_location), 1.0);
            gl.uniform_1_f32(Some(&screen_height_location), 0.0);

            gl.use_program(None);
        }

        let (vertex_array, vertex_buffer, index_buffer) =
            unsafe { create_instance_buffer(gl, MAX_INSTANCES) };

        Pipeline {
            program,
            vertex_array,
            vertex_buffer,
            index_buffer,
            transform_location,
            scale_location,
            screen_height_location,
            current_transform: Transformation::identity(),
            current_scale: 1.0,
            current_target_height: 0,
        }
    }

    pub fn draw(
        &mut self,
        gl: &glow::Context,
        target_height: u32,
        instances: &[layer::Quad],
        transformation: Transformation,
        scale: f32,
        bounds: Rectangle<u32>,
    ) {
        unsafe {
            gl.enable(glow::SCISSOR_TEST);
            gl.scissor(
                bounds.x as i32,
                (target_height - (bounds.y + bounds.height)) as i32,
                bounds.width as i32,
                bounds.height as i32,
            );

            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer));
        }

        if transformation != self.current_transform {
            unsafe {
                let matrix: [f32; 16] = transformation.into();
                gl.uniform_matrix_4_f32_slice(
                    Some(&self.transform_location),
                    false,
                    &matrix,
                );

                self.current_transform = transformation;
            }
        }

        if scale != self.current_scale {
            unsafe {
                gl.uniform_1_f32(Some(&self.scale_location), scale);
            }

            self.current_scale = scale;
        }

        if target_height != self.current_target_height {
            unsafe {
                gl.uniform_1_f32(
                    Some(&self.screen_height_location),
                    target_height as f32,
                );
            }

            self.current_target_height = target_height;
        }

        let mut i = 0;
        let total = instances.len();

        while i < total {
            let end = (i + MAX_INSTANCES).min(total);
            let amount = end - i;

            unsafe {
                let vertices =
                    instances[i..end].iter().fold(Vec::new(), |mut v, q| {
                        v.extend_from_slice(&[
                            layer::Quad { id: 0., ..*q },
                            layer::Quad { id: 1., ..*q },
                            layer::Quad { id: 2., ..*q },
                            layer::Quad { id: 3., ..*q },
                        ]);
                        v
                    });
                gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    0,
                    bytemuck::cast_slice(&vertices),
                );

                let indices = (0..amount as i32).fold(
                    Vec::new(),
                    |mut indices, i| {
                        indices.extend_from_slice(&[
                            0 + i * 4,
                            1 + i * 4,
                            2 + i * 4,
                            2 + i * 4,
                            1 + i * 4,
                            3 + i * 4,
                        ]);
                        indices
                    },
                );
                gl.buffer_sub_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    0,
                    bytemuck::cast_slice(&indices),
                );

                gl.draw_elements(
                    glow::TRIANGLES,
                    indices.len() as i32,
                    glow::UNSIGNED_INT,
                    0,
                );
            }

            i += MAX_INSTANCES;
        }

        unsafe {
            gl.bind_vertex_array(None);
            gl.use_program(None);
            gl.disable(glow::SCISSOR_TEST);
        }
    }
}

unsafe fn create_instance_buffer(
    gl: &glow::Context,
    size: usize,
) -> (
    <glow::Context as HasContext>::VertexArray,
    <glow::Context as HasContext>::Buffer,
    <glow::Context as HasContext>::Buffer,
) {
    let vertex_array = gl.create_vertex_array().expect("Create vertex array");
    let vertex_buffer = gl.create_buffer().expect("Create vertex buffer");
    let index_buffer = gl.create_buffer().expect("Create index buffer");

    gl.bind_vertex_array(Some(vertex_array));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
    gl.buffer_data_size(
        glow::ARRAY_BUFFER,
        (size * 4 * std::mem::size_of::<layer::Quad>()) as i32, // Number of quads * number of vertices per quad * size of quad (bytes)
        glow::DYNAMIC_DRAW,
    );

    let index_buffer_size = (size * 3) / 2; // For every Quad, there are 4 vertices, but 6 indices, which means the index buffer is 1.5x bigger (6/4 -> 3/2)
    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
    gl.buffer_data_size(
        glow::ELEMENT_ARRAY_BUFFER,
        (index_buffer_size * 4) as i32, // Number of indices * size of index (float = 4 bytes)
        glow::DYNAMIC_DRAW,
    );

    let stride = std::mem::size_of::<layer::Quad>() as i32;

    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
    // gl.vertex_attrib_divisor(0, 1);

    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 4 * 2);
    // gl.vertex_attrib_divisor(1, 1);

    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, stride, 4 * (2 + 2));
    // gl.vertex_attrib_divisor(2, 1);

    gl.enable_vertex_attrib_array(3);
    gl.vertex_attrib_pointer_f32(
        3,
        4,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4),
    );
    // gl.vertex_attrib_divisor(3, 1);

    gl.enable_vertex_attrib_array(4);
    gl.vertex_attrib_pointer_f32(
        4,
        1,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4 + 4),
    );
    // gl.vertex_attrib_divisor(4, 1);

    gl.enable_vertex_attrib_array(5);
    gl.vertex_attrib_pointer_f32(
        5,
        1,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4 + 4 + 1),
    );
    // gl.vertex_attrib_divisor(5, 1);

    gl.enable_vertex_attrib_array(6);
    gl.vertex_attrib_pointer_f32(
        6,
        1,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4 + 4 + 1 + 1),
    );
    // gl.vertex_attrib_divisor(6, 1);

    gl.enable_vertex_attrib_array(7);
    gl.vertex_attrib_pointer_f32(
        7,
        1,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4 + 4 + 1 + 1 + 1),
    );

    gl.bind_vertex_array(None);
    gl.bind_buffer(glow::ARRAY_BUFFER, None);
    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

    (vertex_array, vertex_buffer, index_buffer)
}
