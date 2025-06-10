pub mod parameter;

use std::net::SocketAddr;
use clap::Parser;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, Version, body};
use tokio::net::TcpListener;
use tokio::signal;
#[tokio::main]
async fn main() {
	//https://github.com/hyperium/hyper/blob/master/examples/http_proxy.rs
	//https://github.com/hyperium/hyper/blob/master/examples/graceful_shutdown.rs
	let args = parameter::Args::parse();
	let addr: SocketAddr = ([0, 0, 0, 0], args.port).into(); // プロキシの待ち受けポート
	let listener = TcpListener::bind(addr).await.expect("Can't listen");
	println!("listening on http://{}", addr);
	let max_len = args.route.iter().map(|r| r.path.as_str().len()).max().unwrap_or(0);
	for (i, route) in args.route.iter().enumerate() {
		println!(
			"route {:>2} {:width$} => {}:{}",
			i,
			route.path.as_str(),
			route.port,
			route.path_into,
			width = max_len
		);
	}
	loop {
		tokio::select! {
			Ok((stream, _)) = listener.accept() => {
				tokio::spawn(async move {
					if let Err(err) = process(stream).await {
						eprintln!("Connection error: {:?}", err);
					}
				});
			}

			_ = signal::ctrl_c() => {
				println!("Shutdown signal received. Exiting.");
				break;
			}
		}
	}
}

async fn process(
	stream: tokio::net::TcpStream,
) -> hyper::Result<()> {
	let service = service_fn(proxy_handler);
	let io = hyper_util::rt::TokioIo::new(stream);
	http1::Builder::new()
		.keep_alive(true)
		.serve_connection(io, service)
		.await
}
async fn proxy_handler(req: Request<body::Incoming>) -> Result<Response<std::string::String>, &'static str> {
	if req.version() == Version::HTTP_11 {
		Ok(Response::builder()
			.body("Hello World".to_string())
			.unwrap()
		)

	} else {
		Err("not HTTP/ 1.1, abort connection")
	}
}
