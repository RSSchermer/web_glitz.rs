use std::slice;

use web_sys::{WebGl2RenderingContext as Gl, WebGlProgram, WebGlUniformLocation};

use crate::buffer::{Buffer, BufferView};
use crate::runtime::Connection;
use crate::sampler::{
    FloatSampler2DArrayHandle, FloatSampler2DHandle, FloatSampler3DHandle, FloatSamplerCubeHandle,
    IntegerSampler2DArrayHandle, IntegerSampler2DHandle, IntegerSampler3DHandle,
    IntegerSamplerCubeHandle, Sampler2DArrayShadowHandle, Sampler2DShadowHandle,
    SamplerCubeShadowHandle, UnsignedIntegerSampler2DArrayHandle, UnsignedIntegerSampler2DHandle,
    UnsignedIntegerSampler3DHandle, UnsignedIntegerSamplerCubeHandle,
};
use crate::std_140::Std140;
use crate::util::{slice_make_mut, JsId};

pub enum UniformSlot {
    Float(FloatSlot),
    ArrayOfFloat(ArrayOfFloatSlot),
    FloatVector2(FloatVector2Slot),
    ArrayOfFloatVector2(ArrayOfFloatVector2Slot),
    FloatVector3(FloatVector3Slot),
    ArrayOfFloatVector3(ArrayOfFloatVector3Slot),
    FloatVector4(FloatVector4Slot),
    ArrayOfFloatVector4(ArrayOfFloatVector4Slot),
    FloatMatrix2x2(FloatMatrix2x2Slot),
    ArrayOfFloatMatrix2x2(ArrayOfFloatMatrix2x2Slot),
    FloatMatrix2x3(FloatMatrix2x3Slot),
    ArrayOfFloatMatrix2x3(ArrayOfFloatMatrix2x3Slot),
    FloatMatrix2x4(FloatMatrix2x4Slot),
    ArrayOfFloatMatrix2x4(ArrayOfFloatMatrix2x4Slot),
    FloatMatrix3x2(FloatMatrix3x2Slot),
    ArrayOfFloatMatrix3x2(ArrayOfFloatMatrix3x2Slot),
    FloatMatrix3x3(FloatMatrix3x3Slot),
    ArrayOfFloatMatrix3x3(ArrayOfFloatMatrix3x3Slot),
    FloatMatrix3x4(FloatMatrix3x4Slot),
    ArrayOfFloatMatrix3x4(ArrayOfFloatMatrix3x4Slot),
    FloatMatrix4x2(FloatMatrix4x2Slot),
    ArrayOfFloatMatrix4x2(ArrayOfFloatMatrix4x2Slot),
    FloatMatrix4x3(FloatMatrix4x3Slot),
    ArrayOfFloatMatrix4x3(ArrayOfFloatMatrix4x3Slot),
    FloatMatrix4x4(FloatMatrix4x4Slot),
    ArrayOfFloatMatrix4x4(ArrayOfFloatMatrix4x4Slot),
    Integer(IntegerSlot),
    ArrayOfInteger(ArrayOfIntegerSlot),
    IntegerVector2(IntegerVector2Slot),
    ArrayOfIntegerVector2(ArrayOfIntegerVector2Slot),
    IntegerVector3(IntegerVector3Slot),
    ArrayOfIntegerVector3(ArrayOfIntegerVector3Slot),
    IntegerVector4(IntegerVector4Slot),
    ArrayOfIntegerVector4(ArrayOfIntegerVector4Slot),
    UnsignedInteger(UnsignedIntegerSlot),
    ArrayOfUnsignedInteger(ArrayOfUnsignedIntegerSlot),
    UnsignedIntegerVector2(UnsignedIntegerVector2Slot),
    ArrayOfUnsignedIntegerVector2(ArrayOfUnsignedIntegerVector2Slot),
    UnsignedIntegerVector3(UnsignedIntegerVector3Slot),
    ArrayOfUnsignedIntegerVector3(ArrayOfUnsignedIntegerVector3Slot),
    UnsignedIntegerVector4(UnsignedIntegerVector4Slot),
    ArrayOfUnsignedIntegerVector4(ArrayOfUnsignedIntegerVector4Slot),
    Bool(BoolSlot),
    ArrayOfBool(ArrayOfBoolSlot),
    BoolVector2(BoolVector2Slot),
    ArrayOfBoolVector2(ArrayOfBoolVector2Slot),
    BoolVector3(BoolVector3Slot),
    ArrayOfBoolVector3(ArrayOfBoolVector3Slot),
    BoolVector4(BoolVector4Slot),
    ArrayOfBoolVector4(ArrayOfBoolVector4Slot),
    FloatSampler2D(FloatSampler2DSlot),
    ArrayOfFloatSampler2D(ArrayOfFloatSampler2DSlot),
    IntegerSampler2D(IntegerSampler2DSlot),
    ArrayOfIntegerSampler2D(ArrayOfIntegerSampler2DSlot),
    UnsignedIntegerSampler2D(UnsignedIntegerSampler2DSlot),
    ArrayOfUnsignedIntegerSampler2D(ArrayOfUnsignedIntegerSampler2DSlot),
    FloatSampler2DArray(FloatSampler2DArraySlot),
    ArrayOfFloatSampler2DArray(ArrayOfFloatSampler2DArraySlot),
    IntegerSampler2DArray(IntegerSampler2DArraySlot),
    ArrayOfIntegerSampler2DArray(ArrayOfIntegerSampler2DArraySlot),
    UnsignedIntegerSampler2DArray(UnsignedIntegerSampler2DArraySlot),
    ArrayOfUnsignedIntegerSampler2DArray(ArrayOfUnsignedIntegerSampler2DArraySlot),
    FloatSampler3D(FloatSampler3DSlot),
    ArrayOfFloatSampler3D(ArrayOfFloatSampler3DSlot),
    IntegerSampler3D(IntegerSampler3DSlot),
    ArrayOfIntegerSampler3D(ArrayOfIntegerSampler3DSlot),
    UnsignedIntegerSampler3D(UnsignedIntegerSampler3DSlot),
    ArrayOfUnsignedIntegerSampler3D(ArrayOfUnsignedIntegerSampler3DSlot),
    FloatSamplerCube(FloatSamplerCubeSlot),
    ArrayOfFloatSamplerCube(ArrayOfFloatSamplerCubeSlot),
    IntegerSamplerCube(IntegerSamplerCubeSlot),
    ArrayOfIntegerSamplerCube(ArrayOfIntegerSamplerCubeSlot),
    UnsignedIntegerSamplerCube(UnsignedIntegerSamplerCubeSlot),
    ArrayOfUnsignedIntegerSamplerCube(ArrayOfUnsignedIntegerSamplerCubeSlot),
    Sampler2DShadow(Sampler2DShadowSlot),
    ArrayOfSampler2DShadow(ArrayOfSampler2DShadowSlot),
    Sampler2DArrayShadow(Sampler2DArrayShadowSlot),
    ArrayOfSampler2DArrayShadow(ArrayOfSampler2DArrayShadowSlot),
    SamplerCubeShadow(SamplerCubeShadowSlot),
    ArrayOfSamplerCubeShadow(ArrayOfSamplerCubeShadowSlot),
    Block(BlockSlot),
}

impl UniformSlot {
    pub(crate) fn new_value(
        name: String,
        type_id: u32,
        size: usize,
        location: WebGlUniformLocation,
    ) -> Self {
        let location_id = JsId::from_value(location.into());
        let is_array = name.ends_with("[0]");

        if is_array {
            match type_id {
                Gl::FLOAT => UniformSlot::ArrayOfFloat(ArrayOfFloatSlot { location_id, size }),
                Gl::FLOAT_VEC2 => {
                    UniformSlot::ArrayOfFloatVector2(ArrayOfFloatVector2Slot { location_id, size })
                }
                Gl::FLOAT_VEC3 => {
                    UniformSlot::ArrayOfFloatVector3(ArrayOfFloatVector3Slot { location_id, size })
                }
                Gl::FLOAT_VEC4 => {
                    UniformSlot::ArrayOfFloatVector4(ArrayOfFloatVector4Slot { location_id, size })
                }
                Gl::FLOAT_MAT2 => UniformSlot::ArrayOfFloatMatrix2x2(ArrayOfFloatMatrix2x2Slot {
                    location_id,
                    size,
                }),
                Gl::FLOAT_MAT3 => UniformSlot::ArrayOfFloatMatrix3x3(ArrayOfFloatMatrix3x3Slot {
                    location_id,
                    size,
                }),
                Gl::FLOAT_MAT4 => UniformSlot::ArrayOfFloatMatrix4x4(ArrayOfFloatMatrix4x4Slot {
                    location_id,
                    size,
                }),
                Gl::FLOAT_MAT2X3 => UniformSlot::ArrayOfFloatMatrix2x3(ArrayOfFloatMatrix2x3Slot {
                    location_id,
                    size,
                }),
                Gl::FLOAT_MAT2X4 => UniformSlot::ArrayOfFloatMatrix2x4(ArrayOfFloatMatrix2x4Slot {
                    location_id,
                    size,
                }),
                Gl::FLOAT_MAT3X2 => UniformSlot::ArrayOfFloatMatrix3x2(ArrayOfFloatMatrix3x2Slot {
                    location_id,
                    size,
                }),
                Gl::FLOAT_MAT3X4 => UniformSlot::ArrayOfFloatMatrix3x4(ArrayOfFloatMatrix3x4Slot {
                    location_id,
                    size,
                }),
                Gl::FLOAT_MAT4X2 => UniformSlot::ArrayOfFloatMatrix4x2(ArrayOfFloatMatrix4x2Slot {
                    location_id,
                    size,
                }),
                Gl::FLOAT_MAT4X3 => UniformSlot::ArrayOfFloatMatrix4x3(ArrayOfFloatMatrix4x3Slot {
                    location_id,
                    size,
                }),
                Gl::INT => UniformSlot::ArrayOfInteger(ArrayOfIntegerSlot { location_id, size }),
                Gl::INT_VEC2 => UniformSlot::ArrayOfIntegerVector2(ArrayOfIntegerVector2Slot {
                    location_id,
                    size,
                }),
                Gl::INT_VEC3 => UniformSlot::ArrayOfIntegerVector3(ArrayOfIntegerVector3Slot {
                    location_id,
                    size,
                }),
                Gl::INT_VEC4 => UniformSlot::ArrayOfIntegerVector4(ArrayOfIntegerVector4Slot {
                    location_id,
                    size,
                }),
                Gl::BOOL => UniformSlot::ArrayOfBool(ArrayOfBoolSlot { location_id, size }),
                Gl::BOOL_VEC2 => {
                    UniformSlot::ArrayOfBoolVector2(ArrayOfBoolVector2Slot { location_id, size })
                }
                Gl::BOOL_VEC3 => {
                    UniformSlot::ArrayOfBoolVector3(ArrayOfBoolVector3Slot { location_id, size })
                }
                Gl::BOOL_VEC4 => {
                    UniformSlot::ArrayOfBoolVector4(ArrayOfBoolVector4Slot { location_id, size })
                }
                Gl::UNSIGNED_INT => {
                    UniformSlot::ArrayOfUnsignedInteger(ArrayOfUnsignedIntegerSlot {
                        location_id,
                        size,
                    })
                }
                Gl::UNSIGNED_INT_VEC2 => {
                    UniformSlot::ArrayOfUnsignedIntegerVector2(ArrayOfUnsignedIntegerVector2Slot {
                        location_id,
                        size,
                    })
                }
                Gl::UNSIGNED_INT_VEC3 => {
                    UniformSlot::ArrayOfUnsignedIntegerVector3(ArrayOfUnsignedIntegerVector3Slot {
                        location_id,
                        size,
                    })
                }
                Gl::UNSIGNED_INT_VEC4 => {
                    UniformSlot::ArrayOfUnsignedIntegerVector4(ArrayOfUnsignedIntegerVector4Slot {
                        location_id,
                        size,
                    })
                }
                Gl::SAMPLER_2D => UniformSlot::ArrayOfFloatSampler2D(ArrayOfFloatSampler2DSlot {
                    location_id,
                    size,
                }),
                Gl::SAMPLER_CUBE => {
                    UniformSlot::ArrayOfFloatSamplerCube(ArrayOfFloatSamplerCubeSlot {
                        location_id,
                        size,
                    })
                }
                Gl::SAMPLER_3D => UniformSlot::ArrayOfFloatSampler3D(ArrayOfFloatSampler3DSlot {
                    location_id,
                    size,
                }),
                Gl::SAMPLER_2D_SHADOW => {
                    UniformSlot::ArrayOfSampler2DShadow(ArrayOfSampler2DShadowSlot {
                        location_id,
                        size,
                    })
                }
                Gl::SAMPLER_2D_ARRAY => {
                    UniformSlot::ArrayOfFloatSampler2DArray(ArrayOfFloatSampler2DArraySlot {
                        location_id,
                        size,
                    })
                }
                Gl::SAMPLER_2D_ARRAY_SHADOW => {
                    UniformSlot::ArrayOfSampler2DArrayShadow(ArrayOfSampler2DArrayShadowSlot {
                        location_id,
                        size,
                    })
                }
                Gl::SAMPLER_CUBE_SHADOW => {
                    UniformSlot::ArrayOfSamplerCubeShadow(ArrayOfSamplerCubeShadowSlot {
                        location_id,
                        size,
                    })
                }
                Gl::INT_SAMPLER_2D => {
                    UniformSlot::ArrayOfIntegerSampler2D(ArrayOfIntegerSampler2DSlot {
                        location_id,
                        size,
                    })
                }
                Gl::INT_SAMPLER_3D => {
                    UniformSlot::ArrayOfIntegerSampler3D(ArrayOfIntegerSampler3DSlot {
                        location_id,
                        size,
                    })
                }
                Gl::INT_SAMPLER_CUBE => {
                    UniformSlot::ArrayOfIntegerSamplerCube(ArrayOfIntegerSamplerCubeSlot {
                        location_id,
                        size,
                    })
                }
                Gl::INT_SAMPLER_2D_ARRAY => {
                    UniformSlot::ArrayOfIntegerSampler2DArray(ArrayOfIntegerSampler2DArraySlot {
                        location_id,
                        size,
                    })
                }
                Gl::UNSIGNED_INT_SAMPLER_2D => UniformSlot::ArrayOfUnsignedIntegerSampler2D(
                    ArrayOfUnsignedIntegerSampler2DSlot { location_id, size },
                ),
                Gl::UNSIGNED_INT_SAMPLER_3D => UniformSlot::ArrayOfUnsignedIntegerSampler3D(
                    ArrayOfUnsignedIntegerSampler3DSlot { location_id, size },
                ),
                Gl::UNSIGNED_INT_SAMPLER_CUBE => UniformSlot::ArrayOfUnsignedIntegerSamplerCube(
                    ArrayOfUnsignedIntegerSamplerCubeSlot { location_id, size },
                ),
                Gl::UNSIGNED_INT_SAMPLER_2D_ARRAY => {
                    UniformSlot::ArrayOfUnsignedIntegerSampler2DArray(
                        ArrayOfUnsignedIntegerSampler2DArraySlot { location_id, size },
                    )
                }
                _ => panic!("Invalid uniform type ID"),
            }
        } else {
            match type_id {
                Gl::FLOAT => UniformSlot::Float(FloatSlot {
                    location_id,
                    current_value: 0.0,
                }),
                Gl::FLOAT_VEC2 => UniformSlot::FloatVector2(FloatVector2Slot {
                    location_id,
                    current_value: (0.0, 0.0),
                }),
                Gl::FLOAT_VEC3 => UniformSlot::FloatVector3(FloatVector3Slot {
                    location_id,
                    current_value: (0.0, 0.0, 0.0),
                }),
                Gl::FLOAT_VEC4 => UniformSlot::FloatVector4(FloatVector4Slot {
                    location_id,
                    current_value: (0.0, 0.0, 0.0, 0.0),
                }),
                Gl::FLOAT_MAT2 => UniformSlot::FloatMatrix2x2(FloatMatrix2x2Slot {
                    location_id,
                    current_value: ([0.0; 4], false),
                }),
                Gl::FLOAT_MAT3 => UniformSlot::FloatMatrix3x3(FloatMatrix3x3Slot {
                    location_id,
                    current_value: ([0.0; 9], false),
                }),
                Gl::FLOAT_MAT4 => UniformSlot::FloatMatrix4x4(FloatMatrix4x4Slot {
                    location_id,
                    current_value: ([0.0; 16], false),
                }),
                Gl::FLOAT_MAT2X3 => UniformSlot::FloatMatrix2x3(FloatMatrix2x3Slot {
                    location_id,
                    current_value: ([0.0; 6], false),
                }),
                Gl::FLOAT_MAT2X4 => UniformSlot::FloatMatrix2x4(FloatMatrix2x4Slot {
                    location_id,
                    current_value: ([0.0; 8], false),
                }),
                Gl::FLOAT_MAT3X2 => UniformSlot::FloatMatrix3x2(FloatMatrix3x2Slot {
                    location_id,
                    current_value: ([0.0; 6], false),
                }),
                Gl::FLOAT_MAT3X4 => UniformSlot::FloatMatrix3x4(FloatMatrix3x4Slot {
                    location_id,
                    current_value: ([0.0; 12], false),
                }),
                Gl::FLOAT_MAT4X2 => UniformSlot::FloatMatrix4x2(FloatMatrix4x2Slot {
                    location_id,
                    current_value: ([0.0; 8], false),
                }),
                Gl::FLOAT_MAT4X3 => UniformSlot::FloatMatrix4x3(FloatMatrix4x3Slot {
                    location_id,
                    current_value: ([0.0; 12], false),
                }),
                Gl::INT => UniformSlot::Integer(IntegerSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::INT_VEC2 => UniformSlot::IntegerVector2(IntegerVector2Slot {
                    location_id,
                    current_value: (0, 0),
                }),
                Gl::INT_VEC3 => UniformSlot::IntegerVector3(IntegerVector3Slot {
                    location_id,
                    current_value: (0, 0, 0),
                }),
                Gl::INT_VEC4 => UniformSlot::IntegerVector4(IntegerVector4Slot {
                    location_id,
                    current_value: (0, 0, 0, 0),
                }),
                Gl::BOOL => UniformSlot::Bool(BoolSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::BOOL_VEC2 => UniformSlot::BoolVector2(BoolVector2Slot {
                    location_id,
                    current_value: (0, 0),
                }),
                Gl::BOOL_VEC3 => UniformSlot::BoolVector3(BoolVector3Slot {
                    location_id,
                    current_value: (0, 0, 0),
                }),
                Gl::BOOL_VEC4 => UniformSlot::BoolVector4(BoolVector4Slot {
                    location_id,
                    current_value: (0, 0, 0, 0),
                }),
                Gl::UNSIGNED_INT => UniformSlot::UnsignedInteger(UnsignedIntegerSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::UNSIGNED_INT_VEC2 => {
                    UniformSlot::UnsignedIntegerVector2(UnsignedIntegerVector2Slot {
                        location_id,
                        current_value: (0, 0),
                    })
                }
                Gl::UNSIGNED_INT_VEC3 => {
                    UniformSlot::UnsignedIntegerVector3(UnsignedIntegerVector3Slot {
                        location_id,
                        current_value: (0, 0, 0),
                    })
                }
                Gl::UNSIGNED_INT_VEC4 => {
                    UniformSlot::UnsignedIntegerVector4(UnsignedIntegerVector4Slot {
                        location_id,
                        current_value: (0, 0, 0, 0),
                    })
                }
                Gl::SAMPLER_2D => UniformSlot::FloatSampler2D(FloatSampler2DSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::SAMPLER_CUBE => UniformSlot::FloatSamplerCube(FloatSamplerCubeSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::SAMPLER_3D => UniformSlot::FloatSampler3D(FloatSampler3DSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::SAMPLER_2D_SHADOW => UniformSlot::Sampler2DShadow(Sampler2DShadowSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::SAMPLER_2D_ARRAY => UniformSlot::FloatSampler2DArray(FloatSampler2DArraySlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::SAMPLER_2D_ARRAY_SHADOW => {
                    UniformSlot::Sampler2DArrayShadow(Sampler2DArrayShadowSlot {
                        location_id,
                        current_value: 0,
                    })
                }
                Gl::SAMPLER_CUBE_SHADOW => UniformSlot::SamplerCubeShadow(SamplerCubeShadowSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::INT_SAMPLER_2D => UniformSlot::IntegerSampler2D(IntegerSampler2DSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::INT_SAMPLER_3D => UniformSlot::IntegerSampler3D(IntegerSampler3DSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::INT_SAMPLER_CUBE => UniformSlot::IntegerSamplerCube(IntegerSamplerCubeSlot {
                    location_id,
                    current_value: 0,
                }),
                Gl::INT_SAMPLER_2D_ARRAY => {
                    UniformSlot::IntegerSampler2DArray(IntegerSampler2DArraySlot {
                        location_id,
                        current_value: 0,
                    })
                }
                Gl::UNSIGNED_INT_SAMPLER_2D => {
                    UniformSlot::UnsignedIntegerSampler2D(UnsignedIntegerSampler2DSlot {
                        location_id,
                        current_value: 0,
                    })
                }
                Gl::UNSIGNED_INT_SAMPLER_3D => {
                    UniformSlot::UnsignedIntegerSampler3D(UnsignedIntegerSampler3DSlot {
                        location_id,
                        current_value: 0,
                    })
                }
                Gl::UNSIGNED_INT_SAMPLER_CUBE => {
                    UniformSlot::UnsignedIntegerSamplerCube(UnsignedIntegerSamplerCubeSlot {
                        location_id,
                        current_value: 0,
                    })
                }
                Gl::UNSIGNED_INT_SAMPLER_2D_ARRAY => {
                    UniformSlot::UnsignedIntegerSampler2DArray(UnsignedIntegerSampler2DArraySlot {
                        location_id,
                        current_value: 0,
                    })
                }
                _ => panic!("Invalid uniform type ID"),
            }
        }
    }

    pub(crate) fn new_block(program: &WebGlProgram, index: u32) -> Self {
        UniformSlot::Block(BlockSlot {
            program_id: JsId::from_value(program.clone().into()),
            index,
            current_binding: 0,
        })
    }

    pub fn as_bindable<'a>(&'a mut self, connection: &'a mut Connection) -> BindingSlot<'a> {
        match self {
            UniformSlot::Float(slot) => BindingSlot::Float(Binder { slot, connection }),
            UniformSlot::ArrayOfFloat(slot) => {
                BindingSlot::ArrayOfFloat(Binder { slot, connection })
            }
            UniformSlot::FloatVector2(slot) => {
                BindingSlot::FloatVector2(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatVector2(slot) => {
                BindingSlot::ArrayOfFloatVector2(Binder { slot, connection })
            }
            UniformSlot::FloatVector3(slot) => {
                BindingSlot::FloatVector3(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatVector3(slot) => {
                BindingSlot::ArrayOfFloatVector3(Binder { slot, connection })
            }
            UniformSlot::FloatVector4(slot) => {
                BindingSlot::FloatVector4(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatVector4(slot) => {
                BindingSlot::ArrayOfFloatVector4(Binder { slot, connection })
            }
            UniformSlot::FloatMatrix2x2(slot) => {
                BindingSlot::FloatMatrix2x2(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatMatrix2x2(slot) => {
                BindingSlot::ArrayOfFloatMatrix2x2(Binder { slot, connection })
            }
            UniformSlot::FloatMatrix2x3(slot) => {
                BindingSlot::FloatMatrix2x3(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatMatrix2x3(slot) => {
                BindingSlot::ArrayOfFloatMatrix2x3(Binder { slot, connection })
            }
            UniformSlot::FloatMatrix2x4(slot) => {
                BindingSlot::FloatMatrix2x4(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatMatrix2x4(slot) => {
                BindingSlot::ArrayOfFloatMatrix2x4(Binder { slot, connection })
            }
            UniformSlot::FloatMatrix3x2(slot) => {
                BindingSlot::FloatMatrix3x2(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatMatrix3x2(slot) => {
                BindingSlot::ArrayOfFloatMatrix3x2(Binder { slot, connection })
            }
            UniformSlot::FloatMatrix3x3(slot) => {
                BindingSlot::FloatMatrix3x3(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatMatrix3x3(slot) => {
                BindingSlot::ArrayOfFloatMatrix3x3(Binder { slot, connection })
            }
            UniformSlot::FloatMatrix3x4(slot) => {
                BindingSlot::FloatMatrix3x4(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatMatrix3x4(slot) => {
                BindingSlot::ArrayOfFloatMatrix3x4(Binder { slot, connection })
            }
            UniformSlot::FloatMatrix4x2(slot) => {
                BindingSlot::FloatMatrix4x2(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatMatrix4x2(slot) => {
                BindingSlot::ArrayOfFloatMatrix4x2(Binder { slot, connection })
            }
            UniformSlot::FloatMatrix4x3(slot) => {
                BindingSlot::FloatMatrix4x3(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatMatrix4x3(slot) => {
                BindingSlot::ArrayOfFloatMatrix4x3(Binder { slot, connection })
            }
            UniformSlot::FloatMatrix4x4(slot) => {
                BindingSlot::FloatMatrix4x4(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatMatrix4x4(slot) => {
                BindingSlot::ArrayOfFloatMatrix4x4(Binder { slot, connection })
            }
            UniformSlot::Integer(slot) => BindingSlot::Integer(Binder { slot, connection }),
            UniformSlot::ArrayOfInteger(slot) => {
                BindingSlot::ArrayOfInteger(Binder { slot, connection })
            }
            UniformSlot::IntegerVector2(slot) => {
                BindingSlot::IntegerVector2(Binder { slot, connection })
            }
            UniformSlot::ArrayOfIntegerVector2(slot) => {
                BindingSlot::ArrayOfIntegerVector2(Binder { slot, connection })
            }
            UniformSlot::IntegerVector3(slot) => {
                BindingSlot::IntegerVector3(Binder { slot, connection })
            }
            UniformSlot::ArrayOfIntegerVector3(slot) => {
                BindingSlot::ArrayOfIntegerVector3(Binder { slot, connection })
            }
            UniformSlot::IntegerVector4(slot) => {
                BindingSlot::IntegerVector4(Binder { slot, connection })
            }
            UniformSlot::ArrayOfIntegerVector4(slot) => {
                BindingSlot::ArrayOfIntegerVector4(Binder { slot, connection })
            }
            UniformSlot::UnsignedInteger(slot) => {
                BindingSlot::UnsignedInteger(Binder { slot, connection })
            }
            UniformSlot::ArrayOfUnsignedInteger(slot) => {
                BindingSlot::ArrayOfUnsignedInteger(Binder { slot, connection })
            }
            UniformSlot::UnsignedIntegerVector2(slot) => {
                BindingSlot::UnsignedIntegerVector2(Binder { slot, connection })
            }
            UniformSlot::ArrayOfUnsignedIntegerVector2(slot) => {
                BindingSlot::ArrayOfUnsignedIntegerVector2(Binder { slot, connection })
            }
            UniformSlot::UnsignedIntegerVector3(slot) => {
                BindingSlot::UnsignedIntegerVector3(Binder { slot, connection })
            }
            UniformSlot::ArrayOfUnsignedIntegerVector3(slot) => {
                BindingSlot::ArrayOfUnsignedIntegerVector3(Binder { slot, connection })
            }
            UniformSlot::UnsignedIntegerVector4(slot) => {
                BindingSlot::UnsignedIntegerVector4(Binder { slot, connection })
            }
            UniformSlot::ArrayOfUnsignedIntegerVector4(slot) => {
                BindingSlot::ArrayOfUnsignedIntegerVector4(Binder { slot, connection })
            }
            UniformSlot::Bool(slot) => BindingSlot::Bool(Binder { slot, connection }),
            UniformSlot::ArrayOfBool(slot) => BindingSlot::ArrayOfBool(Binder { slot, connection }),
            UniformSlot::BoolVector2(slot) => BindingSlot::BoolVector2(Binder { slot, connection }),
            UniformSlot::ArrayOfBoolVector2(slot) => {
                BindingSlot::ArrayOfBoolVector2(Binder { slot, connection })
            }
            UniformSlot::BoolVector3(slot) => BindingSlot::BoolVector3(Binder { slot, connection }),
            UniformSlot::ArrayOfBoolVector3(slot) => {
                BindingSlot::ArrayOfBoolVector3(Binder { slot, connection })
            }
            UniformSlot::BoolVector4(slot) => BindingSlot::BoolVector4(Binder { slot, connection }),
            UniformSlot::ArrayOfBoolVector4(slot) => {
                BindingSlot::ArrayOfBoolVector4(Binder { slot, connection })
            }
            UniformSlot::FloatSampler2D(slot) => {
                BindingSlot::FloatSampler2D(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatSampler2D(slot) => {
                BindingSlot::ArrayOfFloatSampler2D(Binder { slot, connection })
            }
            UniformSlot::IntegerSampler2D(slot) => {
                BindingSlot::IntegerSampler2D(Binder { slot, connection })
            }
            UniformSlot::ArrayOfIntegerSampler2D(slot) => {
                BindingSlot::ArrayOfIntegerSampler2D(Binder { slot, connection })
            }
            UniformSlot::UnsignedIntegerSampler2D(slot) => {
                BindingSlot::UnsignedIntegerSampler2D(Binder { slot, connection })
            }
            UniformSlot::ArrayOfUnsignedIntegerSampler2D(slot) => {
                BindingSlot::ArrayOfUnsignedIntegerSampler2D(Binder { slot, connection })
            }
            UniformSlot::FloatSampler2DArray(slot) => {
                BindingSlot::FloatSampler2DArray(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatSampler2DArray(slot) => {
                BindingSlot::ArrayOfFloatSampler2DArray(Binder { slot, connection })
            }
            UniformSlot::IntegerSampler2DArray(slot) => {
                BindingSlot::IntegerSampler2DArray(Binder { slot, connection })
            }
            UniformSlot::ArrayOfIntegerSampler2DArray(slot) => {
                BindingSlot::ArrayOfIntegerSampler2DArray(Binder { slot, connection })
            }
            UniformSlot::UnsignedIntegerSampler2DArray(slot) => {
                BindingSlot::UnsignedIntegerSampler2DArray(Binder { slot, connection })
            }
            UniformSlot::ArrayOfUnsignedIntegerSampler2DArray(slot) => {
                BindingSlot::ArrayOfUnsignedIntegerSampler2DArray(Binder { slot, connection })
            }
            UniformSlot::FloatSampler3D(slot) => {
                BindingSlot::FloatSampler3D(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatSampler3D(slot) => {
                BindingSlot::ArrayOfFloatSampler3D(Binder { slot, connection })
            }
            UniformSlot::IntegerSampler3D(slot) => {
                BindingSlot::IntegerSampler3D(Binder { slot, connection })
            }
            UniformSlot::ArrayOfIntegerSampler3D(slot) => {
                BindingSlot::ArrayOfIntegerSampler3D(Binder { slot, connection })
            }
            UniformSlot::UnsignedIntegerSampler3D(slot) => {
                BindingSlot::UnsignedIntegerSampler3D(Binder { slot, connection })
            }
            UniformSlot::ArrayOfUnsignedIntegerSampler3D(slot) => {
                BindingSlot::ArrayOfUnsignedIntegerSampler3D(Binder { slot, connection })
            }
            UniformSlot::FloatSamplerCube(slot) => {
                BindingSlot::FloatSamplerCube(Binder { slot, connection })
            }
            UniformSlot::ArrayOfFloatSamplerCube(slot) => {
                BindingSlot::ArrayOfFloatSamplerCube(Binder { slot, connection })
            }
            UniformSlot::IntegerSamplerCube(slot) => {
                BindingSlot::IntegerSamplerCube(Binder { slot, connection })
            }
            UniformSlot::ArrayOfIntegerSamplerCube(slot) => {
                BindingSlot::ArrayOfIntegerSamplerCube(Binder { slot, connection })
            }
            UniformSlot::UnsignedIntegerSamplerCube(slot) => {
                BindingSlot::UnsignedIntegerSamplerCube(Binder { slot, connection })
            }
            UniformSlot::ArrayOfUnsignedIntegerSamplerCube(slot) => {
                BindingSlot::ArrayOfUnsignedIntegerSamplerCube(Binder { slot, connection })
            }
            UniformSlot::Sampler2DShadow(slot) => {
                BindingSlot::Sampler2DShadow(Binder { slot, connection })
            }
            UniformSlot::ArrayOfSampler2DShadow(slot) => {
                BindingSlot::ArrayOfSampler2DShadow(Binder { slot, connection })
            }
            UniformSlot::Sampler2DArrayShadow(slot) => {
                BindingSlot::Sampler2DArrayShadow(Binder { slot, connection })
            }
            UniformSlot::ArrayOfSampler2DArrayShadow(slot) => {
                BindingSlot::ArrayOfSampler2DArrayShadow(Binder { slot, connection })
            }
            UniformSlot::SamplerCubeShadow(slot) => {
                BindingSlot::SamplerCubeShadow(Binder { slot, connection })
            }
            UniformSlot::ArrayOfSamplerCubeShadow(slot) => {
                BindingSlot::ArrayOfSamplerCubeShadow(Binder { slot, connection })
            }
            UniformSlot::Block(slot) => BindingSlot::Block(Binder { slot, connection }),
        }
    }
}

// Note on the naming convention for slots: array slots are prefixed with `ArrayOf`, rather than
// suffixed with `Array`, because such suffixing would cause name clashing with `Sampler2DArray`
// types.

pub struct FloatSlot {
    location_id: JsId,
    current_value: f32,
}

pub struct ArrayOfFloatSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatVector2Slot {
    location_id: JsId,
    current_value: (f32, f32),
}

pub struct ArrayOfFloatVector2Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatVector2Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatVector3Slot {
    location_id: JsId,
    current_value: (f32, f32, f32),
}

pub struct ArrayOfFloatVector3Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatVector3Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatVector4Slot {
    location_id: JsId,
    current_value: (f32, f32, f32, f32),
}

pub struct ArrayOfFloatVector4Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatVector4Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatMatrix2x2Slot {
    location_id: JsId,
    current_value: ([f32; 4], bool),
}

pub struct ArrayOfFloatMatrix2x2Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatMatrix2x2Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatMatrix2x3Slot {
    location_id: JsId,
    current_value: ([f32; 6], bool),
}

pub struct ArrayOfFloatMatrix2x3Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatMatrix2x3Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatMatrix2x4Slot {
    location_id: JsId,
    current_value: ([f32; 8], bool),
}

pub struct ArrayOfFloatMatrix2x4Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatMatrix2x4Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatMatrix3x2Slot {
    location_id: JsId,
    current_value: ([f32; 6], bool),
}

pub struct ArrayOfFloatMatrix3x2Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatMatrix3x2Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatMatrix3x3Slot {
    location_id: JsId,
    current_value: ([f32; 9], bool),
}

pub struct ArrayOfFloatMatrix3x3Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatMatrix3x3Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatMatrix3x4Slot {
    location_id: JsId,
    current_value: ([f32; 12], bool),
}

pub struct ArrayOfFloatMatrix3x4Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatMatrix3x4Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatMatrix4x2Slot {
    location_id: JsId,
    current_value: ([f32; 8], bool),
}

pub struct ArrayOfFloatMatrix4x2Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatMatrix4x2Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatMatrix4x3Slot {
    location_id: JsId,
    current_value: ([f32; 12], bool),
}

pub struct ArrayOfFloatMatrix4x3Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatMatrix4x3Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatMatrix4x4Slot {
    location_id: JsId,
    current_value: ([f32; 16], bool),
}

pub struct ArrayOfFloatMatrix4x4Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatMatrix4x4Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct IntegerSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfIntegerSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfIntegerSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct IntegerVector2Slot {
    location_id: JsId,
    current_value: (i32, i32),
}

pub struct ArrayOfIntegerVector2Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfIntegerVector2Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct IntegerVector3Slot {
    location_id: JsId,
    current_value: (i32, i32, i32),
}

pub struct ArrayOfIntegerVector3Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfIntegerVector3Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct IntegerVector4Slot {
    location_id: JsId,
    current_value: (i32, i32, i32, i32),
}

pub struct ArrayOfIntegerVector4Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfIntegerVector4Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct UnsignedIntegerSlot {
    location_id: JsId,
    current_value: u32,
}

pub struct ArrayOfUnsignedIntegerSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfUnsignedIntegerSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct UnsignedIntegerVector2Slot {
    location_id: JsId,
    current_value: (u32, u32),
}

pub struct ArrayOfUnsignedIntegerVector2Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfUnsignedIntegerVector2Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct UnsignedIntegerVector3Slot {
    location_id: JsId,
    current_value: (u32, u32, u32),
}

pub struct ArrayOfUnsignedIntegerVector3Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfUnsignedIntegerVector3Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct UnsignedIntegerVector4Slot {
    location_id: JsId,
    current_value: (u32, u32, u32, u32),
}

pub struct ArrayOfUnsignedIntegerVector4Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfUnsignedIntegerVector4Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct BoolSlot {
    location_id: JsId,
    current_value: u32,
}

pub struct ArrayOfBoolSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfBoolSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct BoolVector2Slot {
    location_id: JsId,
    current_value: (u32, u32),
}

pub struct ArrayOfBoolVector2Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfBoolVector2Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct BoolVector3Slot {
    location_id: JsId,
    current_value: (u32, u32, u32),
}

pub struct ArrayOfBoolVector3Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfBoolVector3Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct BoolVector4Slot {
    location_id: JsId,
    current_value: (u32, u32, u32, u32),
}

pub struct ArrayOfBoolVector4Slot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfBoolVector4Slot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatSampler2DSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfFloatSampler2DSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatSampler2DSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct IntegerSampler2DSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfIntegerSampler2DSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfIntegerSampler2DSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct UnsignedIntegerSampler2DSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfUnsignedIntegerSampler2DSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfUnsignedIntegerSampler2DSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatSampler2DArraySlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfFloatSampler2DArraySlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatSampler2DArraySlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct IntegerSampler2DArraySlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfIntegerSampler2DArraySlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfIntegerSampler2DArraySlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct UnsignedIntegerSampler2DArraySlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfUnsignedIntegerSampler2DArraySlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfUnsignedIntegerSampler2DArraySlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatSampler3DSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfFloatSampler3DSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatSampler3DSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct IntegerSampler3DSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfIntegerSampler3DSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfIntegerSampler3DSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct UnsignedIntegerSampler3DSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfUnsignedIntegerSampler3DSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfUnsignedIntegerSampler3DSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FloatSamplerCubeSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfFloatSamplerCubeSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfFloatSamplerCubeSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct IntegerSamplerCubeSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfIntegerSamplerCubeSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfIntegerSamplerCubeSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct UnsignedIntegerSamplerCubeSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfUnsignedIntegerSamplerCubeSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfUnsignedIntegerSamplerCubeSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct Sampler2DShadowSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfSampler2DShadowSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfSampler2DShadowSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct Sampler2DArrayShadowSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfSampler2DArrayShadowSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfSampler2DArrayShadowSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct SamplerCubeShadowSlot {
    location_id: JsId,
    current_value: i32,
}

pub struct ArrayOfSamplerCubeShadowSlot {
    location_id: JsId,
    size: usize,
}

impl ArrayOfSamplerCubeShadowSlot {
    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct BlockSlot {
    program_id: JsId,
    index: u32,
    current_binding: u32,
}

pub enum BindingSlot<'a> {
    Float(Binder<'a, FloatSlot>),
    ArrayOfFloat(Binder<'a, ArrayOfFloatSlot>),
    FloatVector2(Binder<'a, FloatVector2Slot>),
    ArrayOfFloatVector2(Binder<'a, ArrayOfFloatVector2Slot>),
    FloatVector3(Binder<'a, FloatVector3Slot>),
    ArrayOfFloatVector3(Binder<'a, ArrayOfFloatVector3Slot>),
    FloatVector4(Binder<'a, FloatVector4Slot>),
    ArrayOfFloatVector4(Binder<'a, ArrayOfFloatVector4Slot>),
    FloatMatrix2x2(Binder<'a, FloatMatrix2x2Slot>),
    ArrayOfFloatMatrix2x2(Binder<'a, ArrayOfFloatMatrix2x2Slot>),
    FloatMatrix2x3(Binder<'a, FloatMatrix2x3Slot>),
    ArrayOfFloatMatrix2x3(Binder<'a, ArrayOfFloatMatrix2x3Slot>),
    FloatMatrix2x4(Binder<'a, FloatMatrix2x4Slot>),
    ArrayOfFloatMatrix2x4(Binder<'a, ArrayOfFloatMatrix2x4Slot>),
    FloatMatrix3x2(Binder<'a, FloatMatrix3x2Slot>),
    ArrayOfFloatMatrix3x2(Binder<'a, ArrayOfFloatMatrix3x2Slot>),
    FloatMatrix3x3(Binder<'a, FloatMatrix3x3Slot>),
    ArrayOfFloatMatrix3x3(Binder<'a, ArrayOfFloatMatrix3x3Slot>),
    FloatMatrix3x4(Binder<'a, FloatMatrix3x4Slot>),
    ArrayOfFloatMatrix3x4(Binder<'a, ArrayOfFloatMatrix3x4Slot>),
    FloatMatrix4x2(Binder<'a, FloatMatrix4x2Slot>),
    ArrayOfFloatMatrix4x2(Binder<'a, ArrayOfFloatMatrix4x2Slot>),
    FloatMatrix4x3(Binder<'a, FloatMatrix4x3Slot>),
    ArrayOfFloatMatrix4x3(Binder<'a, ArrayOfFloatMatrix4x3Slot>),
    FloatMatrix4x4(Binder<'a, FloatMatrix4x4Slot>),
    ArrayOfFloatMatrix4x4(Binder<'a, ArrayOfFloatMatrix4x4Slot>),
    Integer(Binder<'a, IntegerSlot>),
    ArrayOfInteger(Binder<'a, ArrayOfIntegerSlot>),
    IntegerVector2(Binder<'a, IntegerVector2Slot>),
    ArrayOfIntegerVector2(Binder<'a, ArrayOfIntegerVector2Slot>),
    IntegerVector3(Binder<'a, IntegerVector3Slot>),
    ArrayOfIntegerVector3(Binder<'a, ArrayOfIntegerVector3Slot>),
    IntegerVector4(Binder<'a, IntegerVector4Slot>),
    ArrayOfIntegerVector4(Binder<'a, ArrayOfIntegerVector4Slot>),
    UnsignedInteger(Binder<'a, UnsignedIntegerSlot>),
    ArrayOfUnsignedInteger(Binder<'a, ArrayOfUnsignedIntegerSlot>),
    UnsignedIntegerVector2(Binder<'a, UnsignedIntegerVector2Slot>),
    ArrayOfUnsignedIntegerVector2(Binder<'a, ArrayOfUnsignedIntegerVector2Slot>),
    UnsignedIntegerVector3(Binder<'a, UnsignedIntegerVector3Slot>),
    ArrayOfUnsignedIntegerVector3(Binder<'a, ArrayOfUnsignedIntegerVector3Slot>),
    UnsignedIntegerVector4(Binder<'a, UnsignedIntegerVector4Slot>),
    ArrayOfUnsignedIntegerVector4(Binder<'a, ArrayOfUnsignedIntegerVector4Slot>),
    Bool(Binder<'a, BoolSlot>),
    ArrayOfBool(Binder<'a, ArrayOfBoolSlot>),
    BoolVector2(Binder<'a, BoolVector2Slot>),
    ArrayOfBoolVector2(Binder<'a, ArrayOfBoolVector2Slot>),
    BoolVector3(Binder<'a, BoolVector3Slot>),
    ArrayOfBoolVector3(Binder<'a, ArrayOfBoolVector3Slot>),
    BoolVector4(Binder<'a, BoolVector4Slot>),
    ArrayOfBoolVector4(Binder<'a, ArrayOfBoolVector4Slot>),
    FloatSampler2D(Binder<'a, FloatSampler2DSlot>),
    ArrayOfFloatSampler2D(Binder<'a, ArrayOfFloatSampler2DSlot>),
    IntegerSampler2D(Binder<'a, IntegerSampler2DSlot>),
    ArrayOfIntegerSampler2D(Binder<'a, ArrayOfIntegerSampler2DSlot>),
    UnsignedIntegerSampler2D(Binder<'a, UnsignedIntegerSampler2DSlot>),
    ArrayOfUnsignedIntegerSampler2D(Binder<'a, ArrayOfUnsignedIntegerSampler2DSlot>),
    FloatSampler2DArray(Binder<'a, FloatSampler2DArraySlot>),
    ArrayOfFloatSampler2DArray(Binder<'a, ArrayOfFloatSampler2DArraySlot>),
    IntegerSampler2DArray(Binder<'a, IntegerSampler2DArraySlot>),
    ArrayOfIntegerSampler2DArray(Binder<'a, ArrayOfIntegerSampler2DArraySlot>),
    UnsignedIntegerSampler2DArray(Binder<'a, UnsignedIntegerSampler2DArraySlot>),
    ArrayOfUnsignedIntegerSampler2DArray(Binder<'a, ArrayOfUnsignedIntegerSampler2DArraySlot>),
    FloatSampler3D(Binder<'a, FloatSampler3DSlot>),
    ArrayOfFloatSampler3D(Binder<'a, ArrayOfFloatSampler3DSlot>),
    IntegerSampler3D(Binder<'a, IntegerSampler3DSlot>),
    ArrayOfIntegerSampler3D(Binder<'a, ArrayOfIntegerSampler3DSlot>),
    UnsignedIntegerSampler3D(Binder<'a, UnsignedIntegerSampler3DSlot>),
    ArrayOfUnsignedIntegerSampler3D(Binder<'a, ArrayOfUnsignedIntegerSampler3DSlot>),
    FloatSamplerCube(Binder<'a, FloatSamplerCubeSlot>),
    ArrayOfFloatSamplerCube(Binder<'a, ArrayOfFloatSamplerCubeSlot>),
    IntegerSamplerCube(Binder<'a, IntegerSamplerCubeSlot>),
    ArrayOfIntegerSamplerCube(Binder<'a, ArrayOfIntegerSamplerCubeSlot>),
    UnsignedIntegerSamplerCube(Binder<'a, UnsignedIntegerSamplerCubeSlot>),
    ArrayOfUnsignedIntegerSamplerCube(Binder<'a, ArrayOfUnsignedIntegerSamplerCubeSlot>),
    Sampler2DShadow(Binder<'a, Sampler2DShadowSlot>),
    ArrayOfSampler2DShadow(Binder<'a, ArrayOfSampler2DShadowSlot>),
    Sampler2DArrayShadow(Binder<'a, Sampler2DArrayShadowSlot>),
    ArrayOfSampler2DArrayShadow(Binder<'a, ArrayOfSampler2DArrayShadowSlot>),
    SamplerCubeShadow(Binder<'a, SamplerCubeShadowSlot>),
    ArrayOfSamplerCubeShadow(Binder<'a, ArrayOfSamplerCubeShadowSlot>),
    Block(Binder<'a, BlockSlot>),
}

pub struct Binder<'a, T> {
    connection: &'a mut Connection,
    slot: &'a mut T,
}

impl<'a, T> Binder<'a, T> {
    pub fn slot(&self) -> &T {
        self.slot
    }
}

impl<'a> Binder<'a, FloatSlot> {
    pub fn bind(&mut self, value: f32) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1f(Some(&location), value);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatSlot> {
    pub fn bind(&mut self, value: &[f32]) {
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1fv_with_f32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, FloatVector2Slot> {
    pub fn bind(&mut self, value: (f32, f32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform2f(Some(&location), value.0, value.1);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatVector2Slot> {
    pub fn bind(&mut self, value: &[(f32, f32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 2);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform2fv_with_f32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, FloatVector3Slot> {
    pub fn bind(&mut self, value: (f32, f32, f32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform3f(Some(&location), value.0, value.1, value.2);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatVector3Slot> {
    pub fn bind(&mut self, value: &[(f32, f32, f32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 3);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform3fv_with_f32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, FloatVector4Slot> {
    pub fn bind(&mut self, value: (f32, f32, f32, f32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform4f(Some(&location), value.0, value.1, value.2, value.3);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatVector4Slot> {
    pub fn bind(&mut self, value: &[(f32, f32, f32, f32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 4);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform4fv_with_f32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, FloatMatrix2x2Slot> {
    pub fn bind(&mut self, mut value: [f32; 4], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != (value, transpose) {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform_matrix2fv_with_f32_array(Some(&location), transpose, &mut value);
                });
            }

            self.slot.current_value = (value, transpose);
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatMatrix2x2Slot> {
    pub fn bind(&mut self, value: &[[f32; 4]], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 4);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform_matrix2fv_with_f32_array(
                    Some(&location),
                    transpose,
                    slice_make_mut(value),
                );
            });
        }
    }
}

impl<'a> Binder<'a, FloatMatrix2x3Slot> {
    pub fn bind(&mut self, mut value: [f32; 6], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != (value, transpose) {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform_matrix2x3fv_with_f32_array(Some(&location), transpose, &mut value);
                });
            }

            self.slot.current_value = (value, transpose);
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatMatrix2x3Slot> {
    pub fn bind(&mut self, value: &[[f32; 6]], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 6);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform_matrix2x3fv_with_f32_array(
                    Some(&location),
                    transpose,
                    slice_make_mut(value),
                );
            });
        }
    }
}

impl<'a> Binder<'a, FloatMatrix2x4Slot> {
    pub fn bind(&mut self, mut value: [f32; 8], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != (value, transpose) {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform_matrix2x4fv_with_f32_array(Some(&location), transpose, &mut value);
                });
            }

            self.slot.current_value = (value, transpose);
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatMatrix2x4Slot> {
    pub fn bind(&mut self, value: &[[f32; 8]], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 8);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform_matrix2x4fv_with_f32_array(
                    Some(&location),
                    transpose,
                    slice_make_mut(value),
                );
            });
        }
    }
}

impl<'a> Binder<'a, FloatMatrix3x2Slot> {
    pub fn bind(&mut self, mut value: [f32; 6], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != (value, transpose) {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform_matrix3x2fv_with_f32_array(Some(&location), transpose, &mut value);
                });
            }

            self.slot.current_value = (value, transpose);
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatMatrix3x2Slot> {
    pub fn bind(&mut self, value: &[[f32; 6]], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 6);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform_matrix3x2fv_with_f32_array(
                    Some(&location),
                    transpose,
                    slice_make_mut(value),
                );
            });
        }
    }
}

impl<'a> Binder<'a, FloatMatrix3x3Slot> {
    pub fn bind(&mut self, mut value: [f32; 9], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != (value, transpose) {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform_matrix3fv_with_f32_array(Some(&location), transpose, &mut value);
                });
            }

            self.slot.current_value = (value, transpose);
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatMatrix3x3Slot> {
    pub fn bind(&mut self, value: &[[f32; 9]], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 9);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform_matrix3fv_with_f32_array(
                    Some(&location),
                    transpose,
                    slice_make_mut(value),
                );
            });
        }
    }
}

impl<'a> Binder<'a, FloatMatrix3x4Slot> {
    pub fn bind(&mut self, mut value: [f32; 12], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != (value, transpose) {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform_matrix3x4fv_with_f32_array(Some(&location), transpose, &mut value);
                });
            }

            self.slot.current_value = (value, transpose);
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatMatrix3x4Slot> {
    pub fn bind(&mut self, value: &[[f32; 12]], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 12);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform_matrix3x4fv_with_f32_array(
                    Some(&location),
                    transpose,
                    slice_make_mut(value),
                );
            });
        }
    }
}

impl<'a> Binder<'a, FloatMatrix4x2Slot> {
    pub fn bind(&mut self, mut value: [f32; 8], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != (value, transpose) {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform_matrix4x2fv_with_f32_array(Some(&location), transpose, &mut value);
                });
            }

            self.slot.current_value = (value, transpose);
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatMatrix4x2Slot> {
    pub fn bind(&mut self, value: &[[f32; 8]], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 8);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform_matrix4x2fv_with_f32_array(
                    Some(&location),
                    transpose,
                    slice_make_mut(value),
                );
            });
        }
    }
}

impl<'a> Binder<'a, FloatMatrix4x3Slot> {
    pub fn bind(&mut self, mut value: [f32; 12], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != (value, transpose) {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform_matrix4x3fv_with_f32_array(Some(&location), transpose, &mut value);
                });
            }

            self.slot.current_value = (value, transpose);
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatMatrix4x3Slot> {
    pub fn bind(&mut self, value: &[[f32; 12]], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 12);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform_matrix4x3fv_with_f32_array(
                    Some(&location),
                    transpose,
                    slice_make_mut(value),
                );
            });
        }
    }
}

impl<'a> Binder<'a, FloatMatrix4x4Slot> {
    pub fn bind(&mut self, mut value: [f32; 16], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != (value, transpose) {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform_matrix4fv_with_f32_array(Some(&location), transpose, &mut value);
                });
            }

            self.slot.current_value = (value, transpose);
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatMatrix4x4Slot> {
    pub fn bind(&mut self, value: &[[f32; 16]], transpose: bool) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 16);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform_matrix4fv_with_f32_array(
                    Some(&location),
                    transpose,
                    slice_make_mut(value),
                );
            });
        }
    }
}

impl<'a> Binder<'a, IntegerSlot> {
    pub fn bind(&mut self, value: i32) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), value);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfIntegerSlot> {
    pub fn bind(&mut self, value: &[i32]) {
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, IntegerVector2Slot> {
    pub fn bind(&mut self, value: (i32, i32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform2i(Some(&location), value.0, value.1);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfIntegerVector2Slot> {
    pub fn bind(&mut self, value: &[(i32, i32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const i32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 2);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform2iv_with_i32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, IntegerVector3Slot> {
    pub fn bind(&mut self, value: (i32, i32, i32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform3i(Some(&location), value.0, value.1, value.2);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfIntegerVector3Slot> {
    pub fn bind(&mut self, value: &[(i32, i32, i32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const i32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 3);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform3iv_with_i32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, IntegerVector4Slot> {
    pub fn bind(&mut self, value: (i32, i32, i32, i32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform4i(Some(&location), value.0, value.1, value.2, value.3);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfIntegerVector4Slot> {
    pub fn bind(&mut self, value: &[(i32, i32, i32, i32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const i32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 4);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform4iv_with_i32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, UnsignedIntegerSlot> {
    pub fn bind(&mut self, value: u32) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1ui(Some(&location), value);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfUnsignedIntegerSlot> {
    pub fn bind(&mut self, value: &[u32]) {
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, UnsignedIntegerVector2Slot> {
    pub fn bind(&mut self, value: (u32, u32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform2ui(Some(&location), value.0, value.1);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfUnsignedIntegerVector2Slot> {
    pub fn bind(&mut self, value: &[(u32, u32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 2);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform2uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, UnsignedIntegerVector3Slot> {
    pub fn bind(&mut self, value: (u32, u32, u32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform3ui(Some(&location), value.0, value.1, value.2);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfUnsignedIntegerVector3Slot> {
    pub fn bind(&mut self, value: &[(u32, u32, u32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 3);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform3uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, UnsignedIntegerVector4Slot> {
    pub fn bind(&mut self, value: (u32, u32, u32, u32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform4ui(Some(&location), value.0, value.1, value.2, value.3);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfUnsignedIntegerVector4Slot> {
    pub fn bind(&mut self, value: &[(u32, u32, u32, u32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 4);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform4uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, BoolSlot> {
    pub fn bind(&mut self, value: u32) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1ui(Some(&location), value);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfBoolSlot> {
    pub fn bind(&mut self, value: &[u32]) {
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, BoolVector2Slot> {
    pub fn bind(&mut self, value: (u32, u32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform2ui(Some(&location), value.0, value.1);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfBoolVector2Slot> {
    pub fn bind(&mut self, value: &[(u32, u32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 2);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform2uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, BoolVector3Slot> {
    pub fn bind(&mut self, value: (u32, u32, u32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform3ui(Some(&location), value.0, value.1, value.2);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfBoolVector3Slot> {
    pub fn bind(&mut self, value: &[(u32, u32, u32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 3);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform3uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, BoolVector4Slot> {
    pub fn bind(&mut self, value: (u32, u32, u32, u32)) {
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != value {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform4ui(Some(&location), value.0, value.1, value.2, value.3);
                });
            }

            self.slot.current_value = value;
        }
    }
}

impl<'a> Binder<'a, ArrayOfBoolVector4Slot> {
    pub fn bind(&mut self, value: &[(u32, u32, u32, u32)]) {
        let (gl, _) = unsafe { self.connection.unpack() };
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 4);

            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform4uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }
}

impl<'a> Binder<'a, FloatSampler2DSlot> {
    pub fn bind<F>(&mut self, value: &FloatSampler2DHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatSampler2DSlot> {
    pub fn bind<F>(&mut self, value: &[FloatSampler2DHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, IntegerSampler2DSlot> {
    pub fn bind<F>(&mut self, value: &IntegerSampler2DHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfIntegerSampler2DSlot> {
    pub fn bind<F>(&mut self, value: &[IntegerSampler2DHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, UnsignedIntegerSampler2DSlot> {
    pub fn bind<F>(&mut self, value: &UnsignedIntegerSampler2DHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfUnsignedIntegerSampler2DSlot> {
    pub fn bind<F>(&mut self, value: &[UnsignedIntegerSampler2DHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, FloatSampler2DArraySlot> {
    pub fn bind<F>(&mut self, value: &FloatSampler2DArrayHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatSampler2DArraySlot> {
    pub fn bind<F>(&mut self, value: &[FloatSampler2DArrayHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, IntegerSampler2DArraySlot> {
    pub fn bind<F>(&mut self, value: &IntegerSampler2DArrayHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfIntegerSampler2DArraySlot> {
    pub fn bind<F>(&mut self, value: &[IntegerSampler2DArrayHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, UnsignedIntegerSampler2DArraySlot> {
    pub fn bind<F>(&mut self, value: &UnsignedIntegerSampler2DArrayHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfUnsignedIntegerSampler2DArraySlot> {
    pub fn bind<F>(&mut self, value: &[UnsignedIntegerSampler2DArrayHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, FloatSampler3DSlot> {
    pub fn bind<F>(&mut self, value: &FloatSampler3DHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatSampler3DSlot> {
    pub fn bind<F>(&mut self, value: &[FloatSampler3DHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, IntegerSampler3DSlot> {
    pub fn bind<F>(&mut self, value: &IntegerSampler3DHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfIntegerSampler3DSlot> {
    pub fn bind<F>(&mut self, value: &[IntegerSampler3DHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, UnsignedIntegerSampler3DSlot> {
    pub fn bind<F>(&mut self, value: &UnsignedIntegerSampler3DHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfUnsignedIntegerSampler3DSlot> {
    pub fn bind<F>(&mut self, value: &[UnsignedIntegerSampler3DHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, FloatSamplerCubeSlot> {
    pub fn bind<F>(&mut self, value: &FloatSamplerCubeHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfFloatSamplerCubeSlot> {
    pub fn bind<F>(&mut self, value: &[FloatSamplerCubeHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, IntegerSamplerCubeSlot> {
    pub fn bind<F>(&mut self, value: &IntegerSamplerCubeHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfIntegerSamplerCubeSlot> {
    pub fn bind<F>(&mut self, value: &[IntegerSamplerCubeHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, UnsignedIntegerSamplerCubeSlot> {
    pub fn bind<F>(&mut self, value: &UnsignedIntegerSamplerCubeHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfUnsignedIntegerSamplerCubeSlot> {
    pub fn bind<F>(&mut self, value: &[UnsignedIntegerSamplerCubeHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, Sampler2DShadowSlot> {
    pub fn bind<F>(&mut self, value: &Sampler2DShadowHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfSampler2DShadowSlot> {
    pub fn bind<F>(&mut self, value: &[Sampler2DShadowHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, Sampler2DArrayShadowSlot> {
    pub fn bind<F>(&mut self, value: &Sampler2DArrayShadowHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfSampler2DArrayShadowSlot> {
    pub fn bind<F>(&mut self, value: &[Sampler2DArrayShadowHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, SamplerCubeShadowSlot> {
    pub fn bind<F>(&mut self, value: &SamplerCubeShadowHandle<F>) {
        let unit = value.bind(self.connection) as i32;
        let (gl, _) = unsafe { self.connection.unpack() };

        if self.slot.current_value != unit {
            unsafe {
                self.slot.location_id.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.slot.current_value = unit
        }
    }
}

impl<'a> Binder<'a, ArrayOfSamplerCubeShadowSlot> {
    pub fn bind<F>(&mut self, value: &[SamplerCubeShadowHandle<F>]) {
        let units: Vec<i32> = value
            .iter()
            .map(|s| s.bind(self.connection) as i32)
            .collect();
        let (gl, _) = unsafe { self.connection.unpack() };

        unsafe {
            self.slot.location_id.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}

impl<'a> Binder<'a, BlockSlot> {
    pub fn bind<T>(&mut self, value: &BufferView<T>)
    where
        T: Std140,
    {
        let binding = value.bind_uniform(self.connection);
        let (gl, _) = unsafe { self.connection.unpack() };
        let BlockSlot {
            program_id,
            index,
            current_binding,
        } = self.slot;

        if *current_binding != binding {
            unsafe {
                program_id.with_value_unchecked(|program| {
                    gl.uniform_block_binding(program, *index, binding);
                });
            }

            *current_binding = binding
        }
    }
}
