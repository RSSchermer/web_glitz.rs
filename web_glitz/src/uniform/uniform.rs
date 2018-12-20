use super::{UniformValueIdentifier, UniformBlockIdentifier, UniformValueSlot, UniformBlockSlot};
use program::UniformType;
use std::borrow::Borrow;
use uniform::uniform_identifier::ValueIdentifierTail;
use std::sync::Arc;
use std::ops::Deref;
use uniform::ValueIdentifierSegment;

pub trait Uniforms {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: UniformValueSlot) -> Result<(), UniformBindingError>;

    //fn bind_block(&self, identifier: &UniformBlockIdentifier, slot: UniformBlockSlot);
}

pub trait Uniform {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: UniformValueSlot) -> Result<(), UniformBindingError>;
}

//pub trait UniformArray {
//    fn bind_slice()
//}

pub enum UniformBindingError {
    NotFound,
    TypeMismatch
}

impl Uniform for f32 {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: UniformValueSlot) -> Result<(), UniformBindingError> {
        if !identifier.is_empty() {
            return Err(UniformBindingError::NotFound)
        }

        let value_type = slot.value_type();

        if let UniformType::Float = value_type {
            slot.bind_float(*self);

            Ok(())
        } else {
            Err(UniformBindingError::TypeMismatch)
        }
    }
}

impl<'a> Uniform for &'a [f32] {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: UniformValueSlot) -> Result<(), UniformBindingError> {
        if !identifier.tail().is_empty() || identifier.head() != Some(&ValueIdentifierSegment::ArrayIndex(0)) {
            return Err(UniformBindingError::NotFound)
        }

        let value_type = slot.value_type();

        match slot.value_type() {
            UniformType::ArrayOfFloat(len) if len == self.len() => {
                slot.bind_float_array(&self);

                Ok(())
            },
            _ => Err(UniformBindingError::TypeMismatch)
        }
    }
}

impl Uniform for (f32, f32) {
    fn bind_value(&self, identifier: ValueIdentifierTail, slot: UniformValueSlot) -> Result<(), UniformBindingError> {
        if !identifier.is_empty() {
            return Err(UniformBindingError::NotFound)
        }

        if slot.value_type() == UniformType::FloatVector2 {
            slot.bind_float_vector_2(*self);

            Ok(())
        } else {
            Err(UniformBindingError::TypeMismatch)
        }
    }
}

//impl<T, E> Uniform for T where T: Borrow<[E]>, E: Uniform {
//    fn bind_value(&self, identifier: &UniformValueIdentifier, slot: UniformValueSlot) -> Result<(), UniformBindingError> {
//        if let Some(UniformValueIdentifierSegment) = identifier.get(0) {
//
//        }
//        if identifer.is_array
//        if slot.value_type() == UniformType::FloatVector2 {
//            slot.bind_float_vector_2(*self);
//
//            Ok(())
//        } else {
//            Err(UniformBindingError::TypeMismatch(identifier.clone(), UniformType::FloatVector2))
//        }
//    }
//}

//pub trait BorrowSlice<T> {
//    fn borrow_slice(&self) -> &[T];
//}
//
//impl<T> BorrowSlice<T> for Vec<T> {
//    fn borrow_slice(&self) -> &[T] {
//        self.deref()
//    }
//}
//
//impl<T> BorrowSlice<T> for Arc<[T]> {
//    fn borrow_slice(&self) -> &[T] {
//        self.deref()
//    }
//}
//
//impl<T> BorrowSlice<T> for Box<[T]> {
//    fn borrow_slice(&self) -> &[T] {
//        self.deref()
//    }
//}
//
//impl<T> Uniform for T where T: BorrowSlice<f32> {
//    fn bind_value(&self, identifier: ValueIdentifierTail, slot: UniformValueSlot) -> Result<(), UniformBindingError> {
//        if !identifier.is_empty() {
//            return Err(UniformBindingError::NotFound)
//        }
//
//        let value_type = slot.value_type();
//        let value = self.borrow_slice();
//
//        match slot.value_type() {
//            UniformType::ArrayOfFloat(len) if len == value.len() => {
//                slot.bind_float_array(value);
//
//                Ok(())
//            },
//            _ => Err(UniformBindingError::TypeMismatch)
//        }
//    }
//}
//
//impl<T> Uniform for Vec<T> where T: Uniform {
//    fn bind_value(&self, identifier: ValueIdentifierTail, slot: UniformValueSlot) -> Result<(), UniformBindingError> {
//        if let Some(UniformValueIdentifierSegment::ArrayIndex(index)) = identifier.head() {
//            let slice = self.borrow_slice();
//
//            if let Some(element) = slice.get(index) {
//                return element.bind_value(identifier.tail(), slot);
//            }
//        }
//
//        Err(UniformBindingError::NotFound(identifier.root().clone()))
//    }
//}
