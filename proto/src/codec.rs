use byteorder::{ByteOrder, NetworkEndian};

use errors::{Result};

pub struct Decoder<'a> {
    buffer: &'a [u8],
    index: usize,
}

impl<'a> Decoder<'a> {
    /// Create a new `Decoder`
    ///
    /// # Arguments
    ///
    /// * `buffer` from which all data will be read.
    pub fn new(buffer: &'a [u8]) -> Self {
        Decoder {
            buffer: buffer,
            index: 0,
        }
    }

    /// Read a larger slice from the buffer
    pub fn read_slice(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self.index + length;
        if end > self.buffer.len() {
            bail!("Buffer drained")
        }
        let slice: &'a [u8] = &self.buffer[self.index..end];
        self.index += length;
        Ok(slice)
    }

    /// Read a byte from the buffer
    pub fn read_u8(&mut self) -> Result<u8> {
        // No need for using a slice
        if self.index < self.buffer.len() {
            let byte = self.buffer[self.index];
            self.index += 1;
            Ok(byte)
        } else {
            bail!("Index error")
        }
    }

    /// Read a u16 from the buffer, converting from network byte order
    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(NetworkEndian::read_u16(self.read_slice(2)?))
    }
    ///
    /// Read a u32 from the buffer, converting from network byte order
    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(NetworkEndian::read_u32(self.read_slice(4)?))
    }
}


pub struct Encoder<'a> {
    buffer: &'a mut Vec<u8>,
    index: usize,
}

impl<'a> Encoder<'a> {
    /// Create a new `Encoder`
    pub fn new(buffer: &'a mut Vec<u8>) -> Self {
        Encoder {
            buffer,
            index: 0,
        }
    }

    /// Write a slice
    pub fn write_slice(&mut self, data: &[u8]) -> Result<usize> {
        self.buffer.extend_from_slice(data);
        self.index += data.len();
        Ok(data.len())
    }

    /// Write a u8
    pub fn write_u8(&mut self, data: u8) -> Result<usize> {
        self.buffer.push(data);
        self.index += 1;
        Ok(1)
    }

    /// Write an u16
    pub fn write_u16(&mut self, data: u16) -> Result<usize> {
        let mut encoded = [0; 2];
        {
            NetworkEndian::write_u16(&mut encoded, data);
        }
        self.write_slice(&encoded)
    }

    /// Write a u32
    pub fn write_u32(&mut self, data: u32) -> Result<usize> {
        let mut encoded = [0; 4];
        {
            NetworkEndian::write_u32(&mut encoded, data);
        }
        self.write_slice(&encoded)
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn into_bytes(self) -> &'a Vec<u8> {
        self.buffer
    }
}


pub trait Serialisable<T: Sized> {
    fn read(decoder: &mut Decoder) -> Result<T>;
    fn write(&self, encoder: &mut Encoder) -> Result<usize>;
}

impl Serialisable<u16> for u16 {
    fn read(decoder: &mut Decoder) -> Result<u16> {
        decoder.read_u16()
    }

    fn write(&self, encoder: &mut Encoder) -> Result<usize> {
        encoder.write_u16(*self)
    }
}

impl Serialisable<u32> for u32 {
    fn read(decoder: &mut Decoder) -> Result<u32> {
        decoder.read_u32()
    }

    fn write(&self, encoder: &mut Encoder) -> Result<usize> {
        encoder.write_u32(*self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read() {
        let deadbeef = b"deadbeef!";
        let mut decoder = Decoder::new(deadbeef);

        let read = decoder.read_slice(2).expect("Failed to read 'de'");
        assert_eq!(read, b"de");

        let read = decoder.read_u16().expect("Failed to read 'ad'");
        assert_eq!(read, 0x6164);

        let read = decoder.read_u32().expect("Failed to read 'beef'");
        assert_eq!(read, 0x62656566);

        let read = decoder.read_u8().expect("Failed to read '!'");
        assert_eq!(read, 0x21);

        assert!(decoder.read_u8().is_err());
    }

    #[test]
    fn test_write() {
        let mut result: Vec<u8> = Vec::new();
        let mut encoder = Encoder::new(&mut result);
        let expected = b"deadbeef!";

        encoder.write_slice(b"de").unwrap();
        encoder.write_u16(0x6164).unwrap();
        encoder.write_u32(0x62656566).unwrap();
        encoder.write_u8(0x21).unwrap();

        assert_eq!(encoder.buffer, expected);
    }
}
