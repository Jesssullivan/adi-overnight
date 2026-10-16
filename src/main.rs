//!  SMS Sim, minimal non-keyed, distribution binned
//!
//! Args & intended use:
//! cargo run --release -- [messages], [senders], [mean_ms], [fail_pct], [every_ms]
//!
//! rand() options:
//! RandomState:: (slow, non-keyed, boring)
//!  -- Likely what is basally expected and would be the expectation in a DSA exam; not good enough
//! QuickMix64 non-keyed, extremely fast a single draw steaming impl;
//!  -- this is what I have implemented below
//! QuickMix64 keyed, triple draw (Mix13)
//! -- see my Haskell implementation plz I worked very hard on my initial submission
//! according to my OTel stats on my personal machine, roughly 51 hours spent on that submission
//!
//! this code was written on camera in one take, between ~4:15am and 7am Oct 16
//! (ffmpeg spluttered out I now see around the 2 hour mark when my earphones ran out of battery)

//! going to use mpsc for now, given this is still a demo / kalvinball scoped project
use std::thread;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::RecvTimeoutError;
use std::hint::black_box; // we'll dump this in sender work loop
use std::time::{Duration, Instant};
use std::sync::{mpsc, Mutex};  // will want timeouterr as well

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

static STREAM: AtomicU64 = AtomicU64::new(7);

fn rand(n: u64) -> u64 {
    mix(STREAM.fetch_add(0x9E37_79B9_7F4A_7C15, Relaxed)) % n
}

fn main() {

    let args: Vec<u64> = std::env::args().skip(1).map(|a| a.parse().unwrap()).collect();
    let arg = |i : usize, default : u64| args.get(i).copied().unwrap_or(default);

    let n = arg(0, 1000);
    let senders = arg(1, 10).max(1);
    let mean_ms = arg(2, 100);
    let fail_pct = arg(3, 10);  // this should be percent (pct) not ms ...

    // This should be a duration I think actually
    let every=  Duration::from_millis(arg(4, 1000).max(1));

    // establish queue:
    let (queue_tx, queue_rx) = mpsc::sync_channel::<String>(senders as usize);
    let queue_rx = Mutex::new(queue_rx);


    let sent = AtomicU64::new(0);
    let failed = AtomicU64::new(0);
    let busy_ms= AtomicU64::new(0);

    let start = Instant::now();

    let report = || {
        let (s, f) = (sent.load(Relaxed), failed.load(Relaxed));
        let avg = busy_ms.load(Relaxed) as f64 / (s + f).max(1) as f64; // make avg :squint:
        let t = start.elapsed().as_secs_f64();
        println!("{t:5.1}s  sent {s:5}  failed {f:4}  avg {avg:5.1} ms");
    };

    // establish done:
    let (done_tx, done_rx) = mpsc::channel::<()>();  // right?


    thread::scope(|s| {

        // hmm, produce I think at least for the purpose of this exercise can be scoped here;
        // of course this seems sort underwhelming :/
        let produce = move || {
            // create message of a reasonable size <= 100 chars alphanumericish
            for _ in 0..n {
                // create message body
                // noting rand is going to ofc be one of the discussion points here /
                // personal points of contention (RandomState < tinyMT < xor shifting < QuickMix 1 draw < QuickMix64 keyed
                let body = (0..rand(101)).map(|_| (b' ' + rand(95) as u8) as char).collect();
                // push to queue
                queue_tx.send(body).unwrap();  // why send unhappy?  hhmpf.
            }
        };

        // first create producer thread, give name, scoped; producer is one producer to many senders
        thread::Builder::new().name("producer".into()).spawn_scoped(s, produce).unwrap();

        // sender loop
        for k in 0..senders {
            let done = done_tx.clone();
            let sender_work = || {
                let _done = done;
                loop {
                    let Ok(body) = queue_rx.lock().unwrap().recv() else { break };
                    let rand_ms = rand(2 * mean_ms + 1);
                    thread::sleep(Duration::from_millis(rand_ms));
                    // bb:
                    black_box(body);
                    // deal w/ outcome
                    let outcome = if rand(100) < fail_pct { &failed } else { &sent };
                    outcome.fetch_add(1, Relaxed);
                    busy_ms.fetch_add(rand_ms, Relaxed);

                }
            };
            // sender thread
            thread::Builder::new().name(format!("sender-{k}")).spawn_scoped(s, sender_work).unwrap();
        }
        // drop & timeouts; print report()
        drop(done_tx);
        while let Err(RecvTimeoutError::Timeout) = done_rx.recv_timeout(every)   {
            report()
        }
    });

    // reporter for Summary Monoid; report every_ms
    report();

    // lastly, make OOB prints and stats (^ every_ms)
    let total = sent.load(Relaxed) + failed.load(Relaxed);

    // single assertion, perhaps this can count as a test
    assert_eq!(total, n, "every message is sent or failed exactly once!!  huzzah!");
    let rate = total as f64 / start.elapsed().as_secs_f64();  // total - elapsed
    println!("{total} messages, {senders} senders, {rate:.0} msg/s");
}

