use bytes::{Buf as _, BufMut as _, Bytes};
use http_body::Body as HttpBody;

// Copy from https://github.com/hyperium/hyper/blob/v0.14.24/src/body/to_bytes.rs#L47
pub async fn http_body_to_bytes<T>(body: T) -> Result<Bytes, T::Error>
where
    T: HttpBody,
{
    futures_util::pin_mut!(body);

    // If there's only 1 chunk, we can just return Buf::to_bytes()
    let mut first = if let Some(buf) = body.data().await {
        buf?
    } else {
        return Ok(Bytes::new());
    };

    let second = if let Some(buf) = body.data().await {
        buf?
    } else {
        return Ok(first.copy_to_bytes(first.remaining()));
    };

    // Don't pre-emptively reserve *too* much.
    let rest = (body.size_hint().lower() as usize).min(1024 * 16);
    let cap = first
        .remaining()
        .saturating_add(second.remaining())
        .saturating_add(rest);
    // With more than 1 buf, we gotta flatten into a Vec first.
    let mut vec = Vec::with_capacity(cap);
    vec.put(first);
    vec.put(second);

    while let Some(buf) = body.data().await {
        vec.put(buf?);
    }

    Ok(vec.into())
}

pub async fn http_body_to_bytes_with_max_length<T>(
    body: T,
    max_length: usize,
) -> Result<Bytes, T::Error>
where
    T: HttpBody,
{
    futures_util::pin_mut!(body);

    // If there's only 1 chunk, we can just return Buf::to_bytes()
    let mut first = if let Some(buf) = body.data().await {
        buf?
    } else {
        return Ok(Bytes::new());
    };

    if first.chunk().len() >= max_length {
        return Ok(first.copy_to_bytes(first.remaining()));
    }

    let second = if let Some(buf) = body.data().await {
        buf?
    } else {
        return Ok(first.copy_to_bytes(first.remaining()));
    };

    if first.chunk().len() + second.chunk().len() >= max_length {
        // Don't pre-emptively reserve *too* much.
        let rest = (body.size_hint().lower() as usize).min(1024 * 16);
        let cap = first
            .remaining()
            .saturating_add(second.remaining())
            .saturating_add(rest);
        // With more than 1 buf, we gotta flatten into a Vec first.
        let mut vec = Vec::with_capacity(cap);
        vec.put(first);
        vec.put(second);

        return Ok(vec.into());
    }

    // Don't pre-emptively reserve *too* much.
    let rest = (body.size_hint().lower() as usize).min(1024 * 16);
    let cap = first
        .remaining()
        .saturating_add(second.remaining())
        .saturating_add(rest);
    // With more than 1 buf, we gotta flatten into a Vec first.
    let mut vec = Vec::with_capacity(cap);
    vec.put(first);
    vec.put(second);

    while let Some(buf) = body.data().await {
        vec.put(buf?);

        if vec.len() >= max_length {
            return Ok(vec.into());
        }
    }

    Ok(vec.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures_util::stream;
    use hyper::Body as HyperBody;

    // Test copy from https://github.com/bk-rs/hyper-ext/blob/main/hyper-body-to-bytes/src/lib.rs
    #[tokio::test]
    async fn test_http_body_to_bytes() {
        let hyper_body = HyperBody::from("foo");
        assert_eq!(
            http_body_to_bytes(hyper_body).await.unwrap(),
            Bytes::copy_from_slice(b"foo")
        );
    }

    // Test copy from https://github.com/bk-rs/hyper-ext/blob/main/hyper-body-to-bytes/src/lib.rs
    #[tokio::test]
    async fn test_http_body_to_bytes_with_max_length() {
        let hyper_body = HyperBody::from("foobar");
        assert_eq!(
            http_body_to_bytes_with_max_length(hyper_body, 3)
                .await
                .unwrap(),
            Bytes::copy_from_slice(b"foobar")
        );

        let chunks: Vec<Result<_, std::io::Error>> = vec![Ok("hello"), Ok(" "), Ok("world")];
        let hyper_body = HyperBody::wrap_stream(stream::iter(chunks));
        assert_eq!(
            http_body_to_bytes_with_max_length(hyper_body, 3)
                .await
                .unwrap(),
            Bytes::copy_from_slice(b"hello")
        );

        let chunks: Vec<Result<_, std::io::Error>> = vec![Ok("fo"), Ok("o"), Ok("bar")];
        let hyper_body = HyperBody::wrap_stream(stream::iter(chunks));
        assert_eq!(
            http_body_to_bytes_with_max_length(hyper_body, 3)
                .await
                .unwrap(),
            Bytes::copy_from_slice(b"foo")
        );
    }
}
