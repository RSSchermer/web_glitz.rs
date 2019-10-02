use js_sys::{Uint32Array, Uint8Array};
use web_sys::{WebGl2RenderingContext as Gl, WebGlProgram, WebGlUniformLocation};

use crate::pipeline::interface_block::{MatrixOrder, MemoryUnit, UnitLayout};
use crate::pipeline::resources::resources::{ResourceSlotKind, SampledTextureType};
use crate::pipeline::resources::ResourceSlotIdentifier;

/// Describes a slot for a resource in a GPU pipeline.
#[derive(Debug)]
pub(crate) struct ShaderResourceSlotDescriptor {
    identifier: ResourceSlotIdentifier,
    slot: SlotType,
}

impl ShaderResourceSlotDescriptor {
    pub(crate) fn new(identifier: ResourceSlotIdentifier, slot: SlotType) -> Self {
        ShaderResourceSlotDescriptor { identifier, slot }
    }

    /// Returns the identifier for the slot.
    pub(crate) fn identifier(&self) -> &ResourceSlotIdentifier {
        &self.identifier
    }

    /// Returns information about the type of the slot.
    ///
    /// See [SlotType] for details.
    pub(crate) fn slot_type(&self) -> &SlotType {
        &self.slot
    }
}

#[derive(Debug)]
pub(crate) enum SlotType {
    UniformBlock(UniformBlockSlot),
    TextureSampler(TextureSamplerSlot),
}

impl SlotType {
    pub(crate) fn is_kind(&self, kind: ResourceSlotKind) -> bool {
        match self {
            SlotType::UniformBlock(_) => kind.is_uniform_buffer(),
            SlotType::TextureSampler(_) => kind.is_sampled_texture(),
        }
    }
}

impl From<UniformBlockSlot> for SlotType {
    fn from(slot: UniformBlockSlot) -> Self {
        SlotType::UniformBlock(slot)
    }
}

impl From<TextureSamplerSlot> for SlotType {
    fn from(slot: TextureSamplerSlot) -> Self {
        SlotType::TextureSampler(slot)
    }
}

#[derive(Debug)]
pub(crate) struct UniformBlockSlot {
    layout: Vec<MemoryUnit>,
    index: u32,
}

impl UniformBlockSlot {
    pub(crate) fn new(gl: &Gl, program: &WebGlProgram, index: usize) -> Self {
        let index = index as u32;
        let unit_count = gl
            .get_active_uniform_block_parameter(program, index, Gl::UNIFORM_BLOCK_ACTIVE_UNIFORMS)
            .unwrap()
            .as_f64()
            .unwrap() as usize;

        let mut indices: Vec<u32> = vec![0; unit_count];
        let mut types: Vec<u32> = vec![0; unit_count];
        let mut sizes: Vec<u32> = vec![0; unit_count];
        let mut offsets: Vec<u32> = vec![0; unit_count];
        let mut array_strides: Vec<u32> = vec![0; unit_count];
        let mut matrix_strides: Vec<u32> = vec![0; unit_count];
        let mut matrix_orientations: Vec<u8> = vec![0; unit_count];

        let js_indices_array = gl
            .get_active_uniform_block_parameter(
                program,
                index,
                Gl::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
            )
            .unwrap();

        let js_types_array = gl.get_active_uniforms(program, &js_indices_array, Gl::UNIFORM_TYPE);

        let js_sizes_array = gl.get_active_uniforms(program, &js_indices_array, Gl::UNIFORM_SIZE);

        let js_offsets_array =
            gl.get_active_uniforms(program, &js_indices_array, Gl::UNIFORM_OFFSET);

        let js_array_strides_array =
            gl.get_active_uniforms(program, &js_indices_array, Gl::UNIFORM_ARRAY_STRIDE);

        let js_matrix_strides_array =
            gl.get_active_uniforms(program, &js_indices_array, Gl::UNIFORM_MATRIX_STRIDE);

        let js_matrix_orientations_array =
            gl.get_active_uniforms(program, &js_indices_array, Gl::UNIFORM_IS_ROW_MAJOR);

        Uint32Array::new(&js_indices_array).copy_to(&mut indices);
        Uint32Array::new(&js_types_array).copy_to(&mut types);
        Uint32Array::new(&js_sizes_array).copy_to(&mut sizes);
        Uint32Array::new(&js_offsets_array).copy_to(&mut offsets);
        Uint32Array::new(&js_array_strides_array).copy_to(&mut array_strides);
        Uint32Array::new(&js_matrix_strides_array).copy_to(&mut matrix_strides);
        Uint8Array::new(&js_matrix_orientations_array).copy_to(&mut matrix_orientations);

        let mut layout = Vec::with_capacity(unit_count);

        for i in 0..unit_count {
            use crate::pipeline::interface_block::UnitLayout::*;

            let size = sizes[i];

            let unit = match types[i] {
                Gl::INT => {
                    if size == 1 {
                        Integer
                    } else {
                        IntegerArray {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::INT_VEC2 => {
                    if size == 1 {
                        IntegerVector2
                    } else {
                        IntegerVector2Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::INT_VEC3 => {
                    if size == 1 {
                        IntegerVector3
                    } else {
                        IntegerVector3Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::INT_VEC4 => {
                    if size == 1 {
                        IntegerVector4
                    } else {
                        IntegerVector4Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::UNSIGNED_INT => {
                    if size == 1 {
                        UnsignedInteger
                    } else {
                        UnsignedIntegerArray {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::UNSIGNED_INT_VEC2 => {
                    if size == 1 {
                        UnsignedIntegerVector2
                    } else {
                        UnsignedIntegerVector2Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::UNSIGNED_INT_VEC3 => {
                    if size == 1 {
                        UnsignedIntegerVector3
                    } else {
                        UnsignedIntegerVector3Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::UNSIGNED_INT_VEC4 => {
                    if size == 1 {
                        UnsignedIntegerVector4
                    } else {
                        UnsignedIntegerVector4Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT => {
                    if size == 1 {
                        Float
                    } else {
                        FloatArray {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_VEC2 => {
                    if size == 1 {
                        FloatVector2
                    } else {
                        FloatVector2Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_VEC3 => {
                    if size == 1 {
                        FloatVector3
                    } else {
                        FloatVector3Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_VEC4 => {
                    if size == 1 {
                        FloatVector4
                    } else {
                        FloatVector4Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::BOOL => {
                    if size == 1 {
                        Bool
                    } else {
                        BoolArray {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::BOOL_VEC2 => {
                    if size == 1 {
                        BoolVector2
                    } else {
                        BoolVector2Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::BOOL_VEC3 => {
                    if size == 1 {
                        BoolVector3
                    } else {
                        BoolVector3Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::BOOL_VEC4 => {
                    if size == 1 {
                        BoolVector4
                    } else {
                        BoolVector4Array {
                            stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_MAT2 => {
                    let matrix_stride = matrix_strides[i] as u8;
                    let order = if matrix_orientations[i] == 0 {
                        MatrixOrder::ColumnMajor
                    } else {
                        MatrixOrder::RowMajor
                    };

                    if size == 1 {
                        Matrix2x2 {
                            matrix_stride,
                            order,
                        }
                    } else {
                        Matrix2x2Array {
                            matrix_stride,
                            order,
                            array_stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_MAT3 => {
                    let matrix_stride = matrix_strides[i] as u8;
                    let order = if matrix_orientations[i] == 0 {
                        MatrixOrder::ColumnMajor
                    } else {
                        MatrixOrder::RowMajor
                    };

                    if size == 1 {
                        Matrix3x3 {
                            matrix_stride,
                            order,
                        }
                        .into()
                    } else {
                        Matrix3x3Array {
                            matrix_stride,
                            order,
                            array_stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_MAT4 => {
                    let matrix_stride = matrix_strides[i] as u8;
                    let order = if matrix_orientations[i] == 0 {
                        MatrixOrder::ColumnMajor
                    } else {
                        MatrixOrder::RowMajor
                    };

                    if size == 1 {
                        Matrix4x4 {
                            matrix_stride,
                            order,
                        }
                    } else {
                        Matrix4x4Array {
                            matrix_stride,
                            order,
                            array_stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_MAT2X3 => {
                    let matrix_stride = matrix_strides[i] as u8;
                    let order = if matrix_orientations[i] == 0 {
                        MatrixOrder::ColumnMajor
                    } else {
                        MatrixOrder::RowMajor
                    };

                    if size == 1 {
                        Matrix2x3 {
                            matrix_stride,
                            order,
                        }
                    } else {
                        Matrix2x3Array {
                            matrix_stride,
                            order,
                            array_stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_MAT2X4 => {
                    let matrix_stride = matrix_strides[i] as u8;
                    let order = if matrix_orientations[i] == 0 {
                        MatrixOrder::ColumnMajor
                    } else {
                        MatrixOrder::RowMajor
                    };

                    if size == 1 {
                        Matrix2x4 {
                            matrix_stride,
                            order,
                        }
                    } else {
                        Matrix2x4Array {
                            matrix_stride,
                            order,
                            array_stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_MAT3X2 => {
                    let matrix_stride = matrix_strides[i] as u8;
                    let order = if matrix_orientations[i] == 0 {
                        MatrixOrder::ColumnMajor
                    } else {
                        MatrixOrder::RowMajor
                    };

                    if size == 1 {
                        Matrix3x2 {
                            matrix_stride,
                            order,
                        }
                    } else {
                        Matrix3x2Array {
                            matrix_stride,
                            order,
                            array_stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_MAT3X4 => {
                    let matrix_stride = matrix_strides[i] as u8;
                    let order = if matrix_orientations[i] == 0 {
                        MatrixOrder::ColumnMajor
                    } else {
                        MatrixOrder::RowMajor
                    };

                    if size == 1 {
                        Matrix3x4 {
                            matrix_stride,
                            order,
                        }
                    } else {
                        Matrix3x4Array {
                            matrix_stride,
                            order,
                            array_stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_MAT4X2 => {
                    let matrix_stride = matrix_strides[i] as u8;
                    let order = if matrix_orientations[i] == 0 {
                        MatrixOrder::ColumnMajor
                    } else {
                        MatrixOrder::RowMajor
                    };

                    if size == 1 {
                        Matrix4x2 {
                            matrix_stride,
                            order,
                        }
                    } else {
                        Matrix4x2Array {
                            matrix_stride,
                            order,
                            array_stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                Gl::FLOAT_MAT4X3 => {
                    let matrix_stride = matrix_strides[i] as u8;
                    let order = if matrix_orientations[i] == 0 {
                        MatrixOrder::ColumnMajor
                    } else {
                        MatrixOrder::RowMajor
                    };

                    if size == 1 {
                        Matrix4x3 {
                            matrix_stride,
                            order,
                        }
                    } else {
                        Matrix4x3Array {
                            matrix_stride,
                            order,
                            array_stride: array_strides[i] as u8,
                            len: size as usize,
                        }
                    }
                }
                _ => unreachable!(),
            };

            layout.push(MemoryUnit {
                offset: offsets[i] as usize,
                layout: unit,
            });
        }

        // TODO: unsure if this is ever necessary or if all implementations already guarantee this
        // ordering; may be possible to skip this.
        layout.sort_unstable_by_key(|unit| unit.offset);

        UniformBlockSlot { layout, index }
    }

    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    pub(crate) fn compatibility(
        &self,
        memory_layout: &[MemoryUnit],
    ) -> Result<(), IncompatibleInterface> {
        'outer: for expected_unit in self.layout.iter() {
            'inner: for actual_unit in memory_layout.iter() {
                if expected_unit.offset > actual_unit.offset {
                    return Err(IncompatibleInterface::MissingUnit(*expected_unit));
                } else if expected_unit.offset == actual_unit.offset {
                    if expected_unit.layout == actual_unit.layout {
                        continue 'outer;
                    } else {
                        return Err(IncompatibleInterface::UnitLayoutMismatch(
                            *actual_unit,
                            expected_unit.layout,
                        ));
                    }
                }
            }

            return Err(IncompatibleInterface::MissingUnit(*expected_unit));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum IncompatibleInterface {
    MissingUnit(MemoryUnit),
    UnitLayoutMismatch(MemoryUnit, UnitLayout),
}

#[derive(Debug)]
pub(crate) struct TextureSamplerSlot {
    location: WebGlUniformLocation,
    kind: SampledTextureType,
}

impl TextureSamplerSlot {
    pub(crate) fn new(location: WebGlUniformLocation, kind: SampledTextureType) -> Self {
        TextureSamplerSlot { location, kind }
    }

    pub(crate) fn location(&self) -> &WebGlUniformLocation {
        &self.location
    }

    pub(crate) fn kind(&self) -> SampledTextureType {
        self.kind
    }
}

pub(crate) struct SlotBindingUpdater<'a> {
    gl: &'a Gl,
    program: &'a WebGlProgram,
}

impl<'a> SlotBindingUpdater<'a> {
    pub(crate) fn new(gl: &'a Gl, program: &'a WebGlProgram) -> Self {
        SlotBindingUpdater { gl, program }
    }

    pub(crate) fn update_slot_binding(
        &self,
        descriptor: &ShaderResourceSlotDescriptor,
        binding: u32,
    ) {
        match descriptor.slot_type() {
            SlotType::TextureSampler(slot) => {
                self.gl.uniform1i(Some(slot.location()), binding as i32);
            }
            SlotType::UniformBlock(slot) => {
                self.gl
                    .uniform_block_binding(self.program, slot.index(), binding);
            }
        }
    }
}
