mod primitive {
    use test_fuzz::test_fuzz;

    #[test_fuzz]
    pub fn target(
        bool: bool,
        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
        i128: i128,
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
        u128: u128,
        f32: f32,
        f64: f64,
        char: char,
    ) {
    }

    #[test]
    fn test() {
        target(
            bool::default(),
            i8::default(),
            i16::default(),
            i32::default(),
            i64::default(),
            i128::default(),
            u8::default(),
            u16::default(),
            u32::default(),
            u64::default(),
            u128::default(),
            f32::default(),
            f64::default(),
            char::default(),
        );
    }
}

mod string {
    use test_fuzz::test_fuzz;

    #[test_fuzz]
    fn target(str: &str, string: String, ref_string: &String) {}

    #[test]
    fn test() {
        target(<&str>::default(), String::default(), &String::default());
    }
}

mod byte_array {
    use test_fuzz::test_fuzz;

    #[test_fuzz]
    fn target(
        byte_array_0: [u8; 0],
        ref_byte_array_0: &[u8; 0],
        byte_array_1: [u8; 1],
        ref_byte_array_1: &[u8; 1],
        byte_array_2: [u8; 2],
        ref_byte_array_2: &[u8; 2],
    ) {
    }

    #[test]
    fn test() {
        target(
            <[u8; 0]>::default(),
            &<[u8; 0]>::default(),
            <[u8; 1]>::default(),
            &<[u8; 1]>::default(),
            <[u8; 2]>::default(),
            &<[u8; 2]>::default(),
        );
    }
}

mod option {
    use test_fuzz::test_fuzz;

    #[test_fuzz]
    fn target(option: Option<u8>, ref_option: &Option<u8>) {}

    #[test]
    fn test() {
        target(Option::<u8>::default(), &Option::<u8>::default());
    }
}

mod unit {
    use test_fuzz::test_fuzz;

    #[test_fuzz]
    fn target(unit: (), ref_unit: &()) {}

    #[test]
    fn test() {
        target(<()>::default(), &<()>::default());
    }
}

mod unit_struct {
    use serde::{Deserialize, Serialize};
    use test_fuzz::test_fuzz;

    #[derive(Clone, Default, Deserialize, Serialize)]
    struct UnitStruct;

    #[test_fuzz]
    fn target(unit_struct: UnitStruct, ref_unit_struct: &UnitStruct) {}

    #[test]
    fn test() {
        target(UnitStruct::default(), &UnitStruct::default());
    }
}

mod unit_variant {
    use serde::{Deserialize, Serialize};
    use test_fuzz::test_fuzz;

    #[derive(Clone, Deserialize, Serialize)]
    enum UnitVariant {
        A,
        B,
    }

    #[test_fuzz]
    fn target(unit_variant: UnitVariant, ref_unit_variant: &UnitVariant) {}

    #[test]
    fn test() {
        target(UnitVariant::A, &UnitVariant::B);
    }
}

mod newtype_struct {
    use serde::{Deserialize, Serialize};
    use test_fuzz::test_fuzz;

    #[derive(Clone, Default, Deserialize, Serialize)]
    struct NewtypeStruct(u8);

    #[test_fuzz]
    fn target(newtype_struct: NewtypeStruct, ref_newtype_struct: &NewtypeStruct) {}

    #[test]
    fn test() {
        target(NewtypeStruct::default(), &NewtypeStruct::default());
    }
}

mod newtype_variant {
    use serde::{Deserialize, Serialize};
    use test_fuzz::test_fuzz;

    #[derive(Clone, Deserialize, Serialize)]
    enum NewtypeVariant {
        N(u8),
    }

    impl Default for NewtypeVariant {
        fn default() -> Self {
            NewtypeVariant::N(u8::default())
        }
    }

    #[test_fuzz]
    fn target(newtype_variant: NewtypeVariant, ref_newtype_variant: &NewtypeVariant) {}

    #[test]
    fn test() {
        target(NewtypeVariant::default(), &NewtypeVariant::default());
    }
}

mod seq {
    use std::collections::HashSet;
    use test_fuzz::test_fuzz;

    #[test_fuzz]
    fn target(
        seq_slice: &[u8],
        seq_vec: Vec<u8>,
        ref_seq_vec: &Vec<u8>,
        seq_hash_set: HashSet<u8>,
        ref_seq_hash_set: &HashSet<u8>,
    ) {
    }

    #[test]
    fn test() {
        target(
            <&[u8]>::default(),
            Vec::<u8>::default(),
            &Vec::<u8>::default(),
            HashSet::<u8>::default(),
            &HashSet::<u8>::default(),
        );
    }
}

mod tuple {
    use test_fuzz::test_fuzz;

    #[test_fuzz]
    fn target(
        tuple_u8: (u8,),
        ref_tuple_u8: &(u8,),
        tuple_u8_u8: (u8, u8),
        ref_tuple_u8_u8: &(u8, u8),
    ) {
    }

    #[test]
    fn test() {
        target(
            <(u8,)>::default(),
            &<(u8,)>::default(),
            <(u8, u8)>::default(),
            &<(u8, u8)>::default(),
        );
    }
}

mod tuple_struct {
    use serde::{Deserialize, Serialize};
    use test_fuzz::test_fuzz;

    #[derive(Clone, Default, Deserialize, Serialize)]
    struct TupleStruct(u8, u8, u8);

    #[test_fuzz]
    fn target(tuple_struct: TupleStruct, ref_tuple_struct: &TupleStruct) {}

    #[test]
    fn test() {
        target(TupleStruct::default(), &TupleStruct::default());
    }
}

mod tuple_variant {
    use serde::{Deserialize, Serialize};
    use test_fuzz::test_fuzz;

    #[derive(Clone, Deserialize, Serialize)]
    enum TupleVariant {
        T(u8, u8),
    }

    impl Default for TupleVariant {
        fn default() -> Self {
            TupleVariant::T(u8::default(), u8::default())
        }
    }

    #[test_fuzz]
    fn target(tuple_variant: TupleVariant, ref_tuple_variant: &TupleVariant) {}

    #[test]
    fn test() {
        target(TupleVariant::default(), &TupleVariant::default());
    }
}

mod map {
    use std::collections::BTreeMap;
    use test_fuzz::test_fuzz;

    #[test_fuzz]
    fn target(map: BTreeMap<u8, u8>, ref_map: &BTreeMap<u8, u8>) {}

    #[test]
    fn test() {
        target(
            BTreeMap::<u8, u8>::default(),
            &BTreeMap::<u8, u8>::default(),
        );
    }
}

mod strukt {
    use serde::{Deserialize, Serialize};
    use test_fuzz::test_fuzz;

    #[derive(Clone, Default, Deserialize, Serialize)]
    struct Struct {
        r: u8,
        g: u8,
        b: u8,
    }

    #[test_fuzz]
    fn target(strukt: Struct, ref_strukt: &Struct) {}

    #[test]
    fn test() {
        target(Struct::default(), &Struct::default());
    }
}

mod struct_variant {
    use serde::{Deserialize, Serialize};
    use test_fuzz::test_fuzz;

    #[derive(Clone, Deserialize, Serialize)]
    enum StructVariant {
        S { r: u8, g: u8, b: u8 },
    }

    impl Default for StructVariant {
        fn default() -> Self {
            StructVariant::S {
                r: u8::default(),
                g: u8::default(),
                b: u8::default(),
            }
        }
    }

    #[test_fuzz]
    fn target(struct_variant: StructVariant, ref_struct_variant: &StructVariant) {}

    #[test]
    fn test() {
        target(StructVariant::default(), &StructVariant::default());
    }
}
