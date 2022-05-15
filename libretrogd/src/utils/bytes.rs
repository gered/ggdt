pub trait ReadFixedLengthByteArray {
    fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N], std::io::Error>;
}

impl<T: std::io::Read> ReadFixedLengthByteArray for T {
    fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N], std::io::Error> {
        assert_ne!(N, 0);
        let mut array = [0u8; N];
        self.read_exact(&mut array)?;
        Ok(array)
    }
}
