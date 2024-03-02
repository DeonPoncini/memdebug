use std::io::{Read, Write};

use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use eio::{ReadExt, WriteExt, ToBytes};
use num_traits::PrimInt;
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
