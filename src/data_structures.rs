pub mod data_structures {
    use json::JsonValue;
    use std::collections::HashMap;
    use std::sync::mpsc::Sender;

    #[derive(Debug, Clone)]
    pub struct BlockHeaderData {
        pub nonce: Vec<bool>,
        pub header: Vec<bool>,
    }

    #[derive(Debug, Clone)]
    pub struct DataAddress {
        pub header_bits: Vec<u16>,
        pub nonce_bit: u8,
    }

    impl From<DataAddress> for JsonValue {
        fn from(d: DataAddress) -> Self {
            json::object! {
                header: d.header_bits,
                nonce_bit: d.nonce_bit,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Statistic {
        pub address: DataAddress,
        pub instances: HashMap<u32, DataInstance>,
    }

    impl From<Statistic> for JsonValue {
        fn from(s: Statistic) -> Self {
            let mut data = json::JsonValue::new_array();
            for (k, data_instance) in s.instances {
                data.push(json::object! { key: k, instance: data_instance})
                    .expect("Error inserting JSON");
            }

            let val = json::object! {
                address: s.address,
                instances: data,
            };
            val
        }
    }

    #[derive(Debug, Clone)]
    pub struct DataInstance {
        pub zeros: u32,
        pub ones: u32,
        pub total: u32,
        pub p_zero: f32,
        pub p_one: f32,
        pub entropy: f32,
        pub information: f32,
    }

    impl From<DataInstance> for JsonValue {
        fn from(i: DataInstance) -> Self {
            json::object! {
                zeros: i.zeros,
                ones: i.ones,
                total: i.total,
                p_zero: i.p_zero,
                p_one: i.p_one,
                entropy: i.entropy,
                information: i.information,
            }
        }
    }

    #[derive(Debug)]
    pub enum Message {
        Process(Vec<Statistic>),
        Stop,
        Free(Sender<Message>),
    }
}
