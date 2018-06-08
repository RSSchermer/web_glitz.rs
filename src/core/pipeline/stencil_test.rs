use super::test_function::TestFunction;

enum StencilOperation {
    Keep,
    Zero,
    Replace,
    Increment,
    WrappingIncrement,
    Decrement,
    WrappingDecrement,
    Invert
}

pub struct StencilTest {
    pub test_function_front: TestFunction,
    pub pass_operation_front: StencilOperation,
    pub fail_operation_front: StencilOperation,
    pub pass_depth_fail_operation_front: StencilOperation,
    pub test_function_back: TestFunction,
    pub pass_operation_back: StencilOperation,
    pub fail_operation_back: StencilOperation,
    pub pass_depth_fail_operation_back: StencilOperation,
    pub reference_value: u32,
    pub test_mask: u32,
    pub write_mask: u32
}

impl Default for StencilTest {
    fn default() -> StencilTest {
        StencilTest {
            test_function_front: TestFunction::AlwaysPass,
            pass_operation_front: StencilOperation::Keep,
            fail_operation_front: StencilOperation::Keep,
            pass_depth_fail_operation_front: StencilOperation::Keep,
            test_function_back: TestFunction::AlwaysPass,
            pass_operation_back: StencilOperation::Keep,
            fail_operation_back: StencilOperation::Keep,
            pass_depth_fail_operation_back: StencilOperation::Keep,
            reference_value: 0,
            test_mask: u32::max(),
            write_mask: u32::max()
        }
    }
}
