#![recursion_limit = "1024"]
use std::slice;

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

    pub fn from(c_pkg: &CPackage) -> Self {
        let msg_type : MessageType = if c_pkg.message_type == 0 {
            MessageType::Query
        } else {
            MessageType::Response
        };
        let mut query : Option<String> = None;
        if c_pkg.query_len > 0 {
            let query_str = unsafe {::std::str::from_utf8(slice::from_raw_parts(c_pkg.query, c_pkg.query_len)).unwrap()};
            query = Some(String::from(query_str));
        };
        let mut payload : Option<String> = None;
        if c_pkg.payload_len > 0 {
            let payload_str = unsafe {::std::str::from_utf8(slice::from_raw_parts(c_pkg.payload, c_pkg.payload_len)).unwrap()};
            payload = Some(String::from(payload_str));
        };
        Package {
            id: c_pkg.id,
            message_type: msg_type,
            bold: c_pkg.bold,
            italic: c_pkg.italic,
            underlined: c_pkg.underlined,
            blink: c_pkg.blink,
            red: c_pkg.red,
            green: c_pkg.green,
            blue: c_pkg.blue,
            query,
            payload,
        }
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

    pub fn set_query(&mut self, query: Option<String>){
        self.query = match query {
            None => None,
            Some(q) => Some(String::from(q.as_str()))
        };
    }

    pub fn query_len(&self) -> usize {
        match self.query {
            None => 0,
            Some(ref q) => q.as_bytes().len(),
        }
    }

    pub fn set_payload(mut self, payload: Option<String>) -> Package {
        self.payload = match payload {
            None => None,
            Some(q) => Some(String::from(q.as_str()))
        };
        self
    }

    pub fn payload_len(&self) -> usize {
        match self.payload {
            None => 0,
            Some(ref p) => p.as_bytes().len(),
        }
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
                let q_bytes = query.as_bytes();
                encoder.write_u16(q_bytes.len() as u16)?;
                encoder.write_slice(q_bytes)?;
            },
        }

        match self.payload {
            None => {
                encoder.write_u16(0)?;
            },
            Some(ref payload) => {
                let p_bytes = payload.as_bytes();
                encoder.write_u16(p_bytes.len() as u16)?;
                encoder.write_slice(p_bytes)?;
            },
        }

        Ok(encoder.len())
    }
}


#[repr(C)]
pub struct CPackage {
    pub id: u16,
    pub message_type: u8,
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub blink: bool,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub query_len: usize,
    pub query: *mut u8,
    pub payload_len: usize,
    pub payload: *mut u8,
}


use std::ptr;

impl From<Package> for CPackage {
    fn from(pkg: Package) -> CPackage {
        let msg_type : u8 = match pkg.message_type {
            MessageType::Query => 0,
            MessageType::Response => 1
        };
        let mut q_len : usize = 0;
        let mut q_ptr : *mut u8 = ptr::null_mut();
        match pkg.query {
            None => {},
            Some(q) => {
                q_len = q.len();
                q_ptr = Box::into_raw(q.into_boxed_str()) as *mut u8;
            },
        };
        let mut p_len : usize = 0;
        let mut p_ptr : *mut u8 = ptr::null_mut();
        match pkg.payload {
            None => {},
            Some(p) => {
                p_len = p.len();
                p_ptr = Box::into_raw(p.into_boxed_str()) as *mut u8;
            },
        };
        CPackage {
            id: pkg.id,
            message_type: msg_type,
            bold: pkg.bold,
            italic: pkg.italic,
            underlined: pkg.underlined,
            blink: pkg.blink,
            red: pkg.red,
            green: pkg.green,
            blue: pkg.blue,
            query_len: q_len,
            query: q_ptr,
            payload_len: p_len,
            payload: p_ptr,
        }
    }
}

#[no_mangle]
pub extern "C" fn decode_package(buffer: *const u8, len: usize) -> *mut CPackage {
    let buf : &[u8] = unsafe { slice::from_raw_parts(buffer, len) };
    let mut decoder = Decoder::new(buf);
    let pkg : CPackage = Package::read(&mut decoder).unwrap().into();
    Box::into_raw(Box::new(pkg))
}

#[no_mangle]
pub extern "C" fn encode_package(package: *const CPackage, buffer: *mut *mut u8, len: *mut usize) -> i32 {
    if package.is_null() || len.is_null() {
        return -1
    }

    let mut calculated_size : usize = 8;  // Size of the static fields
    unsafe {
        let pkg : &CPackage = &*package;
        calculated_size += pkg.payload_len + pkg.query_len;
    }

    let mut buf : Vec<u8> = Vec::with_capacity(calculated_size);
    unsafe {
        let c_pkg : &CPackage = &*package;
        let pkg = Package::from(c_pkg);
        let mut encoder = Encoder::new(&mut buf);
        let written = pkg.write(&mut encoder).unwrap_or(0_usize);
        if written == 0 {
            println!("Gah, wie died!");
            return -1
        }
        *len = written;
    }
    unsafe {
        let buf_box = buf.into_boxed_slice();
        *buffer = Box::into_raw(buf_box) as *mut u8;
    }
    0
}


#[no_mangle]
pub extern "C" fn free_package(package: *mut CPackage) {
    if !package.is_null() {
        unsafe {
            let c_pkg = Box::from_raw(package);
            let _pkg = Package::from(&c_pkg);
            // and drop it
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_buffer(buffer: *mut u8, len: usize) {
    if !buffer.is_null() {
        let _buf = Box::from_raw(slice::from_raw_parts_mut(buffer, len));
        // and drop it
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
