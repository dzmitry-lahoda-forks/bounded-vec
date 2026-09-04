use core::convert::TryInto;

use const_bounded_collections::*;

#[test]
fn from_vec() {
    assert!(BoundedVec::<u8, 2, 8>::from_vec(vec![1, 2]).is_ok());
    assert!(BoundedVec::<u8, 2, 8>::from_vec(vec![]).is_err());
    assert!(BoundedVec::<u8, 3, 8>::from_vec(vec![1, 2]).is_err());
    assert!(BoundedVec::<u8, 1, 2>::from_vec(vec![1, 2, 3]).is_err());
}
#[test]
fn is_empty() {
    let data: EmptyBoundedVec<_, 8> = vec![1u8, 2].try_into().unwrap();
    assert!(!data.is_empty());
}

#[test]
fn as_vec() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.as_vec(), &vec![1u8, 2]);
}

#[test]
fn as_slice() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.as_slice(), &[1u8, 2]);
}

#[test]
fn len() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.len(), 2);
}

#[test]
fn first() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.first(), &1u8);
}

#[test]
fn last() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.last(), &2u8);
}

#[test]
fn mapped() {
    let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    let data = data.mapped(|x| x * 2);
    assert_eq!(data, [2u8, 4].into());
}

#[test]
fn mapped_ref() {
    let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    let data = data.mapped_ref(|x| x * 2);
    assert_eq!(data, [2u8, 4].into());
}

#[test]
fn get() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.get(1).unwrap(), &2u8);
    assert!(data.get(3).is_none());
}

#[test]
fn try_mapped() {
    let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    let data = data.try_mapped(|x| 100u8.checked_div(x).ok_or("error"));
    assert_eq!(data, Ok([100u8, 50].into()));
}

#[test]
fn try_mapped_error() {
    let data: BoundedVec<u8, 2, 8> = [0u8, 2].into();
    let data = data.try_mapped(|x| 100u8.checked_div(x).ok_or("error"));
    assert_eq!(data, Err("error"));
}

#[test]
fn try_mapped_ref() {
    let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    let data = data.try_mapped_ref(|x| 100u8.checked_div(*x).ok_or("error"));
    assert_eq!(data, Ok([100u8, 50].into()));
}

#[test]
fn try_mapped_ref_error() {
    let data: BoundedVec<u8, 2, 8> = [0u8, 2].into();
    let data = data.try_mapped_ref(|x| 100u8.checked_div(*x).ok_or("error"));
    assert_eq!(data, Err("error"));
}

#[test]
fn split_last() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    assert_eq!(data.split_last(), (&2u8, [1u8].as_ref()));
    let data1: BoundedVec<_, 1, 8> = vec![1u8].try_into().unwrap();
    assert_eq!(data1.split_last(), (&1u8, Vec::new().as_ref()));
}

#[test]
fn enumerated() {
    let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    let expected: BoundedVec<_, 2, 8> = vec![(0, 1u8), (1, 2)].try_into().unwrap();
    assert_eq!(data.enumerated(), expected);
}

#[test]
fn into_iter() {
    let mut vec = vec![1u8, 2];
    let mut data: BoundedVec<_, 2, 8> = vec.clone().try_into().unwrap();
    assert_eq!(data.clone().into_iter().collect::<Vec<u8>>(), vec);
    assert_eq!(
        data.iter().collect::<Vec<&u8>>(),
        vec.iter().collect::<Vec<&u8>>()
    );
    assert_eq!(
        data.iter_mut().collect::<Vec<&mut u8>>(),
        vec.iter_mut().collect::<Vec<&mut u8>>()
    );
}

#[test]
fn deref_and_slice_methods() {
    let mut data: BoundedVec<i32, 2, 8> = vec![3, 1, 2].try_into().unwrap();
    assert_eq!(data[0], 3);
    assert_eq!(&data[1..], &[1, 2]);
    data.sort_unstable();
    assert_eq!(data.as_slice(), &[1, 2, 3]);
    assert_eq!(data.binary_search(&2), Ok(1));
    data[0] = 10;
    assert_eq!(data[0], 10);
}

#[test]
fn insert_and_try_insert() {
    let mut data: BoundedVec<i32, 1, 3> = vec![1, 3].try_into().unwrap();
    data.insert(1, 2);
    assert_eq!(data.as_slice(), &[1, 2, 3]);
    assert!(data.try_insert(0, 0).is_err());
    assert_eq!(data.len(), 3);
}

#[test]
fn capacity_and_reserve() {
    let mut data: BoundedVec<i32, 1, 10> = vec![1].try_into().unwrap();
    data.reserve(5);
    assert!(data.capacity() >= 6);
    data.shrink_to_fit();
}

#[test]
fn empty_bounded_vec_operations() {
    let mut v: EmptyBoundedVec<i32, 5> = EmptyBoundedVec::new();
    assert!(v.is_empty());
    assert_eq!(v.first_mut(), None);
    assert_eq!(v.last_mut(), None);
    v.push(1);
    v.push(2);
    v.push(2);
    v.push(3);
    assert_eq!(v.first_mut(), Some(&mut 1));
    assert_eq!(v.last_mut(), Some(&mut 3));
    v.dedup();
    assert_eq!(v.as_slice(), &[1, 2, 3]);
    assert_eq!(v.remove(1), 2);
    assert_eq!(v.as_slice(), &[1, 3]);
    assert_eq!(v.pop(), Some(3));
    v.clear();
    assert!(v.is_empty());
}

#[test]
fn non_empty_vec_operations() {
    let mut v: NonEmptyVec<i32> = NonEmptyVec::new(1);
    assert_eq!(*v.first(), 1);
    assert_eq!(*v.first_mut(), 1);
    assert_eq!(*v.last(), 1);
    v.push(2);
    v.push(2);
    v.push(3);
    v.dedup();
    assert_eq!(v.as_slice(), &[1, 2, 3]);
    assert_eq!(v.split_first(), (&1, &[2, 3][..]));
    assert_eq!(v.split_last(), (&3, &[1, 2][..]));
    assert_eq!(v.try_pop(), Ok(3));
    assert_eq!(v.try_pop(), Ok(2));
    assert!(v.try_pop().is_err());
    assert_eq!(v.len(), 1);
    assert!(v.try_remove(0).is_err());
}

#[test]
fn from_head_tail() {
    let v: NonEmptyVec<i32> = NonEmptyVec::from_head_tail(1, vec![2, 3]).unwrap();
    assert_eq!(v.as_slice(), &[1, 2, 3]);
}

#[test]
fn extend_impl() {
    let mut v: BoundedVec<i32, 1, 8> = vec![1].try_into().unwrap();
    v.extend(vec![2, 3]);
    assert_eq!(v.as_slice(), &[1, 2, 3]);
    let more = [4, 5];
    v.extend(&more);
    assert_eq!(v.as_slice(), &[1, 2, 3, 4, 5]);
}

#[cfg(feature = "borsh")]
mod borsh_tests {
    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize};

    #[test]
    #[allow(clippy::expect_used)]
    fn borsh_encdec() {
        let data: BoundedVec<u8, 2, 8> = vec![1u8, 2].try_into().expect("borsh works");
        let buf = &mut Vec::new();
        data.serialize(buf).expect("borsh works");
        let decoded =
            BoundedVec::<u8, 2, 8>::deserialize(&mut buf.as_slice()).expect("borsh works");
        let compatible_decoded =
            BoundedVec::<u8, 1, 255>::deserialize(&mut buf.as_slice()).expect("borsh works");
        assert_eq!(data.get(0), decoded.get(0));
        assert_eq!(data.get(1), decoded.get(1));
        assert_eq!(data.get(0), compatible_decoded.get(0));
        assert_eq!(data.get(1), compatible_decoded.get(1));
        assert!(BoundedVec::<u8, 1, 257>::deserialize(&mut buf.as_slice()).is_err());

        let empty_data: EmptyBoundedVec<u8, 8> = Vec::new().try_into().expect("borsh works");
        let buf = &mut Vec::new();
        empty_data.serialize(buf).expect("borsh works");
        let decoded_empty =
            EmptyBoundedVec::<u8, 8>::deserialize(&mut buf.as_slice()).expect("borsh works");
        assert_eq!(empty_data.as_slice(), decoded_empty.as_slice());
    }
}

#[cfg(feature = "borsh_schema")]
mod borsh_schema_tests {
    use super::*;
    use borsh::schema::BorshSchemaContainer;

    #[test]
    #[allow(clippy::expect_used)]
    fn borsh_schema() {
        let schema = BorshSchemaContainer::for_type::<BoundedVec<u8, 2, 8>>();
        let schema = schema
            .get_definition("BoundedVec<u8, 2, 8>")
            .expect("borsh works");
        assert!(matches!(
            schema,
            borsh::schema::Definition::Sequence {
                length_width: 1,
                ..
            }
        ));

        let schema_empty = BorshSchemaContainer::for_type::<EmptyBoundedVec<u8, 8>>();
        let schema_empty = schema_empty
            .get_definition("BoundedVec<u8, 0, 8>")
            .expect("borsh works");
        assert!(matches!(
            schema_empty,
            borsh::schema::Definition::Sequence {
                length_width: 1,
                ..
            }
        ));
    }
}

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;
    use std::vec;

    #[test]
    fn deserialize_nonempty() {
        assert_eq!(
            serde_json::from_str::<BoundedVec::<u8, 2, 3>>("[2, 3]")
                .unwrap()
                .as_vec(),
            &vec![2, 3]
        );
    }

    #[test]
    fn deserialize_empty() {
        assert!(serde_json::from_str::<EmptyBoundedVec::<u8, 3>>("[]").is_ok());
        assert!(serde_json::from_str::<BoundedVec::<u8, 2, 3>>("[]").is_err());
    }
}

#[cfg(feature = "schemars")]
mod schema_tests {
    use super::*;
    use schemars::schema_for;

    #[test]
    fn json_schema() {
        let root_schema = schema_for!(BoundedVec<u8, 2, 8>);
        let schema_value = serde_json::to_value(&root_schema).unwrap();
        let min_items = schema_value["minItems"].as_u64().unwrap() as u32;
        let max_items = schema_value["maxItems"].as_u64().unwrap() as u32;
        assert_eq!(min_items, 2);
        assert_eq!(max_items, 8);
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::len_zero)]
mod arb_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn const_bounded_collections_length_bounded(v: BoundedVec<u8, 1, 2>) {
            prop_assert!(1 <= v.len() && v.len() <= 2);
        }
    }
}
