use std::fmt;
use std::io::{Read, Write};

use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use eio::{WriteExt, ToBytes};
use num_traits::PrimInt;

use crate::error::Error;
//
// delimiter
const COMMAND_START: u8 = 0xFF;
const COMMAND_END: u8 = 0xFE;

// commands
const READ_COMMAND: u8 = 0x01;
const WRITE_COMMAND: u8 = 0x02;
const WATCH_COMMAND: u8 = 0x03;
const VIEW_WATCH_COMMAND: u8 = 0x04;
const UNWATCH_COMMAND: u8 = 0x05;

// parameters
const TYPE_SIGNED: u8 = 0x01;
const TYPE_UNSIGNED: u8 = 0x02;
const TYPE_FLOAT: u8 = 0x03;

pub fn read_u8(stream: &mut (impl Read+Write), address: u32) -> Result<u8> {
    send_read_command::<u8>(stream, address)?;
    Ok(stream.read_u8()?)
}

pub fn read_u16(stream: &mut (impl Read+Write), address: u32) -> Result<u16> {
    send_read_command::<u16>(stream, address)?;
    Ok(stream.read_u16::<BigEndian>()?)
}

pub fn read_u32(stream: &mut (impl Read+Write), address: u32) -> Result<u32> {
    send_read_command::<u32>(stream, address)?;
    Ok(stream.read_u32::<BigEndian>()?)
}

fn send_read_command<T: PrimInt>(stream: &mut impl Write, address: u32) -> Result<()> {
    // format is
    // COMMAND_START
    // READ_COMMAND
    // <address:4>
    // <nbytes:1>
    //
    // result is one single read of size sizeof(T)
    let mut command = Vec::new();
    command.push(COMMAND_START);
    command.push(READ_COMMAND);
    let nbytes = std::mem::size_of::<T>() as u8;
    let mut addr_bytes = Vec::new();
    addr_bytes.write_be(address)?;
    println!("Reading {} bytes {}", addr_bytes.len(), nbytes);
    command.append(&mut addr_bytes);
    command.push(nbytes);
    command.push(COMMAND_END);
    stream.write(&command)?;
    Ok(())
}

pub fn write_value<T: ToBytes<N>, const N: usize>(
        stream: &mut impl Write, address: u32, value: T) -> Result<()> {
    // format is
    // COMMAND_START
    // WRITE_COMMAND
    // <address:4>
    // <nbytes:1>
    // <value:nbytes>
    // COMMAND_END
    let mut command = Vec::new();
    command.push(COMMAND_START);
    command.push(WRITE_COMMAND);
    let nbytes = std::mem::size_of::<T>() as u8;
    let mut addr_bytes = Vec::new();
    addr_bytes.write_be(address)?;
    let mut value_bytes = Vec::new();
    value_bytes.write_be(value)?;
    command.append(&mut addr_bytes);
    command.push(nbytes);
    command.append(&mut value_bytes);
    command.push(COMMAND_END);
    stream.write(&command)?;
    Ok(())
}

pub fn watch_value(stream: &mut impl Write, address: u32, dt: &DataType) -> Result<()> {
    // format is
    // COMMAND_START
    // WATCH_COMMAND
    // <address:4>
    // <nbytes:1>
    // <type: 1>
    // COMMAND_END
    let mut command = Vec::new();
    command.push(COMMAND_START);
    command.push(WATCH_COMMAND);
    let mut addr_bytes = Vec::new();
    addr_bytes.write_be(address)?;
    command.append(&mut addr_bytes);
    // depending on the data type, push that information
    match dt {
        DataType::U32 => {
            command.push(4_u8);
            command.push(TYPE_UNSIGNED);
        },
        DataType::U16 => {
            command.push(2_u8);
            command.push(TYPE_UNSIGNED);
        },
        DataType::U8 => {
            command.push(1_u8);
            command.push(TYPE_UNSIGNED);
        },
        DataType::I32 => {
            command.push(4_u8);
            command.push(TYPE_SIGNED);
        },
        DataType::I16 => {
            command.push(2_u8);
            command.push(TYPE_SIGNED);
        },
        DataType::I8 => {
            command.push(1_u8);
            command.push(TYPE_SIGNED);
        },
        DataType::F32 => {
            command.push(4_u8);
            command.push(TYPE_FLOAT);
        },
    }
    command.push(COMMAND_END);
    stream.write(&command)?;
    Ok(())
}

#[derive(PartialEq)]
pub enum DataType {
    U32,
    U16,
    U8,
    I32,
    I16,
    I8,
    F32,
}

impl Default for DataType {
    fn default() -> Self {
        Self::U32
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::U32 => write!(f, "u32"),
            DataType::U16 => write!(f, "u16"),
            DataType::U8 => write!(f, "u8"),
            DataType::I32 => write!(f, "i32"),
            DataType::I16 => write!(f, "i16"),
            DataType::I8 => write!(f, "i8"),
            DataType::F32 => write!(f, "f32"),
        }
    }
}

#[derive(Default)]
pub struct Watch {
    pub address: u32,
    pub nbytes: u8,
    pub value: u32,
    pub data_type: DataType,
}

pub fn view_watch_values(stream: &mut (impl Read+Write)) -> Result<Vec<Watch>> {
    // format is
    // COMMAND_START
    // VIEW_WATCH_COMMAND
    // <0:4>
    // COMMAND_END
    let mut command = Vec::new();
    command.push(COMMAND_START);
    command.push(VIEW_WATCH_COMMAND);
    command.push(0_u8);
    command.push(0_u8);
    command.push(0_u8);
    command.push(0_u8);
    command.push(COMMAND_END);
    stream.write(&command)?;

    // return structure is
    read_u8_expect(stream, COMMAND_START)?;
    read_u8_expect(stream, VIEW_WATCH_COMMAND)?;
    let entries = stream.read_u32::<BigEndian>()?;
    //println!("Found {} watches", entries);
    let mut ret = Vec::new();
    for _ in 0..entries {
        let address = stream.read_u32::<BigEndian>()?;
        let nbytes = stream.read_u8()?;
        let typ = stream.read_u8()?;
        let value;
        let mut data_type = DataType::U32;
        if nbytes == 1 {
            value = stream.read_u8()? as u32;
            if typ == TYPE_SIGNED {
                data_type = DataType::I8;
            } else if typ == TYPE_UNSIGNED {
                data_type = DataType::U8;
            }
        } else if nbytes == 2 {
            value = stream.read_u16::<BigEndian>()? as u32;
            if typ == TYPE_SIGNED {
                data_type = DataType::I16;
            } else if typ == TYPE_UNSIGNED {
                data_type = DataType::U16;
            }
        } else if nbytes == 4 {
            value = stream.read_u32::<BigEndian>()?;
            if typ == TYPE_SIGNED {
                data_type = DataType::I32;
            } else if typ == TYPE_UNSIGNED {
                data_type = DataType::U32;
            } else if typ == TYPE_FLOAT {
                data_type = DataType::F32;
            }
        } else {
            return Err(Error::InvalidByteSize(nbytes))?;
        }
        ret.push(Watch {
            address: address,
            nbytes: nbytes,
            value: value,
            data_type: data_type,
        });
    }
    read_u8_expect(stream, COMMAND_END)?;
    Ok(ret)
}

fn read_u8_expect(stream: &mut impl Read, expect: u8) -> Result<u8> {
    let x = stream.read_u8()?;
    if x != expect {
        return Err(Error::InvalidReturnByte(x, expect))?;
    }
    Ok(x)
}

pub fn unwatch_value(stream: &mut impl Write, address: u32) -> Result<()> {
    // format is
    // COMMAND_START
    // UNWATCH_COMMAND
    // <address:4>
    // COMMAND_END
    let mut command = Vec::new();
    command.push(COMMAND_START);
    command.push(UNWATCH_COMMAND);
    let mut addr_bytes = Vec::new();
    addr_bytes.write_be(address)?;
    println!("Unwatching {}", addr_bytes.len());
    command.append(&mut addr_bytes);
    command.push(COMMAND_END);
    stream.write(&command)?;
    Ok(())
}
