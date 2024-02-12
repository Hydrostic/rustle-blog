use clap::Parser;
use once_cell::sync::Lazy;
use crate::types::arg::Arg;

pub static ARGS: Lazy<Arg> = Lazy::new(||{
    Arg::parse()
});

#[macro_export]
macro_rules! get_args {
    ($child:ident) => {
        {
            get_args!().$child
        }
    };
    () => {
        {
            use once_cell::sync::Lazy;
            use crate::internal::arg::ARGS;
            Lazy::force(&ARGS)
        }
    };
}
