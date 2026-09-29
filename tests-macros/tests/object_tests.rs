#[qi_macros::object]
trait MyObject {
    fn my_method(&self, a: i32, b: Vec<f64>) -> String;
}

#[test]
fn test_object_trait_is_declared() {
    struct Impl;
    impl MyObject for Impl {
        fn my_method(&self, a: i32, b: Vec<f64>) -> String {
            format!("{a}:{}", b.len())
        }
    }
    assert_eq!(Impl.my_method(1, vec![0.5]), "1:1");
}
