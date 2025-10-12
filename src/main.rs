mod body;
mod config;
mod log;
mod proxy;
mod serve;
mod service;

#[tokio::main]
async fn main() {
	let args = config::parse();

	// 存在チェックなど
	if !args.input.exists() {
		println!("入力ファイルが見つかりません: {}", args.input.display());
		return;
	}

	// 入力JSON（実運用ではファイルやstdinから読んでください）
	let router = match config::load(&args) {
		Ok(v) => v,
		Err(v) => {
			println!("Error: {v}");
			return;
		}
	};

	let _ = serve::serve(router.frontend, RebabProxy { router }).await;
	log::log("exit");
}
struct RebabProxy {
	router: crate::config::Router,
}
impl crate::proxy::Proxy for RebabProxy {
	fn uri2uri(&self, uri: &hyper::Uri) -> Option<hyper::Uri> {
		let path_q = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

		// URI を組み立て
		self.router
			.rules
			.iter()
			.filter(|v| match &v.frontend_prefix {
				None => true,
				Some(v) => path_q.starts_with(v),
			})
			.map(|v| {
				let target_uri = format!(
					"http://{}{}{}",
					match &v.backend_host {
						Some(v) => v,
						None => "localhost",
					},
					match v.backend_port.or(uri.port_u16()) {
						Some(v) => format!(":{v}"),
						None => "".to_string(),
					},
					path_q
				);
				// 文字列 → hyper::Uri にパース
				target_uri.parse::<hyper::Uri>().unwrap_or_else(|e| {
					eprintln!("invalid URI generated: {e} (from {target_uri})");
					hyper::Uri::from_static("/") // フォールバック
				})
			})
			.next()
	}
}
