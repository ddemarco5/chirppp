use std::env;

extern crate serial;

extern crate lora_driver;
use lora_driver::RadioConfig;
use lora_driver::Driver;
use lora_driver::RadioMode;


fn main() {

	// Get args that were passed
	let args: Vec<String> = env::args().collect();
	let platform = &args[1];
	let send_recv = &args[2];
	let mut passed_string = String::new();
	match args.len() {
		4 => {passed_string.push_str(args[3].as_str());},
		_ => {}
	}

	let mut e32_driver;
	let mut testconfig = RadioConfig::new();

	match &platform[..] {
		"vocore" => { 
						e32_driver = Driver::new(23,24,26, "/dev/ttyS0");
						//set our serial rate to 57600 as well
						e32_driver.set_tty_params(
							serial::Baud115200,
							serial::Bits8,
							serial::ParityNone,
							serial::Stop1,
							serial::FlowNone
						);
						e32_driver.set_tty_baud(serial::Baud57600);
						testconfig.set_serial_rate("57600");
					}
		"chip" => { 
						e32_driver = Driver::new(1013,1015,1017, "/dev/ttyS0");
						e32_driver.set_tty_params(
							serial::Baud115200,
							serial::Bits8,
							serial::ParityNone,
							serial::Stop1,
							serial::FlowNone
						);
						testconfig.set_serial_rate("115200");
				  }
		_ => panic!("Please enter either 'vocore' or 'chip'")
	}
	// we want it to be a mutable driver because we will be changing fields as we go.
	//let mut e32_driver = Driver::new(1013,1015,1017, "/dev/ttyS0"); //Pins for C.H.I.P.
	//let mut e32_driver = Driver::new(23,24,26, "/dev/ttyS0"); //Pins for Vocore.
	// Bring up our serial device with defaults
	

	testconfig.set_transmit_power("20dBm");
	testconfig.set_air_rate("1k");
	testconfig.set_address("FFFF"); // Channel 0
	e32_driver.set_mode(RadioMode::General);
	

	//let (m0_pin, m1_pin, aux_pin) = e32_driver.get_control_gpio_pins();
	//println!("M0: {} M1: {} AUX: {}", m0_pin, m1_pin, aux_pin);


	match &send_recv[..] {
		"r" => {
			// Recieve mode
			e32_driver.write_config(testconfig); // Write after change
			//let mut buffer: Vec<u8> = vec![0;256]; //initialize buffer of zeros
			println!("Waiting to receive from the radio");
			let packet = e32_driver.receive_packet();
			println!("Received a packet {} bytes long", packet.len());
			println!("{}",String::from_utf8(packet).unwrap());
			//println!("{:?}",packet);
		}
		"s" => {
			// Send mode
			e32_driver.write_config(testconfig); // Write after change
			//let send_buf: Vec<u8> = vec![0;256];
			let send_string = passed_string.clone();
			//let send_string = String::from("This is a test of how long a message can get. I should be able to keep typing and the entire packet should be sent.");
			let send_buf = send_string.into_bytes();
			//let send_buf: Vec<u8> = vec![255;116];
			e32_driver.send_packet(&send_buf);
		}
		_ => {
			panic!("Call with either 'r' or 's'");	
		}
	}

	/*
	//Send some data
	let mut packet: Vec<u8> = vec![0; 10]; // create a vector of 256 bytes
	e32_driver.send_packet(&packet);
	e32_driver.receive_packet(&mut packet);
	*/

}
