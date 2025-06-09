pub mod parameter;
use clap::Parser;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, Version, body, http};
use tokio::net::TcpListener;
use tokio::signal;
#[tokio::main]
async fn main() {
	//https://github.com/hyperium/hyper/blob/master/examples/http_proxy.rs
	//https://github.com/hyperium/hyper/blob/master/examples/graceful_shutdown.rs
	let args = parameter::Args::parse();
	println!("{:?}", args);
	let addr = ([0, 0, 0, 0], args.port).into(); // プロキシの待ち受けポート
	let listener = TcpListener::bind(addr).await.expect("Can't listen");
	println!("Listening on http://{}", addr);
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
	socket: tokio::net::TcpStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let service = service_fn(proxy_handler);
	http1::Builder::new()
		.keep_alive(true)
		.serve_connection(socket, service)
		.await?;
	Ok(())
}
async fn proxy_handler(req: Request<body::Incoming>) -> Result<http::Response<()>, &'static str> {
	if req.version() == Version::HTTP_11 {
		Ok(Response::builder()
			.version(req.version())
			.body("Hello World".into())
			.unwrap())
	} else {
		Err("not HTTP/ 1.1, abort connection")
	}
}
