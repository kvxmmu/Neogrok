pub const fn identity<T>(i: T) -> T {
    i
}

pub const fn cold() {}

/// Mark branch as cold path.
#[cold]
pub const fn cold_value<T>(v: T) -> T {
    cold();
    v
}

/// Mark branch as likely to be false.
///
/// Correct work is not guaranteed since it relays on
/// compiler optimizations
pub const fn unlikely(cond: bool) -> bool {
    if cond {
        cold_value(true)
    } else {
        cold_value(false)
    }
}

/// Mark branch as likely to be true.
///
/// Correct work is not guaranteed since it relays on
/// compiler optimizations
pub const fn likely(cond: bool) -> bool {
    if cond {
        true
    } else {
        cold_value(false)
    }
}
