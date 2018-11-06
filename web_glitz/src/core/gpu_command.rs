use std::marker::PhantomData;





















#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext;

    #[derive(Debug)]
    struct ReturnOk<T>(T);

    impl<T> GpuCommand<TestContext> for ReturnOk<T> {
        type Output = T;

        type Error = ();

        fn execute_static(self, _execution_context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Ok(self.0)
        }

        fn execute_dynamic(self: Box<Self>, _execution_context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Ok(self.0)
        }
    }

    #[derive(Debug)]
    struct ReturnErr<T>(T);

    impl<T> GpuCommand<TestContext> for ReturnErr<T> {
        type Output = ();

        type Error = T;

        fn execute_static(self, _execution_context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Err(self.0)
        }

        fn execute_dynamic(self: Box<Self>, _execution_context: &mut TestContext) -> Result<Self::Output, Self::Error> {
            Err(self.0)
        }
    }

    #[test]
    fn test_map_execute_static() {
        let mut context = TestContext;
        let map = Map::new(ReturnOk(1), |v| v * 2);

        assert_eq!(map.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_map_execute_dynamic() {
        let mut context = TestContext;
        let map = Map::new(ReturnOk(1), |v| v * 2);
        let boxed_map = Box::new(map);

        assert_eq!(boxed_map.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_map_err_execute_static() {
        let mut context = TestContext;
        let map_err = MapErr::new(ReturnErr(1), |v| v * 2);

        assert_eq!(map_err.execute(&mut context), Err(2));
    }

    #[test]
    fn test_map_err_execute_dynamic() {
        let mut context = TestContext;
        let map = MapErr::new(ReturnErr(1), |v| v * 2);
        let boxed_map_err = Box::new(map);

        assert_eq!(boxed_map_err.execute(&mut context), Err(2));
    }

    #[test]
    fn test_ok_then_execute_static() {
        let mut context = TestContext;
        let then = Then::new(ReturnOk(1), |res| ReturnOk(res.unwrap() * 2));

        assert_eq!(then.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_ok_then_execute_dynamic() {
        let mut context = TestContext;
        let then = Then::new(ReturnOk(1), |res| ReturnOk(res.unwrap() * 2));
        let boxed_then = Box::new(then);

        assert_eq!(boxed_then.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_err_then_execute_static() {
        let mut context = TestContext;
        let then = Then::new(ReturnErr(1), |res| ReturnErr(res.unwrap_err() * 2));

        assert_eq!(then.execute(&mut context), Err(2));
    }

    #[test]
    fn test_err_then_execute_dynamic() {
        let mut context = TestContext;
        let then = Then::new(ReturnErr(1), |res| ReturnErr(res.unwrap_err() * 2));
        let boxed_then = Box::new(then);

        assert_eq!(boxed_then.execute(&mut context), Err(2));
    }

    #[test]
    fn test_ok_and_then_execute_static() {
        let mut context = TestContext;
        let and_then = AndThen::new(ReturnOk(1), |v| ReturnOk(v * 2));

        assert_eq!(and_then.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_ok_and_then_execute_dynamic() {
        let mut context = TestContext;
        let and_then = AndThen::new(ReturnOk(1), |v| ReturnOk(v * 2));
        let boxed_and_then = Box::new(and_then);

        assert_eq!(boxed_and_then.execute(&mut context), Ok(2));
    }

    #[test]
    fn test_err_and_then_execute_static() {
        let mut context = TestContext;
        let and_then = AndThen::new(ReturnErr(1), |_| ReturnErr(2));

        assert_eq!(and_then.execute(&mut context), Err(1));
    }

    #[test]
    fn test_err_and_then_execute_dynamic() {
        let mut context = TestContext;
        let and_then = AndThen::new(ReturnErr(1), |_| ReturnErr(2));
        let boxed_and_then = Box::new(and_then);

        assert_eq!(boxed_and_then.execute(&mut context), Err(1));
    }

    #[test]
    fn test_ok_or_else_execute_static() {
        let mut context = TestContext;
        let or_else = OrElse::new(ReturnOk(1), |_| ReturnOk(2));

        assert_eq!(or_else.execute(&mut context), Ok(1));
    }

    #[test]
    fn test_ok_or_else_execute_dynamic() {
        let mut context = TestContext;
        let or_else = OrElse::new(ReturnOk(1), |_| ReturnOk(2));
        let boxed_or_else = Box::new(or_else);

        assert_eq!(boxed_or_else.execute(&mut context), Ok(1));
    }

    #[test]
    fn test_err_or_else_execute_static() {
        let mut context = TestContext;
        let or_else = OrElse::new(ReturnErr(1), |v| ReturnErr(v * 2));

        assert_eq!(or_else.execute(&mut context), Err(2));
    }

    #[test]
    fn test_err_or_else_execute_dynamic() {
        let mut context = TestContext;
        let or_else = OrElse::new(ReturnErr(1), |v| ReturnErr(v * 2));
        let boxed_or_else = Box::new(or_else);

        assert_eq!(boxed_or_else.execute(&mut context), Err(2));
    }
}