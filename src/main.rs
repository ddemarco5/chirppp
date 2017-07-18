use std::env;

// For threading and channels
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

//use std::thread::sleep;
use std::time::{Duration, SystemTime};
//use std::fs::File;
//use std::io::prelude::*;

extern crate floating_duration;
use floating_duration::TimeAsFloat;

extern crate serial;

extern crate lora_driver;
use lora_driver::RadioConfig;
use lora_driver::Driver;
use lora_driver::RadioMode;

// Testing for Bryant
//extern crate serialport;


pub fn measure_rate_and_late(driver: &mut Driver, config: &mut RadioConfig, speed_str: &str, data_size: u32, loop_num: u32) -> (f64,f64) {

	let mut latency = Duration::new(0,0);

	let send_buf: Vec<u8> = vec![255;data_size as usize];
	//config.set_air_rate(speed_str);
	//driver.write_config(*config);

	println!("Running size {}",data_size);

	let start_time = SystemTime::now();

	for i in 0..loop_num {
		println!("Sending packet {}",i+1);
		let before_sending = SystemTime::now();
		driver.send_packet(&send_buf, 5000);
		// get a packet back
		let packet = driver.receive_packet(10_000);
		// Record the latency after getting the packet
		latency += before_sending.elapsed().unwrap();
		// print that packet we just got
		//println!("Recieved a {} byte packet",packet.len());
	}

	// TODO: get this working in floats to get an accurate number
	let end_duration = start_time.elapsed().unwrap();
	let seconds_it_took = end_duration.as_fractional_secs();
	let data_rate = (data_size * loop_num) as f64 / seconds_it_took;
	let latency = latency.as_fractional_millis() / loop_num as f64;

	(data_rate,latency)
}

pub fn async_receive(){
	let (tx, rx) = channel();
	// Spawn a thread that will
	let thread = thread::spawn(move || {
		tx.send("hello").unwrap();
	});
}


fn main() {

	// Get args that were passed
	let args: Vec<String> = env::args().collect();
	let platform = &args[1];
	let send_recv = &args[2];
	//let mut passed_string = String::new();
	//match args.len() {
	//	4 => {passed_string.push_str(args[3].as_str());},
	//	_ => {}
	//}

	let mut e32_driver;
	let mut testconfig = RadioConfig::new();

	match &platform[..] {
		"vocore" => { 
						e32_driver = Driver::new(22,23,24, "/dev/ttyS0");
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
	e32_driver.write_config(testconfig);
	

	let loop_num = 10;


	match &send_recv[..] {
		"r" => {
			// Receive mode
			let mut i = 0;
			loop {
				match e32_driver.receive_packet(10_000) {
					Ok(value) => { 
						i += 1;
						println!("Recieved packet {}", i);
						//println!("Recieved packet {}", String::from_utf8(value).unwrap());
						// Send it back
						//e32_driver.send_packet(&packet);
					},
					Err(_) => { println!("Timed out waiting for packet"); }
				}
				// send it right back
				//
				//println!("Responding with size {}",packet.len());
			}
		}
		"s" => {
			// Send mode

			for i in 0..100 {
				e32_driver.send_packet(Vec::from("hello").as_ref(), 5000).expect("Error sending packet");
				println!("Sent packet {}", i+1);
			}


			/*

			// Open a file to log our data
			let mut file = File::create("results.csv").unwrap();
			//file.write_all(b"Radio Speed(k), Observed speed(B/s), latency(ms)\n").unwrap();
			file.write_all(b"packet size(kB), Observed speed(B/s), latency(ms)\n").unwrap();

			
			// 1k 2k 5k 10k 12k 15k 20k 25k
			// A packet of 58 bytes
			//for s in ["1k","2k","5k","10k","12k","15k","20k","25k"].into_iter() {
			//for s in 1..29 {
				//let speed_lat = measure_rate_and_late(&mut e32_driver, &mut testconfig, s, 58, loop_num);
				//let datasize = s*2; // fun math
				//println!("Datasize: {}", datasize);
				let packet_size_int = packet_size.parse().unwrap();
				let speed_lat = measure_rate_and_late(&mut e32_driver, &mut testconfig, "10k", packet_size_int, loop_num);
				//println!("{}, {}B/s, {}ms",datasize,speed_lat.0,speed_lat.1);
				let mut string = String::new();
				string.push_str((packet_size_int).to_string().as_str()); string.push(',');
				string.push_str(speed_lat.0.to_string().as_str()); string.push(',');
				string.push_str(speed_lat.1.to_string().as_str()); string.push('\n');
				file.write_all(string.as_bytes()).unwrap();
				println!("{}",string);

				// Wait a second to allow the other radio to change modes
				//sleep(Duration::from_secs(5));
			//}
			file.flush().unwrap();
			*/
		}
		_ => {
			panic!("Call with either 'r' or 's'");	
		}
	}

}
