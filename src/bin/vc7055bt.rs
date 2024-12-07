extern crate serial;

use std::io;
use std::time::Duration;

use serial::prelude::*;

fn main() {
        let mut port = serial::open("/dev/cu.usbserial-D309PLP8").unwrap();
        interact(&mut port).unwrap();
}

fn interact<T: SerialPort>(port: &mut T) -> io::Result<()> {
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud9600)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })?;

    port.set_timeout(Duration::from_millis(1000))?;

    let mut buf: Vec<u8> = (0..255).collect();
    let cmd: &[u8] = "*IDN\n".as_bytes();

    port.write(cmd)?;
    port.read(&mut buf[..])?;

    Ok(())
}