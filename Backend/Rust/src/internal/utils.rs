use super::frame_type::FrameError;

pub fn constraint_to_degree(deg: f32) -> f32 {
    let mut result = deg;
    if result > 360.0 {
        result = result - (360 * (result / 360.0) as u32) as f32;
    }

    else if result < 0.0 {
        result = result + (360 * (result / -360.0) as u32) as f32;
    }

    result
}


pub fn byte_slice_to_u32(bytes: &[u8]) -> Result<u32, FrameError> {
    if bytes.len() < 4 {
        return Err(FrameError::NotEnoughBytes);
    }
    let bytes = [ bytes[0], bytes[1], bytes[2], bytes[3]];
    Ok(u32::from_be_bytes(bytes))

}

pub fn byte_slice_to_u16(bytes: &[u8]) -> Result<u16, FrameError> {
    if bytes.len() < 2 {
        return Err(FrameError::NotEnoughBytes);
    }
    let bytes = [ bytes[0], bytes[1]];
    Ok(u16::from_be_bytes(bytes))

}
