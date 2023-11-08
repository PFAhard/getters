#[macro_export]
macro_rules! as_ref_impl {
    ($t:ty) => {
        impl AsRef<$t> for $t {
            fn as_ref(&self) -> &$t {
                self
            }
        }
    };
}
#[macro_export]
macro_rules! struct_def {
    ($s:ident $([$($d:ident), *])? { $($f: ident: $t:ty: $r:ty), * }) => {
        $(#[derive($($d, )*)])?
        pub struct $s {
            $(
                $f: $t,
            )*
        }

        impl $s {
            pub fn new( $($f: $t, )*) -> Self {
                Self {
                    $($f, )*
                }
            }

            $(
                pub fn $f(&self) -> $r {
                    &self.$f
                }
            )*

            pub fn inner_tuple(&self) -> ($($r, )*) {
                ($(&self.$f, )*)
            }   
        }
    };
}

#[macro_export]
macro_rules! enum_def {
    ($e:ident $([$($d:ident), *])? { $($v:ident), * }) => {
        $(#[derive($($d, )*)])?
        pub enum $e {
            $(
                $v,
            )*
        }
        paste::paste! {
            impl $e {
                $(
                    pub fn [<is_ $v:snake:lower>](&self) -> bool {
                        matches!(self, Self::$v)
                    }
                )*
            }
        }

        impl Display for $e {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $e::$v => write!(f, "{}", stringify!($v)),
                    )*
                }
            }
        }
    };
    ($e:ident $([$($d:ident), *])? { $($v:ident ($t:ty)), * }) => {
        $(#[derive($($d, )*)])?
        pub enum $e {
            $(
                $v($t),
            )*
        }
        paste::paste! {
            impl $e {
                $(
                    pub fn [<is_ $v:snake:lower>](&self) -> bool {
                        matches!(self, Self::$v(_))
                    }

                    pub fn [<get_ $v:snake:lower >](&self) -> &$t {
                        if let Self::$v(d) = self {
                            d
                        } else {
                            unreachable!("Attemp to ues specific enum ({}) getter ({}), without checking type", stringify!([<$e>]), stringify!([<get_ $v:snake:lower >]))
                        }
                    }
                )*
            }
        }

        impl Display for $e {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $e::$v => write!(f, "{}", stringify!($v)),
                    )*
                }
            }
        }
    };
}
