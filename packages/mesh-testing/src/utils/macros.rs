/// Helpful macro to create strings from enums
#[macro_export]
macro_rules! enum_str {
    (enum $name:ident {
        $($variant:ident),*,
    }) => {
        enum $name {
            $($variant),*
        }

        impl $name {
            fn to_str(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),*
                }
            }
        }
    };
}
