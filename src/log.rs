pub fn log(message: impl AsRef<str>) {
	println!("rebab: {}", message.as_ref())
}

pub fn addr_to_url(addr: std::net::SocketAddr) -> String {
	let host = if addr.ip().is_unspecified() {
		"localhost".to_string()
	} else {
		let ip = addr.ip();
		if ip.is_ipv6() {
			format!("[{}]", ip)
		} else {
			ip.to_string()
		}
	};
	format!("http://{}:{}", host, addr.port())
}
