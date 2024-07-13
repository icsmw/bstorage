use crate::{Bundle, Storage, E};
use ctor::ctor;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::temp_dir,
    fs::{create_dir, remove_dir_all, remove_file},
};
use uuid::Uuid;

#[ctor]
fn logs() {
    env_logger::init();
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum Case {
    String(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
    Bool(bool),
    VecString(Vec<String>),
    VecBool(Vec<bool>),
    VecU8(Vec<u8>),
    VecU16(Vec<u16>),
    VecU32(Vec<u32>),
    VecU64(Vec<u64>),
    VecU128(Vec<u128>),
    VecUsize(Vec<usize>),
    VecI8(Vec<i8>),
    VecI16(Vec<i16>),
    VecI32(Vec<i32>),
    VecI64(Vec<i64>),
    VecI128(Vec<i128>),
    VecIsize(Vec<isize>),
    Map(HashMap<String, String>),
    Struct(Struct),
    Tuple(String, String),
}

impl Case {
    fn gen() -> Vec<BoxedStrategy<Case>> {
        let mut collected = Vec::new();
        collected.push(any::<u8>().prop_map(Case::U8).boxed());
        collected.push(any::<u16>().prop_map(Case::U16).boxed());
        collected.push(any::<u32>().prop_map(Case::U32).boxed());
        collected.push(any::<u64>().prop_map(Case::U64).boxed());
        collected.push(any::<u128>().prop_map(Case::U128).boxed());
        collected.push(any::<usize>().prop_map(Case::Usize).boxed());
        collected.push(any::<i8>().prop_map(Case::I8).boxed());
        collected.push(any::<i16>().prop_map(Case::I16).boxed());
        collected.push(any::<i32>().prop_map(Case::I32).boxed());
        collected.push(any::<i64>().prop_map(Case::I64).boxed());
        collected.push(any::<i128>().prop_map(Case::I128).boxed());
        collected.push(any::<isize>().prop_map(Case::Isize).boxed());
        collected.push(
            "[a-z][a-z0-9]*"
                .prop_map(String::from)
                .prop_map(Case::String)
                .boxed(),
        );
        collected.push(
            prop_oneof![Just(true), Just(false)]
                .prop_map(Case::Bool)
                .boxed(),
        );
        collected.push(Struct::arbitrary_with(()).prop_map(Case::Struct).boxed());
        collected.push(
            prop::collection::vec(any::<u8>(), 0..100)
                .prop_map(Case::VecU8)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<u16>(), 0..100)
                .prop_map(Case::VecU16)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<u32>(), 0..100)
                .prop_map(Case::VecU32)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<u64>(), 0..100)
                .prop_map(Case::VecU64)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<u128>(), 0..100)
                .prop_map(Case::VecU128)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<usize>(), 0..100)
                .prop_map(Case::VecUsize)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<i8>(), 0..100)
                .prop_map(Case::VecI8)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<i16>(), 0..100)
                .prop_map(Case::VecI16)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<i32>(), 0..100)
                .prop_map(Case::VecI32)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<i64>(), 0..100)
                .prop_map(Case::VecI64)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<i128>(), 0..100)
                .prop_map(Case::VecI128)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(any::<isize>(), 0..100)
                .prop_map(Case::VecIsize)
                .boxed(),
        );
        collected.push(
            prop::collection::vec("[a-z][a-z0-9]*".prop_map(String::from), 0..100)
                .prop_map(Case::VecString)
                .boxed(),
        );
        collected.push(
            prop::collection::vec(prop_oneof![Just(true), Just(false)], 0..100)
                .prop_map(Case::VecBool)
                .boxed(),
        );
        collected.push(
            (
                "[a-z][a-z0-9]*".prop_map(String::from),
                "[a-z][a-z0-9]*".prop_map(String::from),
            )
                .prop_map(|(a, b)| Case::Tuple(a, b))
                .boxed(),
        );
        collected.push(
            prop::collection::vec(
                (
                    "[a-z][a-z0-9]*".prop_map(String::from),
                    "[a-z][a-z0-9]*".prop_map(String::from),
                ),
                0..100,
            )
            .prop_map(|v| {
                let mut map = HashMap::new();
                v.into_iter().for_each(|(k, v)| {
                    map.insert(k, v);
                });
                Case::Map(map)
            })
            .boxed(),
        );
        collected
    }
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
struct Struct {
    a: String,
    b: u64,
    c: Option<String>,
    d: (u8, i128),
    e: Vec<u8>,
}

impl Arbitrary for Struct {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            "[a-z][a-z0-9]*".prop_map(String::from),
            any::<u64>(),
            prop_oneof![Just(true), Just(false)],
            (any::<u8>(), any::<i128>()),
            prop::collection::vec(any::<u8>(), 0..100),
        )
            .prop_map(|(a, b, c_state, d, e)| Struct {
                a: a.clone(),
                b,
                c: if c_state { Some(a) } else { None },
                d,
                e,
            })
            .boxed()
    }
}

impl Arbitrary for Case {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop::strategy::Union::new(Case::gen()).boxed()
    }
}

#[derive(Debug)]
struct Cases {
    cases: Vec<(String, Case)>,
}

impl Arbitrary for Cases {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop::collection::vec(
            (
                "[a-z][a-z0-9]*".prop_map(String::from),
                Case::arbitrary_with(()),
            ),
            0..100,
        )
        .prop_map(|cases| Cases { cases })
        .boxed()
    }
}

fn run_for_unpacked(cases: Cases) -> Result<(), E> {
    let storage_path = temp_dir().join(Uuid::new_v4().to_string());
    create_dir(&storage_path)?;
    let mut storage = Storage::open(&storage_path)?;
    let mut cleaned = HashMap::new();
    cases.cases.into_iter().for_each(|(key, case)| {
        cleaned.insert(key, case);
    });
    for (key, case) in cleaned.iter() {
        storage.set(key, case)?;
    }
    drop(storage);
    let storage = Storage::open(&storage_path)?;
    for (key, case) in cleaned.iter() {
        let stored: Case = storage.get(key)?.unwrap();
        assert_eq!(case, &stored);
    }
    remove_dir_all(storage_path)?;
    Ok(())
}

fn run_for_packed(cases: Cases) -> Result<(), E> {
    let storage_path = temp_dir().join(Uuid::new_v4().to_string());
    let bundle = temp_dir().join(Uuid::new_v4().to_string());
    create_dir(&storage_path)?;
    let mut storage = Storage::open(&storage_path)?;
    let mut cleaned = HashMap::new();
    cases.cases.into_iter().for_each(|(key, case)| {
        cleaned.insert(key, case);
    });
    for (key, case) in cleaned.iter() {
        storage.set(key, case)?;
    }
    storage.pack(&bundle)?;
    drop(storage);
    remove_dir_all(storage_path)?;
    let storage = Storage::unpack(&bundle)?;
    for (key, case) in cleaned.iter() {
        let stored: Case = storage.get(key)?.unwrap();
        assert_eq!(case, &stored);
    }
    remove_dir_all(storage.cwd())?;
    remove_file(&bundle)?;
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 5000,
        ..ProptestConfig::with_cases(100)
    })]
    #[test]
    fn unpacked(
        args in any_with::<Cases>(())
    )  {
        run_for_unpacked(args).unwrap();
    }
    #[test]
    fn packed(
        args in any_with::<Cases>(())
    )  {
        run_for_packed(args).unwrap();
    }
}
