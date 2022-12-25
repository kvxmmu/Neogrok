#[macro_export]
macro_rules! define_results {
    (
        $(
            $typename:ident<$generic:ident> = <$error:path>
        ),*
        $(,)?
    ) => {
        $(
            pub type $typename<$generic> = core::result::Result<$generic, $error>;
        )*
    };
}

pub mod compress;
