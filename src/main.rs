#![allow(unused)]

mod common;
mod workers;
use common::*;
use workers::*;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "p", long, default_value = "1000")]
    /// The total number of publisher peers
    num_put_peer: usize,
    #[structopt(short = "s", long, default_value = "10")]
    /// The total number of subscriber peers
    num_sub_peer: usize,
    #[structopt(short = "t", long, default_value = "100")]
    /// The timeout for subscribers to stop receiving messages. Unit: milliseconds (ms).
    /// The subscriber will start receiving the messages at the same time as the publishers.
    round_timeout: u64,
    #[structopt(short = "i", long, default_value = "100")]
    /// The initial time for starting up futures.
    init_time: u64,
    #[structopt(short = "m", long, default_value = "1")]
    /// The number of messages each publisher peer will try to send.
    num_msgs_per_peer: usize,
    #[structopt(short = "n", long, default_value = "8")]
    /// The payload size of the message.
    payload_size: usize,
}
#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    let args = Cli::from_args();
    dbg!(&args);
    test_worker_1(args).await;
}

async fn test_worker_1(args: Cli) {
    let zenoh = Arc::new(Zenoh::new(net::config::default()).await.unwrap());

    let start = Instant::now();
    let start_until = start + Duration::from_millis(args.init_time);
    let timeout = start_until + Duration::from_millis(args.round_timeout);
    let total_sub_number = args.num_sub_peer;
    let total_put_number = args.num_put_peer;

    let mut msg_payload;

    if args.payload_size == 8 {
        msg_payload = format!("{:08}", 1 as usize);
        let payload_size = std::mem::size_of_val(msg_payload.as_bytes());
        assert!(payload_size == args.payload_size);
    } else if args.payload_size < 8 {
        warn!("Payload size cannot be less than 8 bytes, using 8 bytes for current test.");
        msg_payload = format!("{:08}", 1 as usize);
        let payload_size = std::mem::size_of_val(msg_payload.as_bytes());
        assert!(payload_size == 8);
    } else {
        msg_payload = format!("{:08}", 1 as usize);
        let additional_size = args.payload_size - 8;
        let mut postpend_string = String::from(".");
        for _ in 1..additional_size {
            postpend_string.push_str(".");
        }
        msg_payload.push_str(&postpend_string);
        let payload_size = std::mem::size_of_val(msg_payload.as_bytes());
        assert!(payload_size == args.payload_size);
    }

    let sub_handle_vec = (0..total_sub_number)
        .into_par_iter()
        .map(|peer_id: usize| {
            let sub_handle = async_std::task::spawn(subscribe_worker(
                zenoh.clone(),
                start_until,
                timeout,
                peer_id,
            ));
            sub_handle
        })
        .collect::<Vec<_>>();
    async_std::task::sleep(std::time::Duration::from_millis(50)).await;
    // async_std::task::spawn(publish_worker(zenoh.clone(), start_until));
    let pub_futures = (0..total_put_number).map(|peer_index| {
        publish_worker(
            zenoh.clone(),
            start_until,
            timeout,
            peer_index,
            args.num_msgs_per_peer,
            &msg_payload,
        )
    });
    futures::future::try_join_all(pub_futures).await.unwrap();

    // async_std::task::sleep(std::time::Duration::from_secs(1)).await;
    let sub_handle_fut = sub_handle_vec
        .into_iter()
        .map(|sub_handle| async_std::future::timeout(Duration::from_millis(1000), sub_handle))
        .collect::<Vec<_>>();

    let result = futures::future::try_join_all(sub_handle_fut).await;

    // let result = async_std::future::timeout(Duration::from_millis(1000), sub_handle).await;
    if result.is_err() {
        println!("All messages delivered!");
    } else {
        // for change in result.unwrap().iter() {
        //     println!(
        //         ">> {:?} for {} : {:?} at {}",
        //         change.kind, change.path, change.value, change.timestamp
        //     );
        // }
        let result_vec = result.unwrap();
        for (id, change_vec) in result_vec.iter() {
            println!(
                "sub peer {}: total received messages: {}/{}",
                id,
                change_vec.len(),
                total_put_number
            );
        }
    }
}
