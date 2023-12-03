use bytes::{BufMut as _, Bytes};
use http_body::Body as HttpBody;
use http_body_util::BodyExt as _;

pub async fn http_body_to_bytes<T>(body: T) -> Result<Bytes, T::Error>
where
    T: HttpBody,
{
    Ok(body.collect().await?.to_bytes())
}

pub async fn http_body_to_bytes_with_max_length<T>(
    mut body: T,
    max_length: usize,
) -> Result<Bytes, Box<dyn std::error::Error + Send + Sync>>
where
    T: HttpBody + Unpin,
{
    let mut buf = Vec::with_capacity(max_length);
    while let Some(Ok(frame)) = body.frame().await {
        match frame.into_data() {
            Ok(data) => buf.put(data),
            Err(_frame) => {}
        }
        if buf.len() >= max_length {
            return Ok(buf.into());
        }
    }
    Ok(buf.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::convert::Infallible;

    use http_body::Frame;
    use http_body_util::{Full, StreamBody};

    #[tokio::test]
    async fn test_http_body_to_bytes() {
        let body = Full::<Bytes>::from("foo");
        assert_eq!(
            http_body_to_bytes(body).await.unwrap(),
            Bytes::copy_from_slice(b"foo")
        );
    }

    #[tokio::test]
    async fn test_http_body_to_bytes_with_max_length() {
        let body = Full::<Bytes>::from("foobar");
        assert_eq!(
            http_body_to_bytes_with_max_length(body, 3).await.unwrap(),
            Bytes::copy_from_slice(b"foobar")
        );

        let chunks: Vec<Result<_, Infallible>> = vec![
            Ok(Frame::data(Bytes::from("hello"))),
            Ok(Frame::data(Bytes::from(" "))),
            Ok(Frame::data(Bytes::from("world"))),
        ];
        let body = StreamBody::new(futures_util::stream::iter(chunks));
        assert_eq!(
            http_body_to_bytes_with_max_length(body, 3).await.unwrap(),
            Bytes::copy_from_slice(b"hello")
        );

        let chunks: Vec<Result<_, Infallible>> = vec![
            Ok(Frame::data(Bytes::from("fo"))),
            Ok(Frame::data(Bytes::from("o"))),
            Ok(Frame::data(Bytes::from("bar"))),
        ];
        let body = StreamBody::new(futures_util::stream::iter(chunks));
        assert_eq!(
            http_body_to_bytes_with_max_length(body, 3).await.unwrap(),
            Bytes::copy_from_slice(b"foo")
        );
    }
}
