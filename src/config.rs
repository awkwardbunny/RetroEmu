use clap::{Parser, Subcommand, ValueEnum};
use log::LevelFilter;
use std::fmt::Debug;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = "Retro Computing Emulator")]
struct Cli {
    #[arg(long, value_name = "LOGFILE", default_value = "retro.log")]
    logfile: String,

    #[arg(long, value_name = "LOGLEVEL", default_value = "info")]
    loglevel: MyLogLevel,

    #[command(subcommand)]
    machine: Machines,
}

#[derive(ValueEnum, Debug, Clone, Default)]
enum MyLogLevel {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Machines {
    AppleIiE {
        #[arg(long, value_name = "FREQ_KHZ", default_value_t = 1020)]
        freq_khz: usize,

        #[arg(long, value_name = "DISK1")]
        disk1: Option<PathBuf>,

        #[arg(long, value_name = "DISK2")]
        disk2: Option<PathBuf>,
    },
}

// #[derive(Debug)]
pub struct Config {
    prefix: PathBuf,
    pub machine: Machines,
}

impl Config {
    pub fn load() -> Self {
        let retro_path = env::var("RETRO_PATH").unwrap_or(String::from("~/.retro"));
        let prefix = shellexpand::tilde(retro_path.as_str()).to_string();

        let cli = Cli::parse();
        let config = Self {
            prefix: PathBuf::from(prefix),
            machine: cli.machine,
        };

        let loglevel = match cli.loglevel {
            MyLogLevel::Off => LevelFilter::Off,
            MyLogLevel::Error => LevelFilter::Error,
            MyLogLevel::Warn => LevelFilter::Warn,
            MyLogLevel::Info => LevelFilter::Info,
            MyLogLevel::Debug => LevelFilter::Debug,
            MyLogLevel::Trace => LevelFilter::Trace,
        };

        let logfile = Box::new(
            config
                .create_file(cli.logfile)
                .expect("Unabled to open log file"),
        );
        env_logger::builder()
            .filter_level(loglevel)
            .filter(Some("wgpu"), LevelFilter::Off)
            .filter(Some("naga"), LevelFilter::Off)
            .target(env_logger::Target::Pipe(logfile))
            .init();

        config
    }

    fn get_full_path<T: AsRef<Path>>(&self, path: T) -> PathBuf {
        self.prefix.join(path)
    }

    #[allow(dead_code)]
    pub fn get_file_string<T: AsRef<Path>>(&self, p: T) -> io::Result<String> {
        let path = self.get_full_path(p);
        fs::read_to_string(path)
    }

    #[allow(dead_code)]
    pub fn get_file_bytes<T: AsRef<Path>>(&self, p: T) -> io::Result<Vec<u8>> {
        let path = self.get_full_path(p);
        fs::read(path)
    }

    pub fn create_file<T: AsRef<Path>>(&self, p: T) -> io::Result<File> {
        let path = self.get_full_path(p);
        File::create(path)
    }

    pub fn get_file<T: AsRef<Path>>(&self, p: T) -> io::Result<File> {
        let path = self.get_full_path(p);
        File::open(path)
    }
}
