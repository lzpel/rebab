use std::{
	pin::Pin,
	task::{Context, Poll},
};

use hyper::body::{Body, Bytes, Frame, Incoming, SizeHint};

// --- 自前 Body ---------------------------------------------------------------
/// Incoming（ストリーム）か、静的 Bytes かを一つの Body で表現
pub enum RebabBody {
	Incoming(Incoming),
	Static(Option<Bytes>), // 1回だけ data() を返して終わる
}

impl From<Incoming> for RebabBody {
	fn from(b: Incoming) -> Self {
		RebabBody::Incoming(b)
	}
}
impl From<String> for RebabBody {
	fn from(s: String) -> Self {
		RebabBody::Static(Some(Bytes::from(s)))
	}
}
impl From<Bytes> for RebabBody {
	fn from(b: Bytes) -> Self {
		RebabBody::Static(Some(b))
	}
}

impl Body for RebabBody {
	type Data = Bytes;
	// Incoming は hyper::Error を返すので、双方に共通なエラー型として Box<dyn Error + Send + Sync> に寄せる
	type Error = Box<dyn std::error::Error + Send + Sync>;

	fn poll_frame(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
		let this = self.get_mut();
		match this {
			RebabBody::Incoming(inc) => {
				// Incoming のフレームをそのまま中継（エラーは Box 化）
				let pinned = Pin::new(inc);
				match pinned.poll_frame(cx) {
					Poll::Ready(Some(Ok(frame))) => {
						// Data は Bytes なのでそのまま
						Poll::Ready(Some(Ok(frame.map_data(|d| d))))
					}
					Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(Box::new(e)))),
					Poll::Ready(None) => Poll::Ready(None),
					Poll::Pending => Poll::Pending,
				}
			}
			RebabBody::Static(slot) => {
				if let Some(bytes) = slot.take() {
					Poll::Ready(Some(Ok(Frame::data(bytes))))
				} else {
					Poll::Ready(None)
				}
			}
		}
	}

	fn size_hint(&self) -> SizeHint {
		match self {
			RebabBody::Incoming(inc) => inc.size_hint(),
			RebabBody::Static(Some(b)) => SizeHint::with_exact(b.len() as u64),
			RebabBody::Static(None) => SizeHint::with_exact(0),
		}
	}
}
