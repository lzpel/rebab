pub trait Proxy: Send + Sync + 'static {
	fn uri2uri(&self, uri: &hyper::Uri) -> Option<hyper::Uri>;
}
