// use salvo::Router;
//
// use crate::internal::config::{StaticDevMode, CONSOLE_DEV_MODE};
//
// use salvo::proxy::Proxy;
// pub fn init() -> Router{
//     match CONSOLE_DEV_MODE.get().unwrap(){
//         StaticDevMode::Url(t) => {
//             Router::with_path("/console/<**rest>")
//                 .goal(Proxy::default_hyper_client(t.to_owned()))
//         },
//         StaticDevMode::Local(_) => todo!(),
//     }
// }