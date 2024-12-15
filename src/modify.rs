use bytes::{Bytes, BytesMut};

/// modify the json output for the bytes hosting.
pub(crate) fn modify_json_output(body_bytes: Bytes) -> Bytes {
    let buffer = body_bytes.as_ref();
    let target_host = b"127.0.0.1";
    let replacement_host = crate::HOST_NAME.as_bytes();

    let (target_port, replacement_port) = *crate::TARGET_REPLACEMENT;

    // Estimate a suitable capacity
    let mut modified_buffer =
        BytesMut::with_capacity(buffer.len() + (replacement_host.len() - target_host.len()));
    let mut start = 0;

    // Replace occurrences of the target host
    while let Some(pos) = buffer[start..]
        .windows(target_host.len())
        .position(|window| window == target_host)
    {
        modified_buffer.extend_from_slice(&buffer[start..start + pos]);
        modified_buffer.extend_from_slice(replacement_host);
        start += pos + target_host.len();
    }
    modified_buffer.extend_from_slice(&buffer[start..]);

    // Now handle the port replacement
    let mut final_buffer = BytesMut::with_capacity(
        modified_buffer.len() + (replacement_port.len() - target_port.len()),
    );
    start = 0;

    while let Some(pos) = modified_buffer[start..]
        .windows(target_port.len())
        .position(|window| window == target_port)
    {
        final_buffer.extend_from_slice(&modified_buffer[start..start + pos]);
        final_buffer.extend_from_slice(replacement_port);
        start += pos + target_port.len();
    }
    final_buffer.extend_from_slice(&modified_buffer[start..]);

    final_buffer.freeze()
}
