/// Trait that may be implemented on a type which, when stored in a `Buffer`, may safely be used to
/// back a device interface block (uniform block) that has a compatible memory layout.
///
/// # Unsafe
///
/// Types that implement this type must have a fixed memory layout and any instance of the type must
/// match the memory layout specified, otherwise a pipeline that uses an interface block backed by
/// the instance may produce unexpected results.
///
/// # Safely Deriving
///
/// This trait may be automatically derived on a struct when all of its members implement
/// [InterfaceBlockComponent]. Notably, [InterfaceBlockComponent] is implemented for all basic
/// `std140` memory units defined in the [std140] crate:
///
/// ```
/// # #![feature(const_fn, const_loop, const_if_match, const_ptr_offset_from, const_transmute, ptr_offset_from)]
/// use web_glitz::pipeline::interface_block::InterfaceBlock;
///
/// #[std140::repr_std140]
/// #[derive(web_glitz::derive::InterfaceBlock)]
/// struct MyUniforms {
///     transform: std140::mat4x4,
///     base_color: std140::vec4,
/// }
/// ```
///
/// The [InterfaceBlockComponent] trait is also implemented for any type that implements
/// [InterfaceBlock]:
///
/// ```
/// # #![feature(const_fn, const_loop, const_if_match, const_ptr_offset_from, const_transmute, ptr_offset_from)]
/// use web_glitz::pipeline::interface_block::InterfaceBlock;
///
/// #[std140::repr_std140]
/// #[derive(web_glitz::derive::InterfaceBlock)]
/// struct PointLight {
///     position: std140::vec3,
///     color: std140::vec3,
///     constant_attenuation: std140::float,
///     linear_attenuation: std140::float,
///     quadratic_attenuation: std140::float,
/// }
///
/// #[std140::repr_std140]
/// #[derive(web_glitz::derive::InterfaceBlock)]
/// struct MyUniforms {
///     transform: std140::mat4x4,
///     base_color: std140::vec4,
///     light: PointLight,
/// }
/// ```
///
/// Note that both `MyUniforms` and `PointLight` in the example above are also marked with the
/// `#[std140::repr_std140]` attribute to ensure that the struct layout matches the std140
/// convention; this is not a strict requirement for automatically deriving the [InterfaceBlock]
/// trait. However, the [InterfaceBlock] (and [InterfaceBlockComponent]) trait(s) do rely on the
/// struct having a stable representation across builds; the Rust compiler gives no consistency
/// guarantees for non-primitive type representations between builds (that is: if a type seems
/// compatible with a memory layout on your current build, it is not guaranteed to remain compatible
/// on any future builds), unless a specific representation is specified with the `#[repr]`
/// attribute (e.g. `#[repr(C, align(16))]`). The marker trait [StableRepr] may be unsafely
/// implemented on types to mark them as having a stable representation. [StableRepr] is already
/// implemented for all [std140::ReprStd140] types, including any struct marked with
/// `#[std140::repr_std140]`.
pub unsafe trait InterfaceBlock: StableRepr {
    const MEMORY_UNITS: &'static [MemoryUnit];
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
/// Any instance of a type that implements this trait must be bitwise compatible with the memory
/// layout specified by [MEMORY_UNITS].
pub unsafe trait InterfaceBlockComponent: StableRepr {
    const MEMORY_UNITS: &'static [MemoryUnit];
}

unsafe impl<T> InterfaceBlockComponent for T
where
    T: InterfaceBlock,
{
    const MEMORY_UNITS: &'static [MemoryUnit] = T::MEMORY_UNITS;
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
/// This trait is already implemented for any type marked with [std140::ReprStd140], including any
/// user-defined structs marked with the `#[std140::repr_std140]` attribute.
///
/// # Unsafe
///
/// This trait should only be implemented on types which have a representation that is guaranteed
/// to be stable across builds. By default, the Rust compiler gives no guarantees on the memory
/// layout of non-primitive types such as (user-defined) structs or tuples, which allows the
/// compiler to optimize their memory layout. However, the compiler may be instructed to use a
/// well-defined memory layout with the `#[repr]` attribute, e.g. `#[repr(C)]`.
pub unsafe trait StableRepr {}

/// Describes a memory unit in an interface block at which it occurs, and its [UnitLayout].
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct MemoryUnit {
    /// The offset at which this [MemoryUnitDescriptor] occurs within the interface block.
    pub offset: usize,

    /// The [UnitLayout] of this [MemoryUnitDescriptor].
    ///
    /// See the documentation [UnitLayout] for details.
    pub layout: UnitLayout,
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

/// Enumerates the kinds of memory unit layouts for memory units that can occur within an interface
/// block.
#[derive(PartialEq, Clone, Copy, Debug)]
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

unsafe impl<T> StableRepr for T where T: std140::ReprStd140 {}

macro_rules! impl_interface_block_component_std140 {
    ($T:ident, $layout:expr) => {
        unsafe impl InterfaceBlockComponent for std140::$T {
            const MEMORY_UNITS: &'static [MemoryUnit] = &[MemoryUnit {
                offset: 0,
                layout: $layout,
            }];
        }
    };
}

impl_interface_block_component_std140!(float, UnitLayout::Float);
impl_interface_block_component_std140!(vec2, UnitLayout::FloatVector2);
impl_interface_block_component_std140!(vec3, UnitLayout::FloatVector3);
impl_interface_block_component_std140!(vec4, UnitLayout::FloatVector4);
impl_interface_block_component_std140!(int, UnitLayout::Integer);
impl_interface_block_component_std140!(ivec2, UnitLayout::IntegerVector2);
impl_interface_block_component_std140!(ivec3, UnitLayout::IntegerVector3);
impl_interface_block_component_std140!(ivec4, UnitLayout::IntegerVector4);
impl_interface_block_component_std140!(uint, UnitLayout::UnsignedInteger);
impl_interface_block_component_std140!(uvec2, UnitLayout::UnsignedIntegerVector2);
impl_interface_block_component_std140!(uvec3, UnitLayout::UnsignedIntegerVector3);
impl_interface_block_component_std140!(uvec4, UnitLayout::UnsignedIntegerVector4);
impl_interface_block_component_std140!(boolean, UnitLayout::Bool);
impl_interface_block_component_std140!(bvec2, UnitLayout::BoolVector2);
impl_interface_block_component_std140!(bvec3, UnitLayout::BoolVector3);
impl_interface_block_component_std140!(bvec4, UnitLayout::BoolVector4);
impl_interface_block_component_std140!(
    mat2x2,
    UnitLayout::Matrix2x2 {
        order: MatrixOrder::ColumnMajor,
        matrix_stride: 16
    }
);
impl_interface_block_component_std140!(
    mat2x3,
    UnitLayout::Matrix2x3 {
        order: MatrixOrder::ColumnMajor,
        matrix_stride: 16
    }
);
impl_interface_block_component_std140!(
    mat2x4,
    UnitLayout::Matrix2x4 {
        order: MatrixOrder::ColumnMajor,
        matrix_stride: 16
    }
);
impl_interface_block_component_std140!(
    mat3x2,
    UnitLayout::Matrix3x2 {
        order: MatrixOrder::ColumnMajor,
        matrix_stride: 16
    }
);
impl_interface_block_component_std140!(
    mat3x3,
    UnitLayout::Matrix3x3 {
        order: MatrixOrder::ColumnMajor,
        matrix_stride: 16
    }
);
impl_interface_block_component_std140!(
    mat3x4,
    UnitLayout::Matrix3x4 {
        order: MatrixOrder::ColumnMajor,
        matrix_stride: 16
    }
);
impl_interface_block_component_std140!(
    mat4x2,
    UnitLayout::Matrix4x2 {
        order: MatrixOrder::ColumnMajor,
        matrix_stride: 16
    }
);
impl_interface_block_component_std140!(
    mat4x3,
    UnitLayout::Matrix4x3 {
        order: MatrixOrder::ColumnMajor,
        matrix_stride: 16
    }
);
impl_interface_block_component_std140!(
    mat4x4,
    UnitLayout::Matrix4x4 {
        order: MatrixOrder::ColumnMajor,
        matrix_stride: 16
    }
);

macro_rules! impl_interface_block_component_std140_array {
    ($T:ident, $layout_ident:ident) => {
        unsafe impl<const LEN: usize> InterfaceBlockComponent
            for std140::array<std140::$T, { LEN }>
        {
            const MEMORY_UNITS: &'static [MemoryUnit] = &[MemoryUnit {
                offset: 0,
                layout: UnitLayout::$layout_ident {
                    stride: 16,
                    len: LEN,
                },
            }];
        }
    };
}

impl_interface_block_component_std140_array!(float, FloatArray);
impl_interface_block_component_std140_array!(vec2, FloatVector2Array);
impl_interface_block_component_std140_array!(vec3, FloatVector3Array);
impl_interface_block_component_std140_array!(vec4, FloatVector4Array);
impl_interface_block_component_std140_array!(int, IntegerArray);
impl_interface_block_component_std140_array!(ivec2, IntegerVector2Array);
impl_interface_block_component_std140_array!(ivec3, IntegerVector3Array);
impl_interface_block_component_std140_array!(ivec4, IntegerVector4Array);
impl_interface_block_component_std140_array!(uint, UnsignedIntegerArray);
impl_interface_block_component_std140_array!(uvec2, UnsignedIntegerVector2Array);
impl_interface_block_component_std140_array!(uvec3, UnsignedIntegerVector3Array);
impl_interface_block_component_std140_array!(uvec4, UnsignedIntegerVector4Array);
impl_interface_block_component_std140_array!(boolean, BoolArray);
impl_interface_block_component_std140_array!(bvec2, BoolVector2Array);
impl_interface_block_component_std140_array!(bvec3, BoolVector3Array);
impl_interface_block_component_std140_array!(bvec4, BoolVector4Array);

macro_rules! impl_interface_block_component_std140_matrix_array {
    ($T:ident, $layout_ident:ident) => {
        unsafe impl<const LEN: usize> InterfaceBlockComponent
            for std140::array<std140::$T, { LEN }>
        {
            const MEMORY_UNITS: &'static [MemoryUnit] = &[MemoryUnit {
                offset: 0,
                layout: UnitLayout::$layout_ident {
                    order: MatrixOrder::ColumnMajor,
                    array_stride: 16,
                    matrix_stride: 16,
                    len: LEN,
                },
            }];
        }
    };
}

impl_interface_block_component_std140_matrix_array!(mat2x2, Matrix2x2Array);
impl_interface_block_component_std140_matrix_array!(mat2x3, Matrix2x3Array);
impl_interface_block_component_std140_matrix_array!(mat2x4, Matrix2x4Array);
impl_interface_block_component_std140_matrix_array!(mat3x2, Matrix3x2Array);
impl_interface_block_component_std140_matrix_array!(mat3x3, Matrix3x3Array);
impl_interface_block_component_std140_matrix_array!(mat3x4, Matrix3x4Array);
impl_interface_block_component_std140_matrix_array!(mat4x2, Matrix4x2Array);
impl_interface_block_component_std140_matrix_array!(mat4x3, Matrix4x3Array);
impl_interface_block_component_std140_matrix_array!(mat4x4, Matrix4x4Array);
