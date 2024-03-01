use std::io::prelude::*;
use std::net::TcpStream;

use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};

// delimiter
const COMMAND_START: u8 = 0xFF;
const COMMAND_END: u8 = 0xFE;

// commands
const READ_COMMAND: u8 = 0x01;
const WRITE_COMMAND: u8 = 0x02;
const WATCH_COMMAND: u8 = 0x03;
const VIEW_WATCH_COMMAND: u8 = 0x04;
const UNWATCH_COMMAND: u8 = 0x05;

fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:3333")?;

    let command = [COMMAND_START, READ_COMMAND, 0x00, 0xEC, 0xC4, 0x30, 0x04, COMMAND_END];
    stream.write(&command)?;
    println!("Reading memory address 0x00ECC430");
    let b1 = stream.read_u32::<BigEndian>()?;
    println!("0x00ECC430 is {}", b1);

    let command = [COMMAND_START, WRITE_COMMAND, 0x00, 0xEC, 0xC4, 0x30, 0x04, 0x00, 0x00, 0x00, 0x02, COMMAND_END];
    println!("Setting memory address 0x00ECC430 to 2");
    stream.write(&command)?;

    let command = [COMMAND_START, READ_COMMAND, 0x00, 0xEC, 0xC4, 0x30, 0x04, COMMAND_END];
    stream.write(&command)?;
    let b1 = stream.read_u32::<BigEndian>()?;
    println!("0x00ECC430 is {}", b1);

    Ok(())
}

