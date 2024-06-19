use std::thread;
use std::time::Duration;

fn main() {
    loop {
        // allocate memory from 0 to 200MB
        for size in (0..=200).step_by(10) {
            let number_of_bytes = size * 1024 * 2014;
            let mut vec: Vec<u8> = Vec::with_capacity(number_of_bytes);

            for i in 0..number_of_bytes {
                vec.push(i as u8);
            }
            println!{"Current memory usage: {}MB", size};
            thread::sleep(Duration::from_secs(1));
        }

        // deallocate memory from 200 to 0MB
        for size in (0..=200).rev().step_by(10) {
            let number_of_bytes = size * 1024 * 2014;
            let mut vec: Vec<u8> = Vec::with_capacity(number_of_bytes);

            for i in 0..number_of_bytes {
                vec.push(i as u8);
            }
            println!{"Current memory usage: {}MB", size};
            thread::sleep(Duration::from_secs(1));
        }

    }
}
