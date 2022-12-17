#[macro_export]
macro_rules! impl_transmute {
    ($cls:ty) => {
        impl TryFrom<u8> for $cls {
            type Error = ();

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                let last = Self::Reserved as u8;
                if value >= last {
                    Err(())
                } else {
                    Ok(unsafe { std::mem::transmute(value) })
                }
            }
        }
    };
}

pub mod compression;
pub mod protocol;
