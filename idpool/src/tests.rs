use crate::{
    flat::*,
    priority::*,
};

#[test]
fn test_priority_idpool() {
    let mut pool = PriorityIdPool::<u32, LowToHigh>::zero();

    assert_eq!(pool.request_id(), 0);
    assert_eq!(pool.request_id(), 1);
    assert_eq!(pool.request_id(), 2);

    pool.return_id(1);
    pool.return_id(2);
    pool.return_id(0);

    assert_eq!(pool.request_id(), 0);
    assert_eq!(pool.request_id(), 1);
    assert_eq!(pool.request_id(), 2);
}

#[test]
fn test_flat_idpool() {
    let mut pool = FlatIdPool::<u32>::zero();

    assert_eq!(pool.request_id(), 0);
    assert_eq!(pool.request_id(), 1);

    pool.return_id(0);

    assert_eq!(pool.request_id(), 0);
}
