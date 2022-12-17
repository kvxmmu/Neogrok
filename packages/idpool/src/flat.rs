use num::Integer;

#[derive(Debug)]
pub struct FlatIdPool<T> {
    inner: Vec<T>,
    current: T,
}

impl<T> FlatIdPool<T>
where
    T: Integer + Copy,
{
    pub fn request_id(&mut self) -> T {
        if let Some(id) = self.inner.pop() {
            id
        } else {
            let prev = self.current;
            self.current = self.current + T::one();

            prev
        }
    }

    pub fn return_id(&mut self, id: T) {
        self.inner.push(id);
    }

    pub fn with_capacity(current: T, capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            current,
        }
    }

    pub fn new(current: T) -> Self {
        Self {
            inner: Default::default(),
            current,
        }
    }

    pub fn zero_with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            current: T::zero(),
        }
    }

    pub fn zero() -> Self {
        Self {
            inner: Default::default(),
            current: T::zero(),
        }
    }
}
