#[macro_export]
macro_rules! define_integral_enums {
    (
        $(
            $(
                #[derive( $($derive:path),* )]
            )?
            $(
                #[ $($attr:tt)* ]
            )*
            @easy $name:ident = $( $variant:ident ),*
        );*
        $(;)?
    ) => {
        $crate::define_integral_enums! {
            $(
                $( #[derive( $($derive),* )] )?
                $( #[$($attr)*] )*
                enum $name {
                    $( $variant ),*
                }
            )*
        }
    };

    (
        $(
            $(
                #[derive( $($derive:path),* )]
            )?
            $(
                #[ $($attr:tt)* ]
            )*
            enum $name:ident {
                $(
                    $variant:ident $( = $integral_value:expr )?
                ),*
                $(,)?
            }
        )*
    ) => {
        $(
            #[derive(
                Debug, Clone,
                Copy, PartialEq,
                Eq,
                $( $derive ),*
            )]
            $(
                #[$($attr)*]
            )*
            pub enum $name {
                $(
                    $variant $( = $integral_value )?
                ),*
            }
        )*
    };
}

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
