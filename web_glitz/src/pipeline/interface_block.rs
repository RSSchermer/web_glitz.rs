use crate::std_140::ReprStd140;

/// Trait that may be implemented on a type which, when stored in a `Buffer`, may safely be used to
/// back a device interface block (uniform block) that has a compatible memory layout.
///
/// # Unsafe
///
/// Types that implement this type must have a fixed memory layout and any instance of the type must
/// match the memory layout specified, otherwise a pipeline that uses an interface block backed by
/// the instance may produce unexpected results.
///
/// # Recommended use
///
/// This trait may be automatically derived on a struct when all of its members implement
/// [InterfaceBlockComponent]. Notably, [InterfaceBlockComponent] is implemented for all basic
/// `std140` memory units defined in [web_glitz::std140]:
///
/// ```
/// use web_glitz::pipeline::interface_block::InterfaceBlock;
/// use web_glitz::std140;
///
/// #[repr_std140]
/// #[derive(InterfaceBlock)]
/// struct MyUniforms {
///     transform: std140::mat4x4,
///     base_color: std140::vec4,
/// }
/// ```
///
/// The [InterfaceBlockComponent] trait may also be automatically derived on a struct when all of
/// its members implement [InterfaceBlockComponent], which allows you to safely define custom
/// interface block components:
///
/// ```
/// use web_glitz::pipeline::interface_block::InterfaceBlock;
/// use web_glitz::std140;
///
/// #[repr_std140]
/// #[derive(InterfaceBlockComponent)]
/// struct PointLight {
///     position: std140::vec3,
///     color: std140::vec3,
///     constant_attenuation: std140::float,
///     linear_attenuation: std140::float,
///     quadratic_attenuation: std140::float,
/// }
///
/// #[repr_std140]
/// #[derive(InterfaceBlock)]
/// struct MyUniforms {
///     transform: std140::mat4x4,
///     base_color: std140::vec4,
///     light: PointLight,
/// }
/// ```
///
/// Note that both `MyUniforms` and `PointLight` in the example above are also marked with the
/// `#[repr_std140]` attribute to ensure that the struct layout matches the std140 specification;
/// this is not a strict requirement for automatically deriving the [InterfaceBlock] trait. However,
/// the [InterfaceBlock] (and [InterfaceBlockComponent]) trait(s) do rely on the struct having a
/// stable representation across builds; the Rust compiler gives no consistency guarantees for type
/// representations between builds (that is: if a type seems compatible with a memory layout on your
/// current build, it is not guaranteed to remain compatible on any future builds), unless a
/// specific representation is specified with the `#[repr]` attribute
/// (e.g. `#[repr(C, align(16))]`). The marker trait [StableRepr] may be unsafely implemented on
/// types to mark them as having a stable representation. [StableRepr] is automatically implemented
/// for any struct marked with `#[repr_std140]`.
pub unsafe trait InterfaceBlock: StableRepr {
    /// Verifies the compatibility of this type's memory layout against the given `memory_layout`.
    ///
    /// Implementations of this trait may assume the [MemoryUnitDescriptor]s in the `memory_layout`
    /// to be ordered by memory offset.
    fn compatibility(memory_layout: &[MemoryUnitDescriptor]) -> Result<(), Incompatible>;
}

/// Trait that may be implemented on types that are to be used as struct members for a struct
/// deriving [InterfaceBlock].
///
/// If all members of a struct implement [InterfaceBlockComponent], then the [InterfaceBlock] can
/// be automatically derived (see the documentation for [InterfaceBlock] for an example).
///
/// This trait may be automatically derived for a struct, if all of the struct's members implement
/// [InterfaceBlockComponent] as well.
///
/// # Unsafe
///
/// If `check_compatibility` returns [CheckCompatibility::Finished] or
/// [CheckCompatibility::Continue], then the type must be compatible with the memory layout it was
/// checked against, otherwise pipelines that use this type as part of an interface block may
/// produce unpredictable results.
pub unsafe trait InterfaceBlockComponent: StableRepr {
    /// Checks whether this component is compatible with a certain memory layout, when used at
    /// offset `component_offset` relative to the start of the interface block in which it is used.
    ///
    /// Implementers may assume the `remainder` iterator to be advanced to the first memory unit
    /// that is expected to be provided by this type, such that the first call to [Iterator::next]
    /// yields the first [MemoryUnitDescriptor] this type is expected to match. The implementation
    /// may expect the iterator to yield [MemoryUnitDescriptor]s in order of offset
    /// ([MemoryUnitDescriptor::offset]). If the type does not contain a memory unit matching the
    /// memory unit's [UnitLayout] ([MemoryUnitDescriptor::layout]) at the memory unit's offset,
    /// then [CheckCompatibility::Incompatible] must be returned. Note that the offset of any memory
    /// units yielded by the iterator are relative to the start of the interface block, not relative
    /// to the start of this component: given `internal_offset` an offset relative to the start of
    /// the component, the offset relative to the start of the interface block may be computed as
    /// `component_offset + internal_offset`.
    ///
    /// The implementation is expected to advance the iterator once for every memory unit this type
    /// defines (*not* once for every component - you may pass the iterator to a sub-component
    /// which is then expected to advance the iterator once for every memory unit it defines, in a
    /// recursive fashion), such that the iterator may be passed on to the next component (if any)
    /// afterwards. If the iterator completes during this operation ([Iterator::next] returns
    /// [None]), then the implementation is expected to return [CheckCompatibility::Finished]
    /// (unless the component was found to be incompatible with a prior memory unit descriptor, in
    /// which case [CheckCompatibility::Incompatible] should be returned).
    fn check_compatibility<'a, I>(
        component_offset: usize,
        remainder: &'a mut I,
    ) -> CheckCompatibility
    where
        I: Iterator<Item = &'a MemoryUnitDescriptor>;
}

/// Marker trait for types that are guaranteed have a stable memory representation across builds.
///
/// This trait may be unsafely implemented on types to mark them as having a stable representation.
///
/// ```
/// use web_glitz::pipeline::interface_block::StableRepr;
///
/// #[repr(C, align(16))]
/// struct MyType {
///     member: f32
/// }
///
/// unsafe impl StableRepr for MyType {}
/// ```
///
/// This trait is required to be implemented for any type that wishes to implement [InterfaceBlock]
/// or [InterfaceBlockComponent].
///
/// This trait is automatically implemented for any type marked with [web_glitz::std140::ReprStd140],
/// including any user-defined structs marked with the `#[repr_std140]` attribute.
///
/// # Unsafe
///
/// This trait should only be implemented on types which have a representation that is guaranteed
/// to be stable across builds. By default, the Rust compiler gives no guarantees on the memory
/// layout of non-primitive types such as (user-defined) structs or tuples, which allows the
/// compiler to optimize their memory layout. However, the compiler may be instructed to use a
/// well-defined memory layout with the `#[repr]` attribute, e.g. `#[repr(C)]`.
pub unsafe trait StableRepr {}

unsafe impl<T> StableRepr for T where T: ReprStd140 {}

#[derive(Debug)]
pub enum Incompatible {
    MissingUnit(MemoryUnitDescriptor),
    UnitLayoutMismatch(MemoryUnitDescriptor, UnitLayout),
}

pub enum CheckCompatibility {
    Continue,
    Finished,
    Incompatible(Incompatible),
}

/// Describes a memory unit in an interface block at which it occurs, and its [UnitLayout].
#[derive(PartialEq, Clone, Debug)]
pub struct MemoryUnitDescriptor {
    offset: usize,
    layout: UnitLayout,
}

impl MemoryUnitDescriptor {
    pub(crate) fn new(offset: usize, layout: UnitLayout) -> Self {
        MemoryUnitDescriptor { offset, layout }
    }

    /// The offset at which this [MemoryUnitDescriptor] occurs within the interface block.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The [UnitLayout] of this [MemoryUnitDescriptor].
    ///
    /// See the documentation [UnitLayout] for details.
    pub fn layout(&self) -> &UnitLayout {
        &self.layout
    }
}

/// Enumerates the value orderings in memory for matrices.
///
/// When [ColumnMajor], values are ordered such that first the values in the first column are
/// stored from top to bottom, then the values in the second column, then the values in the third
/// column, etc.
///
/// When [RowMajor], values are ordered such that first the values in the first row are stored from
/// left to right, then the values in the second row, then the values in the third row, etc.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MatrixOrder {
    ColumnMajor,
    RowMajor,
}

/// Enumerates the kinds of memory unit layouts that can occur within an interface block.
#[derive(PartialEq, Clone, Debug)]
pub enum UnitLayout {
    Float,
    FloatArray {
        stride: u8,
        len: usize,
    },
    FloatVector2,
    FloatVector2Array {
        stride: u8,
        len: usize,
    },
    FloatVector3,
    FloatVector3Array {
        stride: u8,
        len: usize,
    },
    FloatVector4,
    FloatVector4Array {
        stride: u8,
        len: usize,
    },
    Integer,
    IntegerArray {
        stride: u8,
        len: usize,
    },
    IntegerVector2,
    IntegerVector2Array {
        stride: u8,
        len: usize,
    },
    IntegerVector3,
    IntegerVector3Array {
        stride: u8,
        len: usize,
    },
    IntegerVector4,
    IntegerVector4Array {
        stride: u8,
        len: usize,
    },
    UnsignedInteger,
    UnsignedIntegerArray {
        stride: u8,
        len: usize,
    },
    UnsignedIntegerVector2,
    UnsignedIntegerVector2Array {
        stride: u8,
        len: usize,
    },
    UnsignedIntegerVector3,
    UnsignedIntegerVector3Array {
        stride: u8,
        len: usize,
    },
    UnsignedIntegerVector4,
    UnsignedIntegerVector4Array {
        stride: u8,
        len: usize,
    },
    Bool,
    BoolArray {
        stride: u8,
        len: usize,
    },
    BoolVector2,
    BoolVector2Array {
        stride: u8,
        len: usize,
    },
    BoolVector3,
    BoolVector3Array {
        stride: u8,
        len: usize,
    },
    BoolVector4,
    BoolVector4Array {
        stride: u8,
        len: usize,
    },
    Matrix2x2 {
        order: MatrixOrder,
        matrix_stride: u8,
    },
    Matrix2x2Array {
        order: MatrixOrder,
        matrix_stride: u8,
        array_stride: u8,
        len: usize,
    },
    Matrix2x3 {
        order: MatrixOrder,
        matrix_stride: u8,
    },
    Matrix2x3Array {
        order: MatrixOrder,
        matrix_stride: u8,
        array_stride: u8,
        len: usize,
    },
    Matrix2x4 {
        order: MatrixOrder,
        matrix_stride: u8,
    },
    Matrix2x4Array {
        order: MatrixOrder,
        matrix_stride: u8,
        array_stride: u8,
        len: usize,
    },
    Matrix3x2 {
        order: MatrixOrder,
        matrix_stride: u8,
    },
    Matrix3x2Array {
        order: MatrixOrder,
        matrix_stride: u8,
        array_stride: u8,
        len: usize,
    },
    Matrix3x3 {
        order: MatrixOrder,
        matrix_stride: u8,
    },
    Matrix3x3Array {
        order: MatrixOrder,
        matrix_stride: u8,
        array_stride: u8,
        len: usize,
    },
    Matrix3x4 {
        order: MatrixOrder,
        matrix_stride: u8,
    },
    Matrix3x4Array {
        order: MatrixOrder,
        matrix_stride: u8,
        array_stride: u8,
        len: usize,
    },
    Matrix4x2 {
        order: MatrixOrder,
        matrix_stride: u8,
    },
    Matrix4x2Array {
        order: MatrixOrder,
        matrix_stride: u8,
        array_stride: u8,
        len: usize,
    },
    Matrix4x3 {
        order: MatrixOrder,
        matrix_stride: u8,
    },
    Matrix4x3Array {
        order: MatrixOrder,
        matrix_stride: u8,
        array_stride: u8,
        len: usize,
    },
    Matrix4x4 {
        order: MatrixOrder,
        matrix_stride: u8,
    },
    Matrix4x4Array {
        order: MatrixOrder,
        matrix_stride: u8,
        array_stride: u8,
        len: usize,
    },
}
