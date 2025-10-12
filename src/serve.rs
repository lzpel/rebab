use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

use hyper::server;
use hyper_util::rt::TokioIo;

pub async fn serve(
	addr: SocketAddr,
	proxy: impl crate::proxy::Proxy,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
	crate::log::log(format!("start listen {}", addr));
	let listener = match TcpListener::bind(addr).await {
		Ok(v) => Ok(v),
		Err(e) => {
			crate::log::log(format!("port already used {}", addr));
			Err(e)
		}
	}?;
	// https://github.com/hyperium/hyper/discussions/3471
	let proxy = Arc::new(proxy);
	loop {
		let (stream, _) = listener.accept().await?;
		let io = TokioIo::new(stream);
		let proxy = proxy.clone();
		tokio::task::spawn(async move {
			let svc = crate::service::ProxyHandler { proxy };
			if let Err(err) = server::conn::http1::Builder::new()
				.serve_connection(io, svc)
				.await
			{
				eprintln!("server error: {}", err);
			}
		});
	}
}
