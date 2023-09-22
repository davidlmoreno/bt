mod arguments;
mod data_structures;
mod file;
mod statistics;

use std::env;
use std::sync::mpsc::{Receiver, Sender};

use crate::arguments::arguments::parse_config;
use crate::data_structures::data_structures::{BlockHeaderData, DataAddress, Message, Statistic};
use crate::file::file_import::{import_headers_file, import_nonce_stats_file};
use crate::statistics::computation::account_address;
use crate::statistics::threading::create_thread_pool;
use std::process::exit;

#[macro_use]
extern crate lazy_static;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (
        tuple_size,
        slice_size,
        filename,
        nonce_filename,
        start_bit,
        end_bit,
        sample_threshold,
        info_threshold,
    ) = parse_config(&args);

    let data: Vec<BlockHeaderData> = import_headers_file(filename);
    let nonce_stats = import_nonce_stats_file(nonce_filename);

    // Vector to store the address as we go deeper in the recursion
    let mut stack: Vec<u16> = Vec::new();
    // Vector for storing the statistic of each address
    let mut stats: Vec<Statistic> = Vec::new();

    // Create the thread pool
    //let threads = 6;
    let threads = 14;

    let (th_handles, th_senders, main_rx) = create_thread_pool(
        &data,
        threads,
        &nonce_stats,
        &sample_threshold,
        &info_threshold,
    );

    // End bit is the lowest between input size and provided max bit
    let mut final_bit = data[0].header.len() as u16;
    if end_bit < final_bit {
        final_bit = end_bit;
    }

    // Recursive processing of all possibilities
    // check all possible tuples of tuple_size in the header
    // against each bit in the nonce. Extracts statistical data
    // on each tuple and stores in the stats vector
    unfold(
        start_bit,
        final_bit,
        data[0].header.len() as u16,
        data[0].nonce.len() as u8,
        tuple_size,
        &mut stack,
        &mut stats,
        &main_rx,
        &slice_size,
    );

    println!("Unfold finished!");

    for thread in th_senders {
        thread.send(Message::Stop).expect("Error sending message");
    }

    // Wait for all threads to end
    for handle in th_handles {
        handle.join().unwrap();
    }

    exit(0);
}

fn unfold(
    curr: u16,
    max: u16,
    s: u16,
    nonce_len: u8,
    n: u16,
    stack: &mut Vec<u16>,
    stats: &mut Vec<Statistic>,
    main_rx: &Receiver<Message>,
    slice_size: &usize,
) {
    if stats.len() > *slice_size {
        //println!("[Main] Waiting for worker to be free...");
        let worker = main_rx.recv().unwrap();
        //println!("[Main] Worker called! Using it.");
        match worker {
            Message::Free(channel) => {
                send_data_to_worker(stats, channel);
            }
            _ => {}
        }
    }

    return match n {
        0 => {
            for n in 0..nonce_len {
                account_address(stack, n, stats);
            }
        }
        _ => {
            for p in curr..s {
                // limit the first number range
                if stack.len() == 0 && p > max {
                    println!("Finished Fold on bit {}", &p);
                    return;
                }

                stack.push(p);
                unfold(
                    p + 1,
                    max,
                    s,
                    nonce_len,
                    n - 1,
                    stack,
                    stats,
                    main_rx,
                    slice_size,
                );
                stack.pop();
            }
        }
    };
}

fn send_data_to_worker(stats: &mut Vec<Statistic>, channel: Sender<Message>) {
    let result = channel.send(Message::Process(stats.clone()));
    match result {
        Ok(_) => {
            stats.resize(
                0,
                Statistic {
                    address: DataAddress {
                        header_bits: vec![],
                        nonce_bit: 0,
                    },
                    instances: Default::default(),
                },
            );
        }
        _ => {}
    }
}
