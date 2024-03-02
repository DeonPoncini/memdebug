use std::net::TcpStream;

use anyhow::Result;

mod commands;
mod error;

fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:3333")?;

    let b1 = commands::read_u32(&mut stream, 0x00ECC430)?;
    println!("Reading memory address 0x00ECC430");
    println!("0x00ECC430 is {}", b1);

    commands::write_value(&mut stream, 0x00ECC430, 2_u32)?;
    println!("Setting memory address 0x00ECC430 to 2");

    let b1 = commands::read_u32(&mut stream, 0x00ECC430)?;
    println!("0x00ECC430 is {}", b1);

    commands::watch_value::<u32, 4>(&mut stream, 0x00ECC430)?;
    let watches = commands::view_watch_values(&mut stream)?;
    for watch in watches {
        println!("Watching 0x{:x} value: {} ({} bytes)", watch.address, watch.value, watch.nbytes);
    }
    commands::unwatch_value(&mut stream, 0x00ECC430)?;
    let watches = commands::view_watch_values(&mut stream)?;
    for watch in watches {
        println!("Watching 0x{:x} value: {} ({} bytes)", watch.address, watch.value, watch.nbytes);
    }
    Ok(())
}

