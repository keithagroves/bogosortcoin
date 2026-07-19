use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bogosortcoin_consensus::bogopow::{generate_proof, is_sorted, ticket_meets_target};
use bogosortcoin_consensus::{block_work, BlockHeader};
use bogosortcoin_primitives::Hash256;
use clap::Parser;
use serde::Serialize;

/// Toy CLI miner that searches nonces for a valid BogoPoW block header.
///
/// This has no networking, mempool, or persistence: it builds one candidate
/// header from the given fields and scans nonces until the permutation is
/// sorted and the ticket meets the target, per specifications/consensus.md.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long, default_value_t = 1)]
    network_id: u32,

    #[arg(long, default_value_t = 1)]
    protocol_version: u32,

    #[arg(long, default_value_t = 0)]
    height: u64,

    #[arg(long, default_value = "0000000000000000000000000000000000000000000000000000000000000000")]
    previous_hash: String,

    #[arg(long, default_value = "0000000000000000000000000000000000000000000000000000000000000000")]
    merkle_root: String,

    #[arg(long, default_value = "0000000000000000000000000000000000000000000000000000000000000000")]
    state_root: String,

    #[arg(long, default_value = "0000000000000000000000000000000000000000000000000000000000000000")]
    miner_commitment: String,

    /// Big-endian hex difficulty target; smaller is harder. Defaults to an
    /// easy target so a demo run finishes in seconds.
    #[arg(long, default_value = "0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f")]
    target: String,

    #[arg(long, default_value_t = 8)]
    permutation_size: u16,

    #[arg(long, default_value_t = 0)]
    extra_nonce: u64,

    #[arg(long, default_value_t = 0)]
    start_nonce: u64,

    #[arg(long, default_value_t = 500_000)]
    report_interval: u64,

    /// Write a JSON snapshot of the current attempt to this file (~30 times
    /// per second) for a web front end to poll. Off by default.
    #[arg(long)]
    stream_file: Option<PathBuf>,
}

#[derive(Serialize)]
struct Snapshot {
    attempts: u64,
    nonce: u64,
    permutation_size: u16,
    permutation: Vec<u16>,
    seed: String,
    ticket: String,
    target: String,
    sorted: bool,
    meets_target: bool,
    found: bool,
    elapsed_ms: u128,
    rate: f64,
}

fn write_snapshot(path: &PathBuf, snapshot: &Snapshot) {
    let tmp_path = path.with_extension("json.tmp");
    let body = serde_json::to_vec(snapshot).expect("snapshot serializes");
    let mut file = std::fs::File::create(&tmp_path).expect("create snapshot temp file");
    file.write_all(&body).expect("write snapshot temp file");
    std::fs::rename(&tmp_path, path).expect("rename snapshot into place");
}

fn parse_hash(label: &str, s: &str) -> Hash256 {
    let bytes = hex::decode(s).unwrap_or_else(|e| panic!("invalid hex for {label}: {e}"));
    bytes
        .try_into()
        .unwrap_or_else(|v: Vec<u8>| panic!("{label} must be 32 bytes, got {}", v.len()))
}

fn main() {
    let args = Args::parse();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_secs();

    let base_header = BlockHeader {
        network_id: args.network_id,
        protocol_version: args.protocol_version,
        height: args.height,
        previous_block_hash: parse_hash("previous-hash", &args.previous_hash),
        transaction_merkle_root: parse_hash("merkle-root", &args.merkle_root),
        state_root: parse_hash("state-root", &args.state_root),
        timestamp,
        difficulty_target: parse_hash("target", &args.target),
        permutation_size: args.permutation_size,
        miner_commitment: parse_hash("miner-commitment", &args.miner_commitment),
        extra_nonce: args.extra_nonce,
        nonce: 0,
    };

    println!(
        "mining: N={} target=0x{}",
        args.permutation_size, args.target
    );

    let start = Instant::now();
    let mut nonce = args.start_nonce;
    let mut attempts: u64 = 0;
    let mut last_stream_write = Instant::now() - Duration::from_secs(1);
    let stream_interval = Duration::from_millis(33);

    loop {
        let mut header = base_header.clone();
        header.nonce = nonce;

        let proof = generate_proof(&header);
        attempts += 1;
        let sorted = is_sorted(&proof.permutation);
        let meets_target = ticket_meets_target(&proof.ticket, &header.difficulty_target);
        let found = sorted && meets_target;

        if let Some(path) = &args.stream_file {
            if found || last_stream_write.elapsed() >= stream_interval {
                let elapsed = start.elapsed();
                let rate = attempts as f64 / elapsed.as_secs_f64().max(1e-9);
                write_snapshot(
                    path,
                    &Snapshot {
                        attempts,
                        nonce: header.nonce,
                        permutation_size: header.permutation_size,
                        permutation: proof.permutation.clone(),
                        seed: hex::encode(proof.seed),
                        ticket: hex::encode(proof.ticket),
                        target: hex::encode(header.difficulty_target),
                        sorted,
                        meets_target,
                        found,
                        elapsed_ms: elapsed.as_millis(),
                        rate,
                    },
                );
                last_stream_write = Instant::now();
            }
        }

        if found {
            let elapsed = start.elapsed();
            let rate = attempts as f64 / elapsed.as_secs_f64().max(1e-9);
            println!("--- found valid block ---");
            println!("nonce:       {}", header.nonce);
            println!("attempts:    {attempts}");
            println!("elapsed:     {elapsed:?}");
            println!("attempts/s:  {rate:.1}");
            println!("seed:        {}", hex::encode(proof.seed));
            println!("permutation: {:?}", proof.permutation);
            println!("ticket:      {}", hex::encode(proof.ticket));
            println!("chain work:  {}", block_work(header.permutation_size, &header.difficulty_target));
            break;
        }

        if attempts % args.report_interval == 0 {
            let rate = attempts as f64 / start.elapsed().as_secs_f64().max(1e-9);
            println!("... {attempts} attempts, {rate:.1}/s");
        }

        nonce = nonce.wrapping_add(1);
    }
}
