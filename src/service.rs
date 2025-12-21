use crate::proxy::Proxy;
use hyper::http::uri::Authority;
use hyper::{Request, Response, body::Incoming, header::FORWARDED};
use hyper_util::{
	client::legacy::{Client, connect::HttpConnector},
	rt::TokioExecutor,
};
use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc};

use hyper::http::header::{
	CONNECTION, HOST, HeaderName, HeaderValue, PROXY_AUTHENTICATE, PROXY_AUTHORIZATION, TE,
	TRAILER, TRANSFER_ENCODING, UPGRADE,
};

const HOP_HEADERS: [HeaderName; 7] = [
	CONNECTION,
	TE,
	TRAILER,
	TRANSFER_ENCODING,
	UPGRADE,
	PROXY_AUTHENTICATE,
	PROXY_AUTHORIZATION,
];

// 状態を持つハンドラ構造体
pub struct ProxyHandler<T: Proxy> {
	pub proxy: Arc<T>,
}
// Service トレイトを実装
impl<T: Proxy> hyper::service::Service<Request<Incoming>> for ProxyHandler<T> {
	type Response = Response<crate::body::RebabBody>;
	type Error = Infallible;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn call(&self, req: Request<Incoming>) -> Self::Future {
		let args = self.proxy.clone();
		Box::pin(async move {
			let resp = proxy(args.as_ref(), req).await;
			Ok(resp)
		})
	}
}

pub async fn proxy(proxy: &impl Proxy, req: Request<Incoming>) -> Response<crate::body::RebabBody> {
	//https://hyper.rs/guides/1/server/middleware/
	//Ok(Response::new(req.uri().to_string()))
	// クライアント（接続再利用したいなら外に出して Arc 共有してOK）
	let mut connector = HttpConnector::new();
	connector.enforce_http(true);
	let client: Client<_, Incoming> = Client::builder(TokioExecutor::new()).build(connector);

	// 元リクエストをパーツに分解
	let (parts, body) = req.into_parts();

	let new_uri = match proxy.uri2uri(&parts.uri) {
		Some(v) => v,
		None => return response(404, format!("rebab no route for {}", parts.uri.to_string())),
	};
	// 新しいリクエストを作成（メソッド/URIはコピー）
	let mut out_req = Request::builder()
		.method(&parts.method)
		.uri(&new_uri)
		.header(HOST, new_uri.host().unwrap().to_string())
		.body(body)
		.expect("building forwarded request");

	// ヘッダのコピー（hop-by-hop は削除、Host は上書き）
	{
		let src = &parts.headers;
		let dst = out_req.headers_mut();

		for (name, value) in src.iter() {
			if HOP_HEADERS.iter().all(|v| v != name) && name != &HOST {
				dst.append(name, value.clone());
			}
		}
		// ==== ここから追記：元のホスト情報を転送 ====
		if let Some(orig) = original_authority(&parts)
			&& let Some(orig_proto) = proto(&parts)
		{
			// 例: localhost:8080
			let _orig_host = orig.host();
			let orig_port = orig.port_u16();
			dst.insert(
				HeaderName::from_static("x-forwarded-host"),
				HeaderValue::from_str(orig.as_str()).unwrap(),
			); // 例: localhost:8080
			dst.insert(
				HeaderName::from_static("x-forwarded-proto"),
				HeaderValue::from_str(&orig_proto).unwrap(),
			);
			if let Some(p) = orig_port {
				dst.insert(
					HeaderName::from_static("x-forwarded-port"),
					HeaderValue::from_str(&p.to_string()).unwrap(),
				);
			}
			// Forwarded: proto=http;host="localhost:8080"
			dst.append(
				FORWARDED,
				HeaderValue::from_str(&format!(r#"proto={};host="{}""#, orig_proto, orig.as_str()))
					.unwrap(),
			);
		}
	}
	// 転送してレスポンスを受け取る
	let resp = match client.request(out_req).await {
		Ok(resp) => resp,
		Err(e) => {
			return response(502, format!("Rebab Bad Gateway: {e:?}"));
		}
	};

	// レスポンスから hop-by-hop ヘッダ除去
	let (mut parts, body) = resp.into_parts();
	// RFC的には Connection ヘッダに列挙されたフィールドも落とすべきだが、
	// まずは代表的 hop-by-hop を除去
	for name in HOP_HEADERS {
		parts.headers.remove(name);
	}
	Response::from_parts(parts, crate::body::RebabBody::Incoming(body))
}

fn response(status: u16, body: String) -> Response<crate::body::RebabBody> {
	Response::builder()
		.status(status)
		.body(crate::body::RebabBody::from(body))
		.unwrap()
}

fn original_authority(parts: &hyper::http::request::Parts) -> Option<Authority> {
	// 1) 絶対URIなら URI の authority を優先
	if let Some(a) = parts.uri.authority().cloned() {
		return Some(a);
	}
	// 2) 通常は Host ヘッダ
	parts
		.headers
		.get(HOST)
		.and_then(|v| v.to_str().ok())
		.and_then(|s| s.parse::<Authority>().ok())
}

pub fn proto(parts: &hyper::http::request::Parts) -> Option<String> {
	let host = parts
		.headers
		.get(HOST)
		.and_then(|h| h.to_str().ok())
		.map(str::trim)
		.filter(|s| !s.is_empty())?
		.to_string();
	let hostname = host
		.trim_start_matches('[')
		.split(']')
		.next()
		.unwrap_or(&host)
		.split(':')
		.next()
		.unwrap_or(&host);
	let scheme = match hostname {
		"localhost" | "127.0.0.1" | "::1" => "http",
		_ => "https",
	};
	Some(scheme.to_string())
}
