use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};
use libbpf_rs::query::ProgInfoIter;
use std::time::{Duration, Instant};
use std::mem::MaybeUninit;
use std::thread;

mod bpfprog;
use bpfprog::Bpfprog;

mod pid_iter {
    include!(concat!(env!("OUT_DIR"), "/pid_iter.skel.rs"));
}
use pid_iter::PidIterSkelBuilder;

#[derive(Parser)]
#[command(name = "bpfprof")]
#[command(about = "Profile BPF programs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Prog {
        #[command(subcommand)]
        action: ProgAction,
    },

    List,
}

#[derive(Subcommand)]
enum ProgAction {
    Id {
        prog_id: u32,

        #[arg(short = 't', long = "time", default_value = "30")]
        duration: u64,
    },
}

#[derive(Debug, Clone)]
struct Sample {
    event_per_sec: u64,
    avg_run_time_ns: u64,
    cpu_percent: f64,
}

fn load_pid_iter(iter_link: &mut Option<libbpf_rs::Link>) -> Result<()> {

    let prev_print_fn = unsafe {
        libbpf_sys::libbpf_set_print(None)
    };

    let result = (|| -> Result<()> {
        let skel_builder = PidIterSkelBuilder::default();
        let mut open_object = MaybeUninit::uninit();
        let open_skel = skel_builder.open(&mut open_object)?;
        let mut skel = open_skel.load()?;
        skel.attach()?;
        *iter_link = skel.links.bpf_iter;
        Ok(())
    })();

    unsafe {
        libbpf_sys::libbpf_set_print(prev_print_fn); 
    }
    
    result
}

fn display_results(prog: &Bpfprog, samples: &[Sample]) {
    if samples.is_empty() {
        println!("Failed to collect samples :/");
        return;
    }

    let total_events: u64 = samples.iter().map(|s| s.event_per_sec).sum();
    let avg_events_per_sec: u64 = total_events / samples.len() as u64;

    let runtimes: Vec<u64> = samples.iter()
        .filter(|s| s.avg_run_time_ns > 0)
        .map(|s| s.avg_run_time_ns)
        .collect();

    let avg_runtime_ns = if !runtimes.is_empty() {
        runtimes.iter().sum::<u64>() / runtimes.len() as u64 
    } else {
        0
    };

    let min_runtime_ns = runtimes.iter().copied().min().unwrap_or(0);
    let max_runtime_ns = runtimes.iter().copied().max().unwrap_or(0);

    let avg_cpu = samples.iter().map(|s| s.cpu_percent).sum::<f64>() / samples.len() as f64;
    let min_cpu = samples.iter()
        .map(|s| s.cpu_percent)
        .fold(f64::INFINITY, |a, b| a.min(b));
    let max_cpu = samples.iter()
        .map(|s| s.cpu_percent)
        .fold(0.0f64, |a, b| a.max(b));

    println!("Prog name\t|\tProgram type\t|\tTotal events\t|\tAvg events/sec\t|\tAvg runtime per event\t|\tMin Runtime per event\t|\tMax Runtime per event\t|\tAvg CPU usage\t|\tMin CPU Usage\t|\tMax CPU usage\n");
    println!("{}\t|\t{}\t|\t{}\t|\t{}\t|\t{}\t|\t{}\t|\t{}\t|\t{:.2}%\t|\t{:.2}%\t|\t{:.2}%\n", prog.name, prog.bpf_type , total_events, avg_events_per_sec, avg_runtime_ns, min_runtime_ns, max_runtime_ns, avg_cpu, min_cpu, max_cpu);
}

fn find_program_by_id(prog_id: u32) -> Option<Bpfprog> {
    
    let iter = ProgInfoIter::default();
    
    for prog in iter {
        if prog.id == prog_id {
            let prog_name = prog.name.to_str().ok()?.to_string();
            let bpf_type = format!("{:?}", prog.ty);

            return Some(Bpfprog::new(
                    prog.id,
                    bpf_type,
                    prog_name,
                    prog.run_time_ns,
                    prog.run_cnt,
                ));
        }
    }

    None
}

fn profile_program(prog_id: u32, duration_secs: u64) -> Result<()> {
    println!("Profiling BPF program ID {} for {} seconds...\n", prog_id, duration_secs);

    let initial_prog = find_program_by_id(prog_id)
                        .context(format!("BPF program with ID {} not found", prog_id))?;

    let mut samples: Vec<Sample> = Vec::new();

    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(duration_secs);

    let mut prev_run_time_ns = initial_prog.run_time_ns;
    let mut prev_run_cnt = initial_prog.run_cnt;

    while Instant::now() < end_time {
        // Why exactly? - 1 second seems a lot
        thread::sleep(Duration::from_secs(1));

        if let Some(current_prog) = find_program_by_id(prog_id) {

            let runtime_delta = current_prog.run_time_ns.saturating_sub(prev_run_time_ns);
            let run_cnt_delta = current_prog.run_cnt.saturating_sub(prev_run_cnt);

            let events_per_sec = run_cnt_delta;
            let avg_runtime_ns = if run_cnt_delta > 0 {
                runtime_delta / run_cnt_delta
            } else {
                0
            };

            let cpu_percent = (runtime_delta as f64 / 1_000_000_000.0) * 100.0; 

            samples.push(Sample {
                event_per_sec: events_per_sec,
                avg_run_time_ns: avg_runtime_ns,
                cpu_percent: cpu_percent,
            });

            prev_run_time_ns = current_prog.run_time_ns;
            prev_run_cnt = current_prog.run_cnt;

        } else {
            println!("\n WARN: Prog ID {} no longer exits", prog_id);
            break;
        }
    
    }

    display_results(&initial_prog, &samples);

    Ok(())
}

fn main() -> Result<()> {

//    let mut iter_link: Option<libbpf_rs::Link> = None;
//    match load_pid_iter(&mut iter_link) {
//        Ok(()) => println!("Successfully loaded pid_iter BPF program"),
//        Err(e) => println!("Failed to load pid_iter BPF program: {}, continuing without process information", e),
//    }
//
    unsafe {
        libbpf_sys::bpf_enable_stats(libbpf_sys::BPF_STATS_RUN_TIME);
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Prog { action } => match action {
            ProgAction::Id { prog_id, duration } => {
                profile_program(prog_id, duration)?;
            }
        },
        Commands::List => {
        }
    }


    Ok(())
}
