mod debug_crash {
    use serde::{Deserialize, Serialize};
    use std::fmt::{Debug, Formatter, Result};

    #[derive(Clone, Default, Deserialize, Serialize)]
    struct Struct;

    impl Debug for Struct {
        fn fmt(&self, _f: &mut Formatter<'_>) -> Result {
            panic!("bug");
        }
    }

    #[test_fuzz::test_fuzz]
    fn target(s: &Struct) {}
}

mod debug_hang {
    use serde::{Deserialize, Serialize};
    use std::fmt::{Debug, Formatter, Result};

    #[derive(Clone, Default, Deserialize, Serialize)]
    struct Struct;

    impl Debug for Struct {
        fn fmt(&self, _f: &mut Formatter<'_>) -> Result {
            #[allow(clippy::empty_loop)]
            loop {}
        }
    }

    #[test_fuzz::test_fuzz]
    fn target(s: &Struct) {}
}
