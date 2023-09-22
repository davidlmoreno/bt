pub mod threading {
    use crate::data_structures::data_structures::{BlockHeaderData, Message};
    use crate::statistics::computation::compute_histogram_and_stats;
    use std::collections::HashMap;
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread::JoinHandle;
    use std::time::Instant;

    pub fn create_thread_pool(
        data: &Vec<BlockHeaderData>,
        threads: i32,
        nonce_stats: &HashMap<usize, f32>,
        samples_threshold: &usize,
        info_threshold: &f32,
    ) -> (Vec<JoinHandle<()>>, Vec<Sender<Message>>, Receiver<Message>) {
        let mut th_handles = vec![];
        let mut th_senders = vec![];
        let (main_tx, main_rx) = mpsc::channel();

        for _ in 0..(threads - 1) {
            let (tx, rx) = mpsc::channel();
            let data_clone = data.clone();
            let main_transmitter = main_tx.clone();
            let my_transmitter = tx.clone();
            let nonce_stats_clone = nonce_stats.clone();
            let info_thr_clone = info_threshold.clone();
            let sample_thr_clone = samples_threshold.clone();

            th_senders.push(tx.clone());
            th_handles.push(std::thread::spawn(move || {
                worker(
                    data_clone,
                    rx,
                    my_transmitter,
                    main_transmitter,
                    nonce_stats_clone,
                    info_thr_clone,
                    sample_thr_clone,
                )
            }));
            main_tx
                .send(Message::Free(tx.clone()))
                .expect("Error sending message");
        }
        (th_handles, th_senders, main_rx)
    }

    pub fn worker(
        data: Vec<BlockHeaderData>,
        rx: Receiver<Message>,
        tx: Sender<Message>,
        main_tx: Sender<Message>,
        nonce_stats: HashMap<usize, f32>,
        info_threshold: f32,
        sample_threshold: usize,
    ) {
        for received in rx {
            match received {
                Message::Process(vector) => {
                    let length = vector.len();
                    println!("[worker] Received {} entries.", length);
                    let now = Instant::now();
                    compute_histogram_and_stats(
                        &data,
                        vector,
                        &nonce_stats,
                        &info_threshold,
                        &sample_threshold,
                    );
                    let secs = now.elapsed().as_secs();
                    let rate = (length as f32) / (secs as f32);
                    println!(
                        "[worker] Finished working, sending message for more work. Took {} secs, rate: {} items/s",
                        secs, rate
                    );
                    main_tx
                        .send(Message::Free(tx.clone()))
                        .expect("Error sending message");
                }
                Message::Stop => {
                    println!("[worker] Received stop message, stopping");
                    break;
                }
                _ => {}
            }
        }
    }
}

pub mod computation {
    use crate::data_structures::data_structures::{
        BlockHeaderData, DataAddress, DataInstance, Statistic,
    };
    use crate::file::file_export::save_stats_to_file;
    use std::collections::HashMap;

    pub fn compute_histogram_and_stats(
        data: &Vec<BlockHeaderData>,
        stats: Vec<Statistic>,
        nonce_stats: &HashMap<usize, f32>,
        info_threshold: &f32,
        sample_threshold: &usize,
    ) {
        let mut final_stats = vec![];
        for mut s in stats {
            for entry in data {
                let mut header_value: u32 = 0;
                let h_bits: &Vec<u16> = &s.address.header_bits;
                let test_vec: Vec<bool> = h_bits
                    .into_iter()
                    .map(|x| entry.header[*x as usize])
                    .collect();
                for (i, val) in test_vec.iter().rev().enumerate() {
                    header_value += (*val as u32) * 2u32.pow(i as u32);
                }

                //println!("Header bits: {:?}, test_vec {:?} => Val: {}", &s.address.header_bits, &test_vec, &header_value);

                let nonce_value: u32 = entry.nonce[s.address.nonce_bit as usize] as u32;
                let data_stats: &mut DataInstance = s.instances.get_mut(&header_value).unwrap();
                if nonce_value > 0 {
                    data_stats.ones += 1;
                } else {
                    data_stats.zeros += 1;
                }
            }
            let nonce_as_usize: usize = s.address.nonce_bit as usize;
            let nonce_bit_entropy = nonce_stats
                .get(&nonce_as_usize)
                .expect("Nonce bit not found");
            update_statistics(&mut s.instances, nonce_bit_entropy);

            let mut del_keys = vec![];
            for (key, instance) in &s.instances {
                if !passes_thresholds(&instance, &sample_threshold, &info_threshold) {
                    //		  s.instances.remove(&key);
                    del_keys.push(key.clone());
                }
            }

            for x in del_keys {
                s.instances.remove(&x);
            }

            if s.instances.len() > 0 {
                final_stats.push(s);
            }
        }
        // If something is there, dump it
        if final_stats.len() > 0 {
            save_stats_to_file(final_stats);
        }
    }

    fn passes_thresholds(value: &DataInstance, sample_thr: &usize, info_thr: &f32) -> bool {
        let mut result = false;
        if value.information > *info_thr && value.total > *sample_thr as u32 {
            result = true;
        }
        result
    }

    pub fn account_address(stack: &Vec<u16>, nonce_bit: u8, stats: &mut Vec<Statistic>) {
        let mut address_stat: Statistic = Statistic {
            address: DataAddress {
                header_bits: stack.to_vec().clone(),
                nonce_bit,
            },
            instances: HashMap::new(),
        };
        let max: u32 = 2u32.pow(address_stat.address.header_bits.len() as u32);
        for k in 0..max {
            let new_data_instance: DataInstance = DataInstance {
                zeros: 0,
                ones: 0,
                total: 0,
                p_zero: 0.0,
                p_one: 0.0,
                entropy: 0.0,
                information: 0.0,
            };
            address_stat.instances.insert(k, new_data_instance);
        }

        stats.push(address_stat);
    }

    fn update_statistics(values: &mut HashMap<u32, DataInstance>, nonce_bit_entropy: &f32) {
        for (_, entry) in values.iter_mut() {
            entry.total = entry.ones + entry.zeros;
            if entry.total > 0 {
                entry.p_zero = entry.zeros as f32 / entry.total as f32;
                entry.p_one = entry.ones as f32 / entry.total as f32;
            }
            // Compute entropy and information
            let mut entropy_xy: f32 = 0.0;

            if entry.p_zero > 0.0 {
                entropy_xy = entropy_xy + entry.p_zero * entry.p_zero.log2();
            }
            if entry.p_one > 0.0 {
                entropy_xy = entropy_xy + entry.p_one * entry.p_one.log2();
            }
            entropy_xy = (-1.0) * entropy_xy;
            entry.entropy = entropy_xy;
            entry.information = nonce_bit_entropy - entropy_xy;
        }
    }
}
