mod body;
mod config;
mod log;
mod process;
mod proxy;
mod serve;
mod service;

#[tokio::main]
async fn main() {
	let args = config::parse();

	// Check if input file exists
	if let Some(path) = &args.input {
		if !path.exists() {
			println!("Input file not found: {}", path.display());
			return;
		}
	}

	// Load configuration
	let router = match config::load(&args) {
		Ok(v) => v,
		Err(v) => {
			println!("Error: {v}");
			return;
		}
	};

	// Create process manager and wrap in Arc
	let process_manager = std::sync::Arc::new(process::ProcessManager::new());

	// Execute commands for each rule
	for (index, rule) in router.rules.iter().enumerate() {
		if let Some(command) = &rule.command {
			let rule_id = format!("rule_{}", index);
			if let Err(e) = process_manager.spawn_command(rule_id, command, rule.backend_port) {
				log::log(&format!("Command execution error: {}", e));
				log::log("Terminating all processes");
				process_manager.terminate_all();
				return;
			}
		}
	}

	// Start process monitoring task
	let pm_for_monitor = process_manager.clone();
	let monitor_handle = tokio::spawn(async move {
		let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
		loop {
			interval.tick().await;
			if let Err(e) = pm_for_monitor.check_all() {
				log::log(&format!("Process monitoring error: {}", e));
				log::log("Terminating all processes");
				pm_for_monitor.terminate_all();
				std::process::exit(1);
			}
		}
	});

	// Start server
	let serve_result = serve::serve(router.frontend, RebabProxy { router }).await;

	// Cleanup on server exit
	monitor_handle.abort();
	process_manager.terminate_all();

	if let Err(e) = serve_result {
		log::log(&format!("Server error: {}", e));
	}

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
