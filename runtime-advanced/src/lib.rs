mod future;
mod http;
mod runtime;
use crate::future::Future;
use crate::http::HttpGetFuture;
use crate::runtime::init_runtime;

pub fn async_main() {
    let mut runtime = init_runtime();
    let future = HttpGetFuture::new("/600/HelloWorld1".into());
    runtime.block_on(future);
}
