use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
struct Swap<T, U>(T, U);

#[test_fuzz::test_fuzz_impl]
impl<T, U> Swap<T, U>
where
    T: Clone + Serialize,
    U: Clone + Serialize,
{
    #[test_fuzz::test_fuzz(impl_generic_args = "(), bool")]
    fn swap(self) -> Swap<U, T> {
        Swap(self.1, self.0)
    }
}

#[test]
fn test() {
    let _ = Swap((), false).swap();
}
