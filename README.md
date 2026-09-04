
## Const bounded collections
`BoundedVec<T, L, U>` - Non-empty Rust `alloc::vec::Vec` wrapper with type guarantees on lower(`L`) and upper(`U`) bounds for items quantity (`0 < L <= U`).
`EmptyBoundedVec<T,U>` if only upper bound `U` is needed.
`NonEmptyVec<T>` if a lower bound of one and the largest supported upper bound are needed (`usize::MAX`, or `u32::MAX` with `schemars` or `borsh`).
This crate is `#![no_std]` compatible with `alloc`.

## Example

```rust
use const_bounded_collections::BoundedVec;

let data: BoundedVec<u8, 2, 4> = [1u8,2].into();

assert_eq!(*data.first(), 1);
assert_eq!(*data.last(), 2);

// creates a new BoundedVec by mapping each element
let data = data.mapped(|x|x*2);
assert_eq!(data, [2u8,4].into());
```

## Crate features
- no features enabled by default
- optional(non-default) [serde](https://serde.rs/) feature that adds serialization and deserialization to `BoundedVec`.
- optional(non-default) `schemars` feature that adds JSON schema support via [schemars](https://graham.cool/schemars) (requires `serde`).
- optional(non-default) `arbitrary` feature that adds `proptest::Arbitrary` implementation to `BoundedVec`.
- optional(non-default) `borsh` feature that adds `borsh` binary encoding and decoding.
- optional(non-default) `borsh_schema` feature that adds `borsh` schema support (requires `borsh`).
- optional(non-default) `panic` feature that adds `push`, `insert`, and `Extend`, which panic on upper bound violations, plus mutable access to the underlying `Vec`, which can bypass the bounds. Fallible `try_push`, `try_insert`, and `try_extend` are available without this feature; invalid indices can still panic.
- optional(non-default) `nightly` for experimental allocator support (not supported by the stable build).


# Inspired 

- [vec1](https://github.com/rustonaut/vec1)
- [bounded-vec](https://github.com/ergoplatform/bounded-vec) - i added bounded witness to this crate