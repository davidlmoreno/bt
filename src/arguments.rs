pub mod arguments {
    use std::process::exit;

    pub fn parse_config(args: &[String]) -> (u16, usize, String, String, u16, u16, usize, f32) {
        if args.len() < 8 {
            show_usage();
            exit(0);
        }

        let tuple_size: u16 = (args[1]).parse::<u16>().unwrap();
        let slice_size: usize = (args[2]).parse::<usize>().unwrap();
        let filename: &String = &args[3];
        let nonce_filename: &String = &args[4];
        let bit_start: u16 = (args[5]).parse::<u16>().unwrap();
        let bit_end: u16 = (args[6]).parse::<u16>().unwrap();
        let samples_threshold: usize = (args[7]).parse::<usize>().unwrap();
        let info_threshold: f32 = (args[8]).parse::<f32>().unwrap();
        (
            tuple_size,
            slice_size,
            filename.to_string(),
            nonce_filename.to_string(),
            bit_start,
            bit_end,
            samples_threshold,
            info_threshold,
        )
    }

    fn show_usage() {
        println!("\nArgs: <tuple_size> <slice_size> <hashes file> <nonce_probs file> \
         <start bit> <end bit> <sample threshold> <information threshold>");
         println!("\n<tuple_size>: Number of bits in the block used to correlate with each nonce bit\n\
                   <slice_size>: Batch of tuples to give to each worker\n\
                   <hashes file>: file containing the block and hashes\n\
                   <nonce_probs file>: file containing the probability of each nonce bit being 1 or 0\n\
                   <start bit>: Starting bit for the first bit in the pair. The program pairs that bit with the remaining ones to create the tuples. E.g. starting in bit 10 means that the first 2-bit tuple to correlate will be [10,11], then [10,12], etc.\n\
                   <end bit>: Last initial bit for the tuple creation. E.g. if the last bit is 12, the last 2-bit tuple will be [12,429]\n\
                   <sample threshold>: minimum number of samples from which the entropy/information pair is calculated: too little samples provide no significance to the entropy/information.\n\
                   <information threshold>: minimum level of information to end up in the file.\n");

    }
}
