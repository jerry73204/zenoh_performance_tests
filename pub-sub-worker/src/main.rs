mod common;
mod utils;
mod workers;
use common::*;
use std::path::PathBuf;
use std::str::FromStr;
use utils::*;
use workers::*;

#[derive(Debug, StructOpt, Serialize, Deserialize, Clone)]
pub struct Cli {
    #[structopt(short = "o", long, default_value = "./", parse(from_os_str))]
    /// The path to store the output .json file.
    output_dir: PathBuf,
    #[structopt(short = "p", long)]
    /// The peer ID for this process
    peer_id: usize,
    #[structopt(short = "a", long)]
    /// The total number of publisher peers.
    /// If pub-sub-separate flag not used, this will be the total number of peers.
    total_put_number: usize,
    #[structopt(short = "t", long, default_value = "100")]
    /// The timeout for subscribers to stop receiving messages. Unit: milliseconds (ms).
    /// The subscriber will start receiving the messages at the same time as the publishers.
    round_timeout: u64,
    #[structopt(short = "i", long, default_value = "1000")]
    /// The initialization time (ms) for starting up futures.
    init_time: u64,
    #[structopt(short = "m", long, default_value = "1")]
    /// The number of messages each publisher peer will try to send.
    num_msgs_per_peer: usize,
    #[structopt(short = "n", long, default_value = "8")]
    /// The payload size (bytes) of the message.
    payload_size: usize,
    #[structopt(long)]
    /// Create multiple zenoh runtimes on a single machine or not for each peer.
    /// It will always be set to false if pub_sub_sep is not set, since the worker will generate a new zenoh instance for each pair of pub and sub worker.
    multipeer_mode: bool,
    #[structopt(long)]
    /// Create a zenoh runtime for a pair of pub/sub if not set.
    /// If this flag not set, the total number of peers is read from `num_put_peers`.
    pub_sub_separate: bool,
    #[structopt(short = "e", long)]
    /// Specifies locators for each peer to connect to (example format: tcp/x.x.x.x:7447).
    /// If you'd like to connect to several addresses, separate them with a comma (example: tcp/x.x.x.x:7447,tcp/y.y.y.y:7447)
    locators: Option<String>,
    #[structopt(short = "a", long, default_value = "0")]
    /// Number of remote subscriber peers.
    /// Used to notify subscribers to receive messages from remote peers.
    /// Note that the num_msgs_per_peer needs to be the same on both remote and local machines
    remote_pub_peers: usize,
}

#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    // Get & parse arguments
    let args = Cli::from_args();

    // Parameters
    let start = Instant::now();
    let process_start = datetime::Instant::now();
    let start_until = start + Duration::from_millis(args.init_time);
    let timeout = start_until + Duration::from_millis(args.round_timeout);

    // Start workers
    let pub_sub_fut = pub_and_sub_worker(
        start_until,
        timeout,
        args.peer_id,
        args.num_msgs_per_peer,
        get_msg_payload(args.payload_size, args.peer_id),
        (args.total_put_number + args.remote_pub_peers) * args.num_msgs_per_peer,
        args.locators.clone(),
        args.output_dir.clone(),
        args.total_put_number,
        args.payload_size,
        args.clone(),
        start,
        process_start,
    );
    futures::join!(pub_sub_fut);
}
