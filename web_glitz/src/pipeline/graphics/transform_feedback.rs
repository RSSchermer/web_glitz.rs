use std::borrow::Borrow;

use crate::pipeline::graphics::AttributeType;

pub unsafe trait TransformFeedback {
    fn varyings() -> &'static [VaryingDescriptor];
}

pub unsafe trait TransformFeedbackLayout {
    type Layout: Borrow<[&'static [VaryingDescriptor]]> + 'static;

    fn transform_feedback_layout() -> Self::Layout;
}

#[derive(Clone, Copy)]
pub struct VaryingDescriptor {
    pub name: &'static str,
    pub attribute_type: AttributeType,
}

pub unsafe trait TransformFeedbackDescription {
    type Layout: Borrow<[&'static [VaryingDescriptor]]> + 'static;

    fn transform_feedback_layout() -> Option<Self::Layout>;
}

unsafe impl TransformFeedbackDescription for () {
    type Layout = [&'static [VaryingDescriptor]; 0];

    fn transform_feedback_layout() -> Option<Self::Layout> {
        None
    }
}

unsafe impl<T> TransformFeedbackDescription for T
where
    T: TransformFeedbackLayout,
{
    type Layout = T::Layout;

    fn transform_feedback_layout() -> Option<Self::Layout> {
        let layout = T::transform_feedback_layout();

        if layout.borrow().len() > 0 {
            Some(layout)
        } else {
            None
        }
    }
}

macro_rules! impl_separate_transform_feedback_layout {
    ($n:tt, $($T:ident),*) => {
        unsafe impl<$($T),*> TransformFeedbackLayout for ($($T),*) where $($T: TransformFeedback),* {
            type Layout = [&'static [VaryingDescriptor]; $n];

            fn transform_feedback_layout() -> Self::Layout {
                [
                    $($T::varyings()),*
                ]
            }
        }
    }
}

impl_separate_transform_feedback_layout!(2, T0, T1);
impl_separate_transform_feedback_layout!(3, T0, T1, T2);
impl_separate_transform_feedback_layout!(4, T0, T1, T2, T3);
impl_separate_transform_feedback_layout!(5, T0, T1, T2, T3, T4);
impl_separate_transform_feedback_layout!(6, T0, T1, T2, T3, T4, T5);
impl_separate_transform_feedback_layout!(7, T0, T1, T2, T3, T4, T5, T6);
impl_separate_transform_feedback_layout!(8, T0, T1, T2, T3, T4, T5, T6, T7);
impl_separate_transform_feedback_layout!(9, T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_separate_transform_feedback_layout!(10, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_separate_transform_feedback_layout!(11, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_separate_transform_feedback_layout!(12, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_separate_transform_feedback_layout!(13, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_separate_transform_feedback_layout!(
    14, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13
);
impl_separate_transform_feedback_layout!(
    15, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14
);
impl_separate_transform_feedback_layout!(
    16, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);
