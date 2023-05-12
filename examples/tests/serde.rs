#![cfg_attr(dylint_lib = "crate_wide_allow", allow(crate_wide_allow))]
#![allow(clippy::default_constructed_unit_structs)]
#![allow(clippy::disallowed_names)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unit_arg)]

fn consume<T>(_: T) {}

mod primitive {
    // smoelius: `no_auto_generate` because `serde_cbor` does not fully support 128-bit integers:
    // https://github.com/pyfisch/cbor/pull/145
    // We might use `ciborium` as an alternative to `serde_cbor`. But `ciborium` currently has no
    // way to limit the size of an allocation: https://github.com/enarx/ciborium/issues/11
    #[test_fuzz::test_fuzz(no_auto_generate)]
    fn target(
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
        super::consume(bool);
        super::consume(i8);
        super::consume(i16);
        super::consume(i32);
        super::consume(u64);
        super::consume(i128);
        super::consume(u8);
        super::consume(u16);
        super::consume(u32);
        super::consume(u64);
        super::consume(u128);
        super::consume(f32);
        super::consume(f64);
        super::consume(char);
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
    #[test_fuzz::test_fuzz]
    fn target(str: &str, string: String, ref_string: &String, ref_mut_string: &mut String) {
        super::consume(str);
        super::consume(string);
        super::consume(ref_string);
        super::consume(ref_mut_string);
    }

    #[test]
    fn test() {
        target(
            <&str>::default(),
            String::default(),
            &String::default(),
            &mut String::default(),
        );
    }
}

mod byte_array {
    #[test_fuzz::test_fuzz]
    fn target(
        byte_array_0: [u8; 0],
        ref_byte_array_0: &[u8; 0],
        ref_mut_byte_array_0: &mut [u8; 0],
        byte_array_1: [u8; 1],
        ref_byte_array_1: &[u8; 1],
        ref_mut_byte_array_1: &mut [u8; 1],
        byte_array_2: [u8; 2],
        ref_byte_array_2: &[u8; 2],
        ref_mut_byte_array_2: &mut [u8; 2],
    ) {
        super::consume(byte_array_0);
        super::consume(ref_byte_array_0);
        super::consume(ref_mut_byte_array_0);
        super::consume(byte_array_1);
        super::consume(ref_byte_array_1);
        super::consume(ref_mut_byte_array_1);
        super::consume(byte_array_2);
        super::consume(ref_byte_array_2);
        super::consume(ref_mut_byte_array_2);
    }

    #[test]
    fn test() {
        target(
            <[u8; 0]>::default(),
            &<[u8; 0]>::default(),
            &mut <[u8; 0]>::default(),
            <[u8; 1]>::default(),
            &<[u8; 1]>::default(),
            &mut <[u8; 1]>::default(),
            <[u8; 2]>::default(),
            &<[u8; 2]>::default(),
            &mut <[u8; 2]>::default(),
        );
    }
}

mod option {
    #[test_fuzz::test_fuzz]
    fn target(option: Option<u8>, ref_option: &Option<u8>, ref_mut_option: &mut Option<u8>) {
        super::consume(option);
        super::consume(ref_option);
        super::consume(ref_mut_option);
    }

    #[test]
    fn test() {
        target(
            Option::<u8>::default(),
            &Option::<u8>::default(),
            &mut Option::<u8>::default(),
        );
    }
}

mod unit {
    #[test_fuzz::test_fuzz]
    fn target(unit: (), ref_unit: &(), ref_mut_unit: &mut ()) {
        super::consume(unit);
        super::consume(ref_unit);
        super::consume(ref_mut_unit);
    }

    #[test]
    fn test() {
        target(<()>::default(), &<()>::default(), &mut <()>::default());
    }
}

mod unit_struct {
    use serde::{Deserialize, Serialize};
    #[derive(Clone, Default, Deserialize, Serialize)]
    struct UnitStruct;

    #[test_fuzz::test_fuzz]
    fn target(
        unit_struct: UnitStruct,
        ref_unit_struct: &UnitStruct,
        ref_mut_unit_struct: &mut UnitStruct,
    ) {
        super::consume(unit_struct);
        super::consume(ref_unit_struct);
        super::consume(ref_mut_unit_struct);
    }

    #[test]
    fn test() {
        target(
            UnitStruct::default(),
            &UnitStruct::default(),
            &mut UnitStruct::default(),
        );
    }
}

mod unit_variant {
    #![allow(clippy::use_self)]

    use serde::{Deserialize, Serialize};
    #[derive(Clone, Deserialize, Serialize)]
    enum UnitVariant {
        A,
        B,
    }

    #[test_fuzz::test_fuzz]
    fn target(
        unit_variant: UnitVariant,
        ref_unit_variant: &UnitVariant,
        ref_mut_unit_variant: &mut UnitVariant,
    ) {
        super::consume(unit_variant);
        super::consume(ref_unit_variant);
        super::consume(ref_mut_unit_variant);
    }

    #[test]
    fn test() {
        target(UnitVariant::A, &UnitVariant::B, &mut UnitVariant::B);
    }
}

mod newtype_struct {
    use serde::{Deserialize, Serialize};
    #[derive(Clone, Default, Deserialize, Serialize)]
    struct NewtypeStruct(u8);

    #[test_fuzz::test_fuzz]
    fn target(
        newtype_struct: NewtypeStruct,
        ref_newtype_struct: &NewtypeStruct,
        ref_mut_newtype_struct: &mut NewtypeStruct,
    ) {
        super::consume(newtype_struct);
        super::consume(ref_newtype_struct);
        super::consume(ref_mut_newtype_struct);
    }

    #[test]
    fn test() {
        target(
            NewtypeStruct::default(),
            &NewtypeStruct::default(),
            &mut NewtypeStruct::default(),
        );
    }
}

mod newtype_variant {
    use serde::{Deserialize, Serialize};
    #[derive(Clone, Deserialize, Serialize)]
    enum NewtypeVariant {
        N(u8),
    }

    impl Default for NewtypeVariant {
        fn default() -> Self {
            Self::N(u8::default())
        }
    }

    #[test_fuzz::test_fuzz]
    fn target(
        newtype_variant: NewtypeVariant,
        ref_newtype_variant: &NewtypeVariant,
        ref_mut_newtype_variant: &mut NewtypeVariant,
    ) {
        super::consume(newtype_variant);
        super::consume(ref_newtype_variant);
        super::consume(ref_mut_newtype_variant);
    }

    #[test]
    fn test() {
        target(
            NewtypeVariant::default(),
            &NewtypeVariant::default(),
            &mut NewtypeVariant::default(),
        );
    }
}

mod seq {
    use std::collections::HashSet;
    #[test_fuzz::test_fuzz]
    fn target(
        seq_slice: &[u8],
        seq_vec: Vec<u8>,
        ref_seq_vec: &Vec<u8>,
        ref_mut_seq_vec: &mut Vec<u8>,
        seq_hash_set: HashSet<u8>,
        ref_seq_hash_set: &HashSet<u8>,
        ref_mut_seq_hash_set: &mut HashSet<u8>,
    ) {
        super::consume(seq_slice);
        super::consume(seq_vec);
        super::consume(ref_seq_vec);
        super::consume(ref_mut_seq_vec);
        super::consume(seq_hash_set);
        super::consume(ref_seq_hash_set);
        super::consume(ref_mut_seq_hash_set);
    }

    #[test]
    fn test() {
        target(
            <&[u8]>::default(),
            Vec::<u8>::default(),
            &Vec::<u8>::default(),
            &mut Vec::<u8>::default(),
            HashSet::<u8>::default(),
            &HashSet::<u8>::default(),
            &mut HashSet::<u8>::default(),
        );
    }
}

mod tuple {
    #[test_fuzz::test_fuzz]
    fn target(
        tuple_u8: (u8,),
        ref_tuple_u8: &(u8,),
        ref_mut_tuple_u8: &mut (u8,),
        tuple_u8_u8: (u8, u8),
        ref_tuple_u8_u8: &(u8, u8),
        ref_mut_tuple_u8_u8: &mut (u8, u8),
    ) {
        super::consume(tuple_u8);
        super::consume(ref_tuple_u8);
        super::consume(ref_mut_tuple_u8);
        super::consume(tuple_u8_u8);
        super::consume(ref_tuple_u8_u8);
        super::consume(ref_mut_tuple_u8_u8);
    }

    #[test]
    fn test() {
        target(
            <(u8,)>::default(),
            &<(u8,)>::default(),
            &mut <(u8,)>::default(),
            <(u8, u8)>::default(),
            &<(u8, u8)>::default(),
            &mut <(u8, u8)>::default(),
        );
    }
}

mod tuple_struct {
    use serde::{Deserialize, Serialize};
    #[derive(Clone, Default, Deserialize, Serialize)]
    struct TupleStruct(u8, u8, u8);

    #[test_fuzz::test_fuzz]
    fn target(
        tuple_struct: TupleStruct,
        ref_tuple_struct: &TupleStruct,
        ref_mut_tuple_struct: &mut TupleStruct,
    ) {
        super::consume(tuple_struct);
        super::consume(ref_tuple_struct);
        super::consume(ref_mut_tuple_struct);
    }

    #[test]
    fn test() {
        target(
            TupleStruct::default(),
            &TupleStruct::default(),
            &mut TupleStruct::default(),
        );
    }
}

mod tuple_variant {
    use serde::{Deserialize, Serialize};
    #[derive(Clone, Deserialize, Serialize)]
    enum TupleVariant {
        T(u8, u8),
    }

    impl Default for TupleVariant {
        fn default() -> Self {
            Self::T(u8::default(), u8::default())
        }
    }

    #[test_fuzz::test_fuzz]
    fn target(
        tuple_variant: TupleVariant,
        ref_tuple_variant: &TupleVariant,
        ref_mut_tuple_variant: &mut TupleVariant,
    ) {
        super::consume(tuple_variant);
        super::consume(ref_tuple_variant);
        super::consume(ref_mut_tuple_variant);
    }

    #[test]
    fn test() {
        target(
            TupleVariant::default(),
            &TupleVariant::default(),
            &mut TupleVariant::default(),
        );
    }
}

mod map {
    use std::collections::BTreeMap;
    #[test_fuzz::test_fuzz]
    fn target(
        map: BTreeMap<u8, u8>,
        ref_map: &BTreeMap<u8, u8>,
        ref_mut_map: &mut BTreeMap<u8, u8>,
    ) {
        super::consume(map);
        super::consume(ref_map);
        super::consume(ref_mut_map);
    }

    #[test]
    fn test() {
        target(
            BTreeMap::<u8, u8>::default(),
            &BTreeMap::<u8, u8>::default(),
            &mut BTreeMap::<u8, u8>::default(),
        );
    }
}

mod strukt {
    use serde::{Deserialize, Serialize};
    #[derive(Clone, Default, Deserialize, Serialize)]
    struct Struct {
        r: u8,
        g: u8,
        b: u8,
    }

    #[test_fuzz::test_fuzz]
    fn target(strukt: Struct, ref_strukt: &Struct, ref_mut_strukt: &mut Struct) {
        super::consume(strukt);
        super::consume(ref_strukt);
        super::consume(ref_mut_strukt);
    }

    #[test]
    fn test() {
        target(
            Struct::default(),
            &Struct::default(),
            &mut Struct::default(),
        );
    }
}

mod struct_variant {
    use serde::{Deserialize, Serialize};
    #[derive(Clone, Deserialize, Serialize)]
    enum StructVariant {
        S { r: u8, g: u8, b: u8 },
    }

    impl Default for StructVariant {
        fn default() -> Self {
            Self::S {
                r: u8::default(),
                g: u8::default(),
                b: u8::default(),
            }
        }
    }

    #[test_fuzz::test_fuzz]
    fn target(
        struct_variant: StructVariant,
        ref_struct_variant: &StructVariant,
        ref_mut_struct_variant: &mut StructVariant,
    ) {
        super::consume(struct_variant);
        super::consume(ref_struct_variant);
        super::consume(ref_mut_struct_variant);
    }

    #[test]
    fn test() {
        target(
            StructVariant::default(),
            &StructVariant::default(),
            &mut StructVariant::default(),
        );
    }
}
