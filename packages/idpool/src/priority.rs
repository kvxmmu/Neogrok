use {
    num::Integer,
    std::{
        collections::BinaryHeap,
        marker::PhantomData,
    },
};

macro_rules! impl_trait {
    ($trait:ident, $for:ty) => {
        impl $trait for $for {
            #[inline(always)]
            fn reverse(self) -> Self {
                Self::MAX - self
            }
        }
    };
}

pub trait ReverseExt {
    fn reverse(self) -> Self;
}

pub trait OrderingExt {
    fn compute<T: ReverseExt>(val: T) -> T;
}

#[derive(Debug)]
pub enum LowToHigh {}

#[derive(Debug)]
pub enum HighToLow {}

#[derive(Debug)]
pub struct PriorityIdPool<T, Ordering> {
    inner: BinaryHeap<T>,
    current: T,

    ordering: PhantomData<Ordering>,
}

impl_trait!(ReverseExt, i8);
impl_trait!(ReverseExt, i16);
impl_trait!(ReverseExt, i32);
impl_trait!(ReverseExt, i64);
impl_trait!(ReverseExt, i128);

impl_trait!(ReverseExt, u8);
impl_trait!(ReverseExt, u16);
impl_trait!(ReverseExt, u32);
impl_trait!(ReverseExt, u64);
impl_trait!(ReverseExt, u128);

impl OrderingExt for LowToHigh {
    fn compute<T: ReverseExt>(val: T) -> T {
        val.reverse()
    }
}

impl OrderingExt for HighToLow {
    fn compute<T: ReverseExt>(val: T) -> T {
        val
    }
}

impl<T, Ordering> PriorityIdPool<T, Ordering>
where
    T: Integer + Copy + ReverseExt,
    Ordering: OrderingExt,
{
    pub fn return_id(&mut self, id: T) {
        self.inner.push(Ordering::compute(id));
    }

    pub fn request_id(&mut self) -> T {
        if let Some(id) = self.inner.pop() {
            Ordering::compute(id)
        } else {
            let prev = self.current;
            self.current = self.current + T::one();

            prev
        }
    }

    pub fn zero_with_capacity(capacity: usize) -> Self {
        Self {
            inner: BinaryHeap::with_capacity(capacity),
            current: T::zero(),
            ordering: PhantomData,
        }
    }

    pub fn zero() -> Self {
        Self {
            inner: Default::default(),
            current: T::zero(),
            ordering: PhantomData,
        }
    }

    pub fn with_capacity(current: T, capacity: usize) -> Self {
        Self {
            inner: BinaryHeap::with_capacity(capacity),
            current,
            ordering: PhantomData,
        }
    }

    pub fn new(current: T) -> Self {
        Self {
            inner: Default::default(),
            current,
            ordering: PhantomData,
        }
    }
}
