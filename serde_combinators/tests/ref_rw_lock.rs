use serde::{Deserialize, Serialize};
use serde_assert::{Deserializer, Serializer, Token};
use serde_combinators::{RefF, RwLockF, Type, With};
use std::sync::RwLock;

#[derive(Deserialize, Serialize, Debug)]
struct Struct<'a> {
    #[serde(with = "RefF::<RwLockF<Type<_>>>")]
    ref_rw_lock: &'a RwLock<i32>,
}

impl PartialEq for Struct<'_> {
    fn eq(&self, other: &Self) -> bool {
        let x = self.ref_rw_lock.read().unwrap();
        let y = other.ref_rw_lock.read().unwrap();
        *x == *y
    }
}

#[test]
fn eq() {
    let rw_lock_a = RwLock::new(0);
    let rw_lock_b = RwLock::new(0);
    let rw_lock_c = RwLock::new(1);
    let strukt_a = Struct {
        ref_rw_lock: &rw_lock_a,
    };
    let strukt_b = Struct {
        ref_rw_lock: &rw_lock_b,
    };
    let strukt_c = Struct {
        ref_rw_lock: &rw_lock_c,
    };
    assert_eq!(strukt_a, strukt_b);
    assert_ne!(strukt_a, strukt_c);
}

#[cfg_attr(
    dylint_lib = "assert_eq_arg_misordering",
    allow(assert_eq_arg_misordering)
)]
#[test]
fn serde() {
    let rw_lock = RwLock::new(0);
    let strukt = Struct {
        ref_rw_lock: &rw_lock,
    };
    let serializer = Serializer::builder().build();
    let tokens = strukt.serialize(&serializer).unwrap();
    assert_eq!(
        tokens,
        [
            Token::Struct {
                name: "Struct",
                len: 1
            },
            Token::Field("ref_rw_lock"),
            Token::I32(0),
            Token::StructEnd
        ]
    );
    let mut deserializer = Deserializer::builder(tokens).build();
    let other = Struct::deserialize(&mut deserializer).unwrap();
    assert_eq!(strukt, other);
}
