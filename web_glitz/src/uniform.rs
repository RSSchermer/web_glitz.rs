use buffer::BufferData;
use buffer::BufferHandle;
use rendering_context::RenderingContext;
use std::borrow::Borrow;
use std::fmt;
use std::fmt::Display;
use std::marker;
use rendering_context::Connection;
use program::ActiveUniform;
use util::JsId;
use program::UniformValue;

pub trait Uniform {
    fn bind_value(&self, binder: UniformBindingSlot);
}

pub struct UniformBindingSlot<'a> {
    uniform: &'a mut ActiveUniform,
    connection: *mut Connection
}

impl<'a> UniformBindingSlot<'a> {
    pub fn identifier(&self) -> &UniformIdentifier {
        &self.uniform.identifier
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

    pub fn bind_vector_2(self, value: (f32, f32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::Vector2(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform2f(Some(&location), value.0, value.1);
                });
            }

            self.uniform.current_value = Some(UniformValue::Vector2(value))
        }
    }

    pub fn bind_vector_3(self, value: (f32, f32, f32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::Vector3(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform3f(Some(&location), value.0, value.1, value.2);
                });
            }

            self.uniform.current_value = Some(UniformValue::Vector3(value))
        }
    }

    pub fn bind_vector_4(self, value: (f32, f32, f32, f32)) {
        let Connection(gl, _) = unsafe { &mut *self.connection };

        if self.uniform.current_value != Some(UniformValue::Vector4(value)) {
            unsafe {
                self.uniform.location.with_value_unchecked(|location| {
                    gl.uniform4f(Some(&location), value.0, value.1, value.2, value.3);
                });
            }

            self.uniform.current_value = Some(UniformValue::Vector4(value))
        }
    }
}

pub trait Uniforms {
    fn bind_uniforms(&self, slots: UniformBindingSlots);
}

pub struct UniformBindingSlots<'a> {
    active_uniforms: std::slice::IterMut<'a, ActiveUniform>,
    connection: *mut Connection
}

impl<'a> Iterator for UniformBindingSlots<'a> {
    type Item = UniformBindingSlot<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.active_uniforms.next().map(|active_uniform| {
            UniformBindingSlot {
                uniform: active_uniform,
                connection: self.connection
            }
        })
    }
}

#[derive(PartialEq, Hash)]
pub struct UniformIdentifier {
    segments: Vec<UniformIdentifierSegment>,
}

impl UniformIdentifier {
    pub fn from_string(string: &str) -> Self {
        UniformIdentifier {
            segments: string
                .split(".")
                .map(|s| UniformIdentifierSegment::from_string(s))
                .collect(),
        }
    }

    pub fn is_array_identifier(&self) -> bool {
        if let Some(segment) = self.segments.last() {
            segment.is_array_identifier()
        } else {
            false
        }
    }
}

impl Display for UniformIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.segments
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}

#[derive(Clone, PartialEq, Hash)]
pub enum UniformIdentifierSegment {
    Simple(String),
    ArrayElement(String, u32),
}

impl UniformIdentifierSegment {
    pub fn from_string(string: &str) -> Self {
        let parts = string.split("[").collect::<Vec<_>>();

        if parts.len() == 1 {
            UniformIdentifierSegment::Simple(parts[0].to_string())
        } else {
            let index = parts[1].trim_right_matches("]").parse::<u32>().unwrap();

            UniformIdentifierSegment::ArrayElement(parts[0].to_string(), index)
        }
    }

    pub fn is_array_identifier(&self) -> bool {
        if let UniformIdentifierSegment::ArrayElement(_, _) = self {
            true
        } else {
            false
        }
    }
}

impl Into<UniformIdentifier> for UniformIdentifierSegment {
    fn into(self) -> UniformIdentifier {
        UniformIdentifier {
            segments: vec![self],
        }
    }
}

impl Display for UniformIdentifierSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UniformIdentifierSegment::Simple(name) => write!(f, "{}", name),
            UniformIdentifierSegment::ArrayElement(array_name, index) => {
                write!(f, "{}[{}]", array_name, index)
            }
        }
    }
}
