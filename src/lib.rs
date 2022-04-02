use std::net::TcpStream;
use std::fs;
use std::io::Read;
use nalgebra::DMatrix;
use toml::Value;

#[derive(Debug)]
pub struct Config {
    pub neuracle_addr: String,
    pub channel: usize,
    pub sample_rate: usize,
    pub time_buffer: f64,
    pub server: String,
    pub ds_rate: u32,
    pub strategy: Strategy,
    pub topic: String,
}

#[derive(Debug)]
pub enum Strategy {
    COL,
    // 按照列发送
    PACK,   // 按照包发送
}

impl Config {
    pub fn from_file(file_path: &str) -> Self {
        let config = fs::read_to_string(file_path).expect("cannot read file");
        let config = config.parse::<Value>().unwrap();
        let str = config["forwarding"]["strategy"].as_str().unwrap().to_string();
        Self {
            neuracle_addr: String::from(config["collection"]["neuracle_addr"].as_str().unwrap()),
            channel: config["collection"]["channel_size"].as_integer().unwrap() as usize,
            sample_rate: config["collection"]["sample_rate"].as_integer().unwrap() as usize,
            time_buffer: config["collection"]["time_buffer"].as_float().unwrap(),
            server: String::from(config["forwarding"]["bootstrap_server"].as_str().unwrap()),
            ds_rate: config["eeg"]["ds_rate"].as_integer().unwrap() as u32,
            strategy: match str.as_str() {
                "col" => Strategy::COL,
                "pack" => Strategy::PACK,
                _ => panic!("config wrong, should be col or pack")
            },
            topic: String::from(config["forwarding"]["topic"].as_str().unwrap()),
        }
    }
}

pub fn read_data(client: &mut TcpStream, config: &Config) -> DMatrix<f32> {
    let buf_size: usize = (config.channel as f64 * config.sample_rate as f64 * config.time_buffer * 4 as f64) as usize;
    let mut bytes_vec = Vec::new();
    bytes_vec.resize(buf_size, 0u8);
    client.read(&mut bytes_vec).unwrap();
    let eeg_size = buf_size / 4;
    let mut data_vec = Vec::new();
    data_vec.resize(eeg_size, 0f32);
    for ind in 0..data_vec.len() {
        data_vec[ind] = f32::from_le_bytes(bytes_vec[ind * 4..ind * 4 + 4].try_into().unwrap());
    }
    DMatrix::<f32>::from_vec(eeg_size / config.channel, config.channel, data_vec).transpose()
}

pub fn down_sample(eeg_data: DMatrix<f32>, config: &Config) -> DMatrix<f32> {
    let downsample_rate = config.ds_rate as usize;
    let origin_size = eeg_data.shape();
    let trigger = eeg_data.row(eeg_data.shape().0 - 1);
    let trigger_num = trigger.iter().filter(|&val| *val != 0f32).count();
    let mut ret_vec = Vec::<f32>::new();
    for item in eeg_data.column_iter().step_by(downsample_rate) {
        ret_vec.append(&mut item.as_slice().to_vec());
    }
    let mut ret_mat = DMatrix::from_vec(origin_size.0, origin_size.1 / downsample_rate, ret_vec);
    return match trigger_num {
        0 => {
            ret_mat
        }
        _ => {
            let trigger_ind = trigger.iter()
                .enumerate()
                .filter_map(|(index, &value)| (value > 0f32).then(|| index)).collect::<Vec<_>>();
            for ind in trigger_ind {
                ret_mat[(origin_size.0 - 1, ind / downsample_rate)] = trigger[ind];
            }
            ret_mat
        }
    };
}