use crate::proxy::Proxy;
use hyper::{Request, Response, body::Incoming};
use hyper_util::{
	client::legacy::{Client, connect::HttpConnector},
	rt::TokioExecutor,
};
use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc};

use hyper::http::header::{
	CONNECTION, HOST, HeaderName, PROXY_AUTHENTICATE, PROXY_AUTHORIZATION, TE, TRAILER,
	TRANSFER_ENCODING, UPGRADE,
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
		.method(parts.method)
		.uri(new_uri)
		.body(body)
		.expect("building forwarded request");

	// ヘッダのコピー（hop-by-hop は削除、Host は上書き）
	{
		let src = parts.headers;
		let dst = out_req.headers_mut();

		for (name, value) in src.iter() {
			if HOP_HEADERS.iter().all(|v| v != name) && name != &HOST {
				dst.append(name, value.clone());
			}
		}
		// Host をバックエンドに合わせる
		//dst.insert(HOST, HeaderValue::from_static("127.0.0.1:8000"));
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
