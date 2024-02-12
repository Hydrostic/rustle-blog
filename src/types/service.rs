

pub trait AppService{
    async fn initialize() -> EmptyErrResult<()>;
    fn name() -> &'static str;
}


macro_rules! init_services {
    ($($srv:ident),*) => {{
		use crate::types::service::AppService;
        if false { unreachable!() }
		$(
			else if let Err(_) = $srv::initialize().await {
				tracing::error!("failed to initialize service: {}", $srv::name());
				false
			}
		)*
		else { true }
    }};
}

pub(crate) use init_services;
use crate::types::err::EmptyErrResult;