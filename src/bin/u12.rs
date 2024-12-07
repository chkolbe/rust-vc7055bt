use std::time::Duration;

use rusb::{open_device_with_vid_pid, TransferType, Direction};

fn main() {

    assert_eq!(rusb::devices().unwrap().is_empty(), false, "LibUSB no Device found!");

    // Find specific Device and Open it.
    let dh = open_device_with_vid_pid(0x0cd5, 0x0001);

    if dh.is_none() {
        assert!(false, "Labjack U12 not found!")
    }

    let mut dh = dh.unwrap();

    dh.reset().unwrap();

    dh.set_auto_detach_kernel_driver(true).unwrap();

    match dh.kernel_driver_active(0u8) {
        Ok(active) => println!("Kernel Driver active: {}", active),
        Err(err) => panic!("Error checking Kernel Driver: {}", err),
    }

    // Device Configuration 0 must be used
    dh.set_active_configuration(0u8).unwrap();
    assert_eq!(dh.active_configuration().unwrap(), 0u8, "U12 USB Configuration Active is not cfg #0!");

    dh.claim_interface(0u8).unwrap();

    // Read all Endpoints to find read and write Interrupt Endpoints
    let config_descriptor = dh.device().config_descriptor(0u8).unwrap();
    assert_eq!(config_descriptor.num_interfaces(), 1u8, "U12 USB Device offers more as 1 Interface!");

    // Give us back the first Interface
    let interface = config_descriptor.interfaces().next().unwrap();
    let interface_descriptor = interface.descriptors().next().unwrap();

    assert_eq!(interface_descriptor.interface_number(), 0, "U12 Interface Number is not 0!");
    assert_eq!(interface_descriptor.endpoint_descriptors().count(), 2, "U12 Interface offers more as 2 Endpoints!");

    // Get all Endpoints with Support of Interrupt Read and Write
    let epd_interrupts = interface_descriptor.endpoint_descriptors().filter(|epd| epd.transfer_type() == TransferType::Interrupt);

    assert_eq!(epd_interrupts.count(), 2, "U12 USB Interface offers more as 2 Interrupt Endpoints!");

    let mut epd_interrupts = interface_descriptor.endpoint_descriptors().filter(|epd| epd.transfer_type() == TransferType::Interrupt);
    assert_eq!(epd_interrupts.next().unwrap().address(), 0x81, "U12 Interface #0 EP #0 Address is not 0x81!");
    assert_eq!(epd_interrupts.next().unwrap().address(), 0x02, "U12 Interface #0 EP #0 Address is not 0x02!");

    let mut epd_interrupts = interface_descriptor.endpoint_descriptors().filter(|epd| epd.transfer_type() == TransferType::Interrupt);
    assert_eq!(epd_interrupts.next().unwrap().direction(), Direction::In, "U12 Interface #0 EP Address 0x81 is not Readable!");
    assert_eq!(epd_interrupts.next().unwrap().direction(), Direction::Out, "U12 Interface #0 EP Address 0x02 is not Writeable!");

    // Init Read Procedure somehow needed for the U12 to operate.
    match dh.write_interrupt(0x02u8, &[0x0, 0x0, 0x0, 0x0, 0x0, 0x57, 0x0, 0x0], Duration::from_millis(20)) {
        Ok(len) => {
            assert_eq!(len, 8, "Not all Bytes written to Device!");
        }
        Err(err) => panic!("could not write to endpoint: {}", err),
    }
    // This Read will fail every single time
    let mut recv_buffer = [0u8; 8];
    match dh.read_interrupt(0x81u8, &mut recv_buffer, Duration::from_millis(100)) {
        Ok(len) => {
            assert_eq!(len, 8, "Not all Bytes read from Device!");
        }
        Err(err) => panic!("could not read from endpoint: {}", err),
    }

    // Disable the Watchdog and Read the Firmware Version
    match dh.write_interrupt(0x02u8, &[0x1, 0x0, 0x0, 0x0, 0x0, 0x53, 0x0, 0x0], Duration::from_millis(20)) {
        Ok(len) => {
            assert_eq!(len, 8, "Not all Bytes written to Device!");
        }
        Err(err) => panic!("could not write to endpoint: {}", err),
    }
    let mut recv_buffer = [0u8; 8];
    match dh.read_interrupt(0x81u8, &mut recv_buffer, Duration::from_millis(100)) {
        Ok(len) => {
            assert_eq!(len, 8, "Not all Bytes read from Device!");
            assert_eq!(recv_buffer[0], 1, "Firmware Major Version is not 1!");
            assert_eq!(recv_buffer[1], 10, "Firmware Minor Version is not 10!")
        }
        Err(err) => panic!("could not read from endpoint: {}", err),
    }
}