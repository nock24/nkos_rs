macro_rules! const_assert {
    ($cond:expr) => {
        const { assert!($cond) }
    };
}

/// To be used outside items (e.g. functions, impls).
macro_rules! const_block {
    { $($body:tt)* } => {
        const _: () = { $($body)* };
    };
}

/// To be used outside items (e.g. functions, impls).
macro_rules! const_assert_item {
    ($cond:expr) => {
        const_block! { assert!($cond) }
    };
}
