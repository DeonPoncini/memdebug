use std::net::TcpStream;

use anyhow::Result;

mod commands;

fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:3333")?;

    let b1 = commands::read_u32(&mut stream, 0x00ECC430)?;
    println!("Reading memory address 0x00ECC430");
    println!("0x00ECC430 is {}", b1);

    commands::write_value(&mut stream, 0x00ECC430, 2)?;
    println!("Setting memory address 0x00ECC430 to 2");

    let b1 = commands::read_u32(&mut stream, 0x00ECC430)?;
    println!("0x00ECC430 is {}", b1);

    Ok(())
}

