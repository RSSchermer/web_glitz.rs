use std::slice;

use crate::program::{UniformValue, UniformType, UniformInfo};
use crate::rendering_context::Connection;
use crate::sampler::{Sampler, AsSampled};
use crate::util::{identical, slice_make_mut};

pub struct UniformValueSlot<'a> {
    uniform: &'a mut UniformInfo,
    connection: *mut Connection,
}

impl<'a> UniformValueSlot<'a> {
    pub fn value_type(&self) -> UniformType {
        self.uniform.value_type
    }

    pub fn bind_float(self, value: f32) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::Float(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform1f(Some(&location), value);
                });
            }

            self.uniform.current_value = Some(UniformValue::Float(value))
        }
    }

    pub fn bind_float_array(self, value: &[f32]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        unsafe {
            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform1fv_with_f32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_vector_2(self, value: (f32, f32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatVector2(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform2f(Some(&location), value.0, value.1);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatVector2(value))
        }
    }

    pub fn bind_float_vector_2_array(self, value: &[(f32, f32)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 2);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform2fv_with_f32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_vector_3(self, value: (f32, f32, f32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatVector3(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform3f(Some(&location), value.0, value.1, value.2);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatVector3(value))
        }
    }

    pub fn bind_float_vector_3_array(self, value: &[(f32, f32, f32)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 3);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform3fv_with_f32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_vector_4(self, value: (f32, f32, f32, f32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatVector4(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform4f(Some(&location), value.0, value.1, value.2, value.3);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatVector4(value))
        }
    }

    pub fn bind_float_vector_4_array(self, value: &[(f32, f32, f32, f32)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 4);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform4fv_with_f32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_matrix_2x2(self, mut value: [f32; 4]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatMatrix2x2(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform_matrix2fv_with_f32_array(Some(&location), false, &mut value);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatMatrix2x2(value))
        }
    }

    pub fn bind_float_matrix_2x2_array(self, value: &[[f32; 4]]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 4);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform_matrix2fv_with_f32_array(Some(&location), false, slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_matrix_2x3(self, mut value: [f32; 6]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatMatrix2x3(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform_matrix2x3fv_with_f32_array(Some(&location), false, &mut value);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatMatrix2x3(value))
        }
    }

    pub fn bind_float_matrix_2x3_array(self, value: &[[f32; 6]]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 6);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform_matrix2x3fv_with_f32_array(Some(&location), false, slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_matrix_2x4(self, mut value: [f32; 8]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatMatrix2x4(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform_matrix2x4fv_with_f32_array(Some(&location), false, &mut value);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatMatrix2x4(value))
        }
    }

    pub fn bind_float_matrix_2x4_array(self, value: &[[f32; 8]]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 8);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform_matrix2x4fv_with_f32_array(Some(&location), false, slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_matrix_3x2(self, mut value: [f32; 6]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatMatrix3x2(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform_matrix3x2fv_with_f32_array(Some(&location), false, &mut value);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatMatrix3x2(value))
        }
    }

    pub fn bind_float_matrix_3x2_array(self, value: &[[f32; 6]]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 6);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform_matrix3x2fv_with_f32_array(Some(&location), false, slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_matrix_3x3(self, mut value: [f32; 9]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatMatrix3x3(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform_matrix3fv_with_f32_array(Some(&location), false, &mut value);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatMatrix3x3(value))
        }
    }

    pub fn bind_float_matrix_3x3_array(self, value: &[[f32; 9]]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 9);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform_matrix3fv_with_f32_array(Some(&location), false, slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_matrix_3x4(self, mut value: [f32; 12]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatMatrix3x4(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform_matrix3x4fv_with_f32_array(Some(&location), false, &mut value);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatMatrix3x4(value))
        }
    }

    pub fn bind_float_matrix_3x4_array(self, value: &[[f32; 12]]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 12);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform_matrix3x4fv_with_f32_array(Some(&location), false, slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_matrix_4x2(self, mut value: [f32; 8]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatMatrix4x2(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform_matrix4x2fv_with_f32_array(Some(&location), false, &mut value);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatMatrix4x2(value))
        }
    }

    pub fn bind_float_matrix_4x2_array(self, value: &[[f32; 8]]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 8);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform_matrix4x2fv_with_f32_array(Some(&location), false, slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_matrix_4x3(self, mut value: [f32; 12]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatMatrix4x3(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform_matrix4x3fv_with_f32_array(Some(&location), false, &mut value);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatMatrix4x3(value))
        }
    }

    pub fn bind_float_matrix_4x3_array(self, value: &[[f32; 12]]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 12);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform_matrix4x3fv_with_f32_array(Some(&location), false, slice_make_mut(value));
            });
        }
    }

    pub fn bind_float_matrix_4x4(self, mut value: [f32; 16]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::FloatMatrix4x4(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform_matrix4fv_with_f32_array(Some(&location), false, &mut value);
                });
            }

            self.uniform.current_value = Some(UniformValue::FloatMatrix4x4(value))
        }
    }

    pub fn bind_float_matrix_4x4_array(self, value: &[[f32; 16]]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const f32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 16);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform_matrix4fv_with_f32_array(Some(&location), false, slice_make_mut(value));
            });
        }
    }

    pub fn bind_integer(self, value: i32) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::Integer(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), value);
                });
            }

            self.uniform.current_value = Some(UniformValue::Integer(value))
        }
    }

    pub fn bind_integer_array(self, value: &[i32]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        unsafe {
            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_integer_vector_2(self, value: (i32, i32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::IntegerVector2(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform2i(Some(&location), value.0, value.1);
                });
            }

            self.uniform.current_value = Some(UniformValue::IntegerVector2(value))
        }
    }

    pub fn bind_integer_vector_2_array(self, value: &[(i32, i32)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const i32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 2);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform2iv_with_i32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_integer_vector_3(self, value: (i32, i32, i32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::IntegerVector3(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform3i(Some(&location), value.0, value.1, value.2);
                });
            }

            self.uniform.current_value = Some(UniformValue::IntegerVector3(value))
        }
    }

    pub fn bind_integer_vector_3_array(self, value: &[(i32, i32, i32)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const i32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 3);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform3iv_with_i32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_integer_vector_4(self, value: (i32, i32, i32, i32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::IntegerVector4(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform4i(Some(&location), value.0, value.1, value.2, value.3);
                });
            }

            self.uniform.current_value = Some(UniformValue::IntegerVector4(value))
        }
    }

    pub fn bind_integer_vector_4_array(self, value: &[(i32, i32, i32, i32)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const i32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 4);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform4iv_with_i32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_unsigned_integer(self, value: u32) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::UnsignedInteger(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform1ui(Some(&location), value);
                });
            }

            self.uniform.current_value = Some(UniformValue::UnsignedInteger(value))
        }
    }

    pub fn bind_unsigned_integer_array(self, value: &[u32]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        unsafe {
            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform1uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_unsigned_integer_vector_2(self, value: (u32, u32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::UnsignedIntegerVector2(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform2ui(Some(&location), value.0, value.1);
                });
            }

            self.uniform.current_value = Some(UniformValue::UnsignedIntegerVector2(value))
        }
    }

    pub fn bind_unsigned_integer_vector_2_array(self, value: &[(u32, u32)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 2);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform2uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_unsigned_integer_vector_3(self, value: (u32, u32, u32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::UnsignedIntegerVector3(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform3ui(Some(&location), value.0, value.1, value.2);
                });
            }

            self.uniform.current_value = Some(UniformValue::UnsignedIntegerVector3(value))
        }
    }

    pub fn bind_unsigned_integer_vector_3_array(self, value: &[(u32, u32, u32)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 3);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform3uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_unsigned_integer_vector_4(self, value: (u32, u32, u32, u32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::UnsignedIntegerVector4(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform4ui(Some(&location), value.0, value.1, value.2, value.3);
                });
            }

            self.uniform.current_value = Some(UniformValue::UnsignedIntegerVector4(value))
        }
    }

    pub fn bind_unsigned_integer_vector_4_array(self, value: &[(u32, u32, u32, u32)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 4);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform4uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_bool(self, value: bool) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let value = value.into();

        if self.uniform.current_value != Some(UniformValue::UnsignedInteger(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform1ui(Some(&location), value);
                });
            }

            self.uniform.current_value = Some(UniformValue::UnsignedInteger(value))
        }
    }

    pub fn bind_bool_array(self, value: &[bool]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let value: Vec<u32> = value.iter().map(|v| (*v).into()).collect();

        unsafe {
            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform1uiv_with_u32_array(Some(&location), slice_make_mut(&value));
            });
        }
    }

    pub fn bind_bool_vector_2(self, value: (bool, bool)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let value = (value.0.into(), value.1.into());

        if self.uniform.current_value != Some(UniformValue::UnsignedIntegerVector2(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform2ui(Some(&location), value.0, value.1);
                });
            }

            self.uniform.current_value = Some(UniformValue::UnsignedIntegerVector2(value))
        }
    }

    pub fn bind_bool_vector_2_array(self, value: &[(bool, bool)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let value: Vec<(u32, u32)> = value.iter().map(|v| (v.0.into(), v.1.into())).collect();
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 2);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform2uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_bool_vector_3(self, value: (bool, bool, bool)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let value = (value.0.into(), value.1.into(), value.2.into());

        if self.uniform.current_value != Some(UniformValue::UnsignedIntegerVector3(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform3ui(Some(&location), value.0, value.1, value.2);
                });
            }

            self.uniform.current_value = Some(UniformValue::UnsignedIntegerVector3(value))
        }
    }

    pub fn bind_bool_vector_3_array(self, value: &[(bool, bool, bool)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let value: Vec<(u32, u32, u32)> = value.iter().map(|v| (v.0.into(), v.1.into(), v.2.into())).collect();
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 3);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform3uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_bool_vector_4(self, value: (bool, bool, bool, bool)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let value = (value.0.into(), value.1.into(), value.2.into(), value.3.into());

        if self.uniform.current_value != Some(UniformValue::UnsignedIntegerVector4(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform4ui(Some(&location), value.0, value.1, value.2, value.3);
                });
            }

            self.uniform.current_value = Some(UniformValue::UnsignedIntegerVector4(value))
        }
    }

    pub fn bind_bool_vector_4_array(self, value: &[(bool, bool, bool, bool)]) {
        let Connection(gl, _) = unsafe { &mut *self.connection };
        let value: Vec<(u32, u32, u32, u32)> = value.iter().map(|v| (v.0.into(), v.1.into(), v.2.into(), v.3.into())).collect();
        let ptr = value.as_ptr() as *const u32;
        let len = value.len();

        unsafe {
            let value = slice::from_raw_parts(ptr, len * 4);

            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform4uiv_with_u32_array(Some(&location), slice_make_mut(value));
            });
        }
    }

    pub fn bind_sampler<T>(self, sampler: &Sampler<T>)
        where
            T: AsSampled,
    {
        let connection = unsafe { &mut *self.connection };
        let unit = sampler.bind(connection) as i32;
        let Connection(gl, _) = connection;

        if self.uniform.current_value != Some(UniformValue::Integer(unit)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform1i(Some(&location), unit);
                });
            }

            self.uniform.current_value = Some(UniformValue::Integer(unit))
        }
    }

    pub fn bind_sampler_array<T>(self, samplers: &[Sampler<T>])
        where
            T: AsSampled,
    {
        let connection = unsafe { &mut *self.connection };
        let units: Vec<i32> = samplers.iter().map(|s| s.bind(connection) as i32).collect();
        let Connection(gl, _) = connection;

        unsafe {
            self.uniform.location.with_value_unchecked(|location| {
                gl.uniform1iv_with_i32_array(Some(&location), slice_make_mut(&units));
            });
        }
    }
}
