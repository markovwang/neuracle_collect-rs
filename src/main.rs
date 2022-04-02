use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::thread;
use kafka::producer;
use kafka::producer::Record;
use neuracle_collect::{Config, down_sample, read_data, Strategy};

fn main() {
    let config = Config::from_file("Neuracle.toml");
    let topic = config.topic.clone();
    let mut producer = producer::Producer::from_hosts(vec![config.server.clone()]).create().unwrap();
    let (sender, receiver) = channel::<Vec<f32>>();
    let mut client = TcpStream::connect(&config.neuracle_addr).unwrap();
    let receive_thread = thread::spawn(move || {
        println!("start to fetch data");
        loop {
            let eeg_pack = read_data(&mut client, &config);
            let eeg_pack = down_sample(eeg_pack, &config);
            match config.strategy {
                Strategy::COL => {
                    for col in eeg_pack.column_iter() {
                        sender.send(col.as_slice().to_vec()).unwrap();
                    }
                }
                Strategy::PACK => {
                    sender.send(eeg_pack.as_slice().to_vec()).unwrap()
                }
            }
        }
    });
    let forward_thread = thread::spawn(move || {
        loop {
            let send_data = receiver.recv().unwrap();
            let mut data_bytes = Vec::<u8>::new();
            for item in send_data.into_iter() {
                data_bytes.append(&mut item.to_le_bytes().to_vec());
            }
            producer
                .send(&Record {
                    topic: topic.as_str(),
                    partition: 0,
                    key: (),
                    value: data_bytes,
                })
                .unwrap();
        }
    });
    receive_thread.join().unwrap();
    forward_thread.join().unwrap();
}