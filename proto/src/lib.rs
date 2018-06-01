#![recursion_limit = "1024"]

/// Parser library for the SambaXP 2018 demo protocol
///
/// ```text
/// The demo packets look like the following:
///
///                                    1  1  1  1  1  1
///      0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                      ID                       |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |QR|BD|IT|UL|BL|Reserved|         Red           |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |        Green          |         Blue          |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     | Query ...                                     |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     | ...                                           |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     | Payload ...                                   |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     | ...                                           |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///
/// Where:
///     ID      ID of the message requested
///     QR      0 when query, 1 when response
///     BD      1 when text should be bold
///     IT      1 when text should be italic
///     UL      1 when text should be underlined
///     BL      1 when text should blink
///     Red     u8 of red channel intensity
///     Green   u8 of green channel intensity
///     Blue    u8 of blue channel intensity
///
/// Both Query and Payload start with a u16 length value
/// followed by a utf-8 encoded string.


#[macro_use]
extern crate error_chain;

extern crate byteorder;

mod errors {
    error_chain! {}
}

mod codec;

pub use codec::*;
use errors::*;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Package {
    pub id: u16,
    pub message_type: MessageType,
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub blink: bool,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub query: Option<String>,
    pub payload: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum MessageType {
    Query,
    Response,
}

impl Default for Package {
    fn default() -> Self {
        Package {
            id: 0,
            message_type: MessageType::Query,
            bold: false,
            italic: false,
            underlined: false,
            blink: false,
            red: 0,
            green: 0,
            blue: 0,
            query: None,
            payload: None,
        }
    }
}

impl Package {
    /// Create a new `Package`
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_id(mut self, id: u16) -> Package {
        self.id = id;
        self
    }

    pub fn set_message_type(mut self, mtype: MessageType) -> Package {
        self.message_type = mtype;
        self
    }

    pub fn set_bold(mut self, bold: bool) -> Package {
        self.bold = bold;
        self
    }
    pub fn set_italic(mut self, italic: bool) -> Package {
        self.italic = italic;
        self
    }
    pub fn set_underlined(mut self, underlined: bool) -> Package {
        self.underlined = underlined;
        self
    }
    pub fn set_blink(mut self, blink: bool) -> Package {
        self.blink = blink;
        self
    }

    pub fn set_rgb(mut self, red: u8, green: u8, blue: u8) -> Package {
        self.red = red;
        self.green = green;
        self.blue = blue;
        self
    }

    pub fn set_query(mut self, query: Option<String>) -> Package {
        self.query = query;
        self
    }
    pub fn set_payload(mut self, payload: Option<String>) -> Package {
        self.payload = payload;
        self
    }
}

impl Serialisable<Package> for Package {
    fn read(decoder: &mut Decoder) -> errors::Result<Self> {
        let id = decoder.read_u16().chain_err(|| "reading ID failed")?;

        // Parse the bit flag field
        let flags = decoder.read_u8().chain_err(|| "reading bitflags failed")?;
        let message_type = if (0b1000_0000 & flags) == 0b1000_0000 {
            MessageType::Response
        } else {
            MessageType::Query
        };
        let bold =       (0b0100_0000 & flags) == 0b0100_0000;
        let italic =     (0b0010_0000 & flags) == 0b0010_0000;
        let underlined = (0b0001_0000 & flags) == 0b0001_0000;
        let blink =      (0b0000_1000 & flags) == 0b0000_1000;

        let red = decoder.read_u8().chain_err(|| "reading red failed")?;
        let green = decoder.read_u8().chain_err(|| "reading green failed")?;
        let blue = decoder.read_u8().chain_err(|| "reading blue failed")?;

        let len = decoder.read_u16().chain_err(|| "reading string length failed")?;
        let query;
        if len > 0 {
            let raw_query = decoder.read_slice(len as usize).chain_err(|| "reading the query failed")?;
            query = Some(String::from_utf8(raw_query.to_vec()).chain_err(|| "converting the query failed")?);
        } else {
            query = None;
        }

        let len = decoder.read_u16().chain_err(|| "reading string length failed")?;
        let payload;
        if len > 0 {
            let raw_payload = decoder.read_slice(len as usize).chain_err(|| "reading the payload failed")?;
            payload = Some(String::from_utf8(raw_payload.to_vec()).chain_err(|| "converting the payload failed")?);
        } else {
            payload = None;
        }

        Ok(Package{
            id,
            message_type,
            bold,
            italic,
            underlined,
            blink,
            red,
            green,
            blue,
            query,
            payload,
        })
    }

    fn write(&self, encoder: &mut Encoder) -> errors::Result<usize> {
        encoder.write_u16(self.id).chain_err(|| "writing ID failed")?;

        let mut flags : u8 = match self.message_type {
            MessageType::Query => 0,
            MessageType::Response => 0b1000_0000,
        };
        if self.bold {
            flags |= 0b0100_0000;
        }
        if self.italic {
            flags |= 0b0010_0000;
        }
        if self.underlined {
            flags |= 0b0001_0000;
        }
        if self.blink {
            flags |= 0b0000_1000;
        }
        encoder.write_u8(flags).chain_err(|| "writing bitflags failed")?;

        encoder.write_u8(self.red)?;
        encoder.write_u8(self.green)?;
        encoder.write_u8(self.blue)?;

        match self.query {
            None => {
                encoder.write_u16(0)?;
            },
            Some(ref query) => {
                encoder.write_u16(query.len() as u16)?;
                encoder.write_slice(query.as_bytes())?;
            },
        }

        match self.payload {
            None => {
                encoder.write_u16(0)?;
            },
            Some(ref payload) => {
                encoder.write_u16(payload.len() as u16)?;
                encoder.write_slice(payload.as_bytes())?;
            },
        }

        Ok(encoder.len())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read() {
        let buffer = vec![
            0x23, 0x42,  // ID
            0b1100_1000, // response, bold, blink
            0x12,
            0x34,
            0x56,
            0x00, 0x02,  // len
            0x48,        // H
            0x69,        // i
            0x00, 0x00,  // len
        ];

        let mut decoder = Decoder::new(&buffer);

        let expected = Package{
            id: 0x2342,
            message_type: MessageType::Response,
            bold: true,
            italic: false,
            underlined: false,
            blink: true,
            red: 0x12,
            green: 0x34,
            blue: 0x56,
            query: Some(String::from("Hi")),
            payload: None,
        };

        let got = Package::read(&mut decoder).unwrap();

        assert_eq!(expected, got);

    }

    #[test]
    fn test_write() {
        let expected: Vec<u8> = vec![
            0x23, 0x42,  // ID
            0b1100_1000, // response, bold, blink
            0x12,
            0x34,
            0x56,
            0x00, 0x02,  // len
            0x48,        // H
            0x69,        // i
            0x00, 0x00,  // len
        ];

        let package = Package {
            id: 0x2342,
            message_type: MessageType::Response,
            bold: true,
            italic: false,
            underlined: false,
            blink: true,
            red: 0x12,
            green: 0x34,
            blue: 0x56,
            query: Some(String::from("Hi")),
            payload: None,
        };

        let mut buffer: Vec<u8> = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);

        let written = package.write(&mut encoder).unwrap();

        assert_eq!(written, expected.len());
        assert_eq!(*encoder.into_bytes(), expected);

    }
}
