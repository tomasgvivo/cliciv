use byteorder::{LittleEndian, WriteBytesExt};

pub trait AsBytes {
    fn as_bytes(&self) -> Vec<u8>;
}

impl AsBytes for f64 {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bs = [0u8; std::mem::size_of::<Self>()];
        bs.as_mut()
            .write_f64::<LittleEndian>(*self)
            .expect("Unable to write");

        bs.to_vec()
    }
}

pub trait RoundTo2 {
    fn round_to_2(&self) -> Self;
}

impl RoundTo2 for f64 {
    fn round_to_2(&self) -> Self {
        (self * 100.0).round() / 100.0
    }
}
