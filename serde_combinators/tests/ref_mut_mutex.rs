use serde::{Deserialize, Serialize};
use serde_assert::{Deserializer, Serializer, Token};
use serde_combinators::{MutexF, RefMutF, Type, With};
use std::sync::Mutex;

#[derive(Deserialize, Serialize, Debug)]
struct Struct<'a> {
    #[serde(with = "RefMutF::<MutexF<Type<_>>>")]
    ref_mut_mutex: &'a mut Mutex<i32>,
}

impl Struct<'_> {
    fn swap(&mut self, other: &mut Self) {
        let mut x = self.ref_mut_mutex.lock().unwrap();
        let mut y = other.ref_mut_mutex.lock().unwrap();
        std::mem::swap::<i32>(&mut x, &mut y)
    }
}

impl PartialEq for Struct<'_> {
    fn eq(&self, other: &Self) -> bool {
        let x = self.ref_mut_mutex.lock().unwrap();
        let y = other.ref_mut_mutex.lock().unwrap();
        *x == *y
    }
}

#[test]
fn swap() {
    let mut mutex_a = Mutex::new(0);
    let mut mutex_b = Mutex::new(1);
    let mut strukt_a = Struct {
        ref_mut_mutex: &mut mutex_a,
    };
    let mut strukt_b = Struct {
        ref_mut_mutex: &mut mutex_b,
    };
    strukt_a.swap(&mut strukt_b);
    assert_eq!(1, *mutex_a.lock().unwrap());
    assert_eq!(0, *mutex_b.lock().unwrap());
}

#[cfg_attr(
    dylint_lib = "assert_eq_arg_misordering",
    allow(assert_eq_arg_misordering)
)]
#[test]
fn serde() {
    let mut mutex = Mutex::new(0);
    let strukt = Struct {
        ref_mut_mutex: &mut mutex,
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
            Token::Field("ref_mut_mutex"),
            Token::I32(0),
            Token::StructEnd
        ]
    );
    let mut deserializer = Deserializer::builder(tokens).build();
    let other = Struct::deserialize(&mut deserializer).unwrap();
    assert_eq!(strukt, other);
}
