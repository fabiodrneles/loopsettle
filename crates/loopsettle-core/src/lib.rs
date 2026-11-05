//! loopsettle-core (placeholder, see CHECKPOINT.md).

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn smoke(a in 0i64..1000, b in 0i64..1000) { prop_assert_eq!(a + b, b + a); }
    }
}
