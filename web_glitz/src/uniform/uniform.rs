use super::{UniformValueIdentifier, UniformBlockIdentifier, UniformValueSlot, UniformBlockSlot};
use program::UniformType;
use std::borrow::Borrow;
use uniform::uniform_identifier::ValueIdentifierTail;
use std::sync::Arc;
use std::ops::Deref;
use uniform::ValueIdentifierSegment;
use program::BindingSlot;
use sampler::SamplerHandle;
use texture::Texture2DHandle;
use texture::TextureFormat;
use image_format::FloatSamplable;

pub trait Uniforms {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError>;
}

pub trait Uniform {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError>;
}

pub enum UniformBindingError {
    NotFound,
    TypeMismatch
}

impl Uniform for f32 {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError> {
        if !identifier.is_empty() {
            return Err(UniformBindingError::NotFound)
        }

        if let BindingSlot::Float(binder) = slot {
            binder.bind(*self);

            Ok(())
        } else {
            Err(UniformBindingError::TypeMismatch)
        }
    }
}

impl<'a> Uniform for &'a [f32] {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError> {
        if !identifier.tail().is_empty() || identifier.head() != Some(&ValueIdentifierSegment::ArrayIndex(0)) {
            return Err(UniformBindingError::NotFound)
        }

        if let BindingSlot::ArrayOfFloat(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(UniformBindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for SamplerHandle<Texture2DHandle<F>> where F: TextureFormat + FloatSamplable + 'static {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError> {
        if !identifier.is_empty() {
            return Err(UniformBindingError::NotFound)
        }

        if let BindingSlot::FloatSampler2D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(UniformBindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [SamplerHandle<Texture2DHandle<F>>] where F: TextureFormat + FloatSamplable + 'static {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError> {
        if !identifier.is_empty() {
            return Err(UniformBindingError::NotFound)
        }

        if let BindingSlot::ArrayOfFloatSampler2D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(UniformBindingError::TypeMismatch)
        }
    }
}

impl<T> Uniform for Vec<T> where for<'a> &'a [T]: Uniform {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError> {
        self.deref().bind_value(identifier, slot)
    }
}

impl<T> Uniform for Box<T> where T: Uniform {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError> {
        self.deref().bind_value(identifier, slot)
    }
}

impl<T> Uniform for Box<[T]> where for<'a> &'a [T]: Uniform {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError> {
        self.deref().bind_value(identifier, slot)
    }
}

impl<T> Uniform for Arc<T> where T: Uniform {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError> {
        self.deref().bind_value(identifier, slot)
    }
}

impl<T> Uniform for Arc<[T]> where for<'a> &'a [T]: Uniform {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError> {
        self.deref().bind_value(identifier, slot)
    }
}

macro_rules! generate_array_impl {
    ($size:tt) => {
        impl<T> Uniform for [T;$size] where for<'a> &'a [T]: Uniform {
            fn bind_value(&self, identifier: ValueIdentifierTail, slot: &mut BindingSlot) -> Result<(), UniformBindingError> {
                self.bind_value(identifier, slot)
            }
        }
    }
}

generate_array_impl!(0);
generate_array_impl!(1);
generate_array_impl!(2);
generate_array_impl!(3);
generate_array_impl!(4);
generate_array_impl!(5);
generate_array_impl!(6);
generate_array_impl!(7);
generate_array_impl!(8);
generate_array_impl!(9);
generate_array_impl!(10);
generate_array_impl!(11);
generate_array_impl!(12);
generate_array_impl!(13);
generate_array_impl!(14);
generate_array_impl!(15);
generate_array_impl!(16);
generate_array_impl!(17);
generate_array_impl!(18);
generate_array_impl!(19);
generate_array_impl!(20);
generate_array_impl!(21);
generate_array_impl!(22);
generate_array_impl!(23);
generate_array_impl!(24);
generate_array_impl!(25);
generate_array_impl!(26);
generate_array_impl!(27);
generate_array_impl!(28);
generate_array_impl!(29);
generate_array_impl!(30);
generate_array_impl!(31);
generate_array_impl!(32);
