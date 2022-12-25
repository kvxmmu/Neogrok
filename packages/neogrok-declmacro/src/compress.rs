#[macro_export]
macro_rules! __define_copyable_compress_errors {
    (
        $(
            $(
                #[derive(
                    $($derive:path),*
                )]
            )?
            enum $name:ident {
                $(
                    $variant:ident = $data:expr
                ),*
                $(,)?
            }
        )*
    ) => {
        $(
            #[derive(
                Debug, Clone,
                Copy, PartialEq,
                Eq, thiserror::Error,
                $(
                    $derive
                ),*
            )]
            pub enum $name {
                $(
                    #[error( $data )]
                    $variant
                ),*
            }
        )*
    };
}

pub use crate::__define_copyable_compress_errors as define_copyable_compress_errors;
