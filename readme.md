# NeuracleCollect-rs

一款使用Rust语言编写的博睿康放大器脑电采集软件

## features:
1. 性能好, 占用内存低( 5800H 上占用 1.6% CPU, 2.1MB 内存)
2. 便携性拉满, 无需在计算机上安装各种不知道是什么的库
3. 可以通过配置按照列/包为单位进行数据的转发
4. 我再想想
5. 凑个整

## 配置文件格式

```toml
[collection]
neuracle_addr = "127.0.0.1:8712"    # ip::port
channel_size = 9                    # 导联数(包含Trigger导)
sample_rate = 1000                  # 设备采样率
time_buffer = 0.12                  # 数据包累计量

[forwarding]
bootstrap_server = "server:60000"   # kafka地址请去问李哥捏 :)
topic = "markov_test"               # 接收端的topic
strategy = "col"                    # col: 按列转发, pack: 按包转发

[eeg]
ds_rate = 4                         # 降采样率
```