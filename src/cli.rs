use clap::{Parser, Args};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 监控间隔（秒）
    #[arg(short, long, default_value_t = 1)]
    pub interval: u64,

    #[command(flatten)]
    pub monitors: MonitorArgs,
}

#[derive(Args, Debug)]
pub struct MonitorArgs {
    /// 是否监控 CPU
    #[arg(long, default_value_t = true)]
    pub cpu: bool,

    /// 是否监控内存
    #[arg(long, default_value_t = true)]
    pub memory: bool,

    /// 是否监控磁盘
    #[arg(long, default_value_t = true)]
    pub disk: bool,

    /// 是否监控网络
    #[arg(long, default_value_t = true)]
    pub network: bool,
} 