use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use libbpf_rs::query::ProgInfoIter;
use std::time::{Duration, Instant};
use std::thread;
use std::fs::File;
use std::io::Write;

mod bpfprog;
use bpfprog::Bpfprog;

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
        #[arg(value_name = "PROG_ID", help = "One or more program IDs to profile")]
        prog_ids: Vec<u32>,

        #[arg(short = 't', long = "time", default_value = "30")]
        duration: u64,

        #[arg(short = 'f', long = "freq", default_value = "997")]
        freq: u64,

        #[arg(short = 'o', long = "output", help = "Output CSV file path")]
        output: Option<String>,
    },

    All {
        #[arg(short = 't', long = "time", default_value = "30")]
        duration: u64,

        #[arg(short = 'f', long = "freq", default_value = "997")]
        freq: u64,

        #[arg(short = 'o', long = "output", help = "Output CSV file path")]
        output: Option<String>,
    },
}

#[derive(Debug, Clone)]
struct RunningStats {
    sample_count: u64,
    total_events: u64,
    avg_runtime_ns: u64,
    min_runtime_ns: u64,
    max_runtime_ns: u64,
    avg_cpu_percent: f64,
    min_cpu_percent: f64,
    max_cpu_percent: f64,
}

impl RunningStats {
    fn new() -> Self {
        Self {
            sample_count: 0,
            total_events: 0,
            avg_runtime_ns: 0,
            min_runtime_ns: u64::MAX,
            max_runtime_ns: 0,
            avg_cpu_percent: 0.0,
            min_cpu_percent: f64::INFINITY,
            max_cpu_percent: 0.0
        }
    }

    fn update(&mut self, events: u64, runtime_delta: u64, cpu_percent: f64) {
        self.sample_count += 1;
        self.total_events += events;

        if events > 0 {
            let avg_runtime_this_sample = runtime_delta / events;

            let old_events_count = self.total_events - events;

            if old_events_count > 0 {
                self.avg_runtime_ns = (self.avg_runtime_ns * old_events_count + runtime_delta) / self.total_events;
            } else {
                self.avg_runtime_ns = avg_runtime_this_sample;
            }

            self.avg_cpu_percent = (self.avg_cpu_percent * old_events_count as f64 + cpu_percent *
                events as f64) / self.total_events as f64;

            self.min_runtime_ns = self.min_runtime_ns.min(avg_runtime_this_sample);
            self.max_runtime_ns = self.max_runtime_ns.max(avg_runtime_this_sample);
        }

        self.min_cpu_percent = self.min_cpu_percent.min(cpu_percent);
        self.max_cpu_percent = self.max_cpu_percent.max(cpu_percent);
    }

    fn avg_events_per_sec(&self, duration_secs: u64) -> u64 {
        if duration_secs == 0 {
            return 0;
        }
        self.total_events / duration_secs 
    }

}

fn print_table_header() {
    println!("┌────────┬──────────────────┬──────────────┬──────────────┬────────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│ {:^6} │ {:^16} │ {:^12} │ {:^12} │ {:^14} │ {:^12} │ {:^12} │ {:^12} │ {:^12} │ {:^12} │ {:^12} │",
             "ID", "Name", "Type", "Events", "Events/sec", "Avg RT(ns)", "Min RT(ns)", "Max RT(ns)", "Avg CPU%", "Min CPU%", "Max CPU%");
    println!("├────────┼──────────────────┼──────────────┼──────────────┼────────────────┼──────────────┼──────────────┼──────────────┼──────────────┼──────────────┼──────────────┤");
}

fn print_table_footer() {
    println!("└────────┴──────────────────┴──────────────┴──────────────┴────────────────┴──────────────┴──────────────┴──────────────┴──────────────┴──────────────┴──────────────┘");
}

fn display_results(prog: &Bpfprog, duration_secs: u64, stats: &RunningStats) {
    if stats.sample_count == 0 {
        println!("│ {:^6} │ {:^16} │ {:^12} │ {:^12} │ {:^14} │ {:^12} │ {:^12} │ {:^12} │ {:^12} │ {:^12} │ {:^12} │",
                 prog.id, prog.name, prog.bpf_type, "N/A", "N/A", "N/A", "N/A", "N/A", "N/A", "N/A", "N/A");
        return;
    }
    
    let avg_events_per_sec = stats.avg_events_per_sec(duration_secs);
    let avg_runtime_ns = stats.avg_runtime_ns;
    let min_runtime_ns = if stats.min_runtime_ns == u64::MAX { 0 } else { stats.min_runtime_ns };
    let max_runtime_ns = stats.max_runtime_ns;
    let avg_cpu = stats.avg_cpu_percent;
    let min_cpu = if stats.min_cpu_percent.is_infinite() { 0.0 } else { stats.min_cpu_percent };
    let max_cpu = stats.max_cpu_percent;

    let display_name = if prog.name.len() > 16 {
        format!("{}...", &prog.name[..13])
    } else {
        prog.name.clone()
    };

    let display_type = if prog.bpf_type.len() > 12 {
        format!("{}...", &prog.bpf_type[..9])
    } else {
        prog.bpf_type.clone()
    };

    println!("│ {:^6} │ {:^16} │ {:^12} │ {:^12} │ {:^14.2} │ {:^12} │ {:^12} │ {:^12} │ {:^12.3} │ {:^12.3} │ {:^12.3} │",
             prog.id,
             display_name,
             display_type,
             stats.total_events,
             avg_events_per_sec,
             avg_runtime_ns,
             min_runtime_ns,
             max_runtime_ns,
             avg_cpu,
             min_cpu,
             max_cpu);
}

fn write_csv_header(file: &mut File) -> Result<()> {
    writeln!(file, "ID,Name,Type,Events,Events_per_sec,Avg_RT_ns,Min_RT_ns,Max_RT_ns,Avg_CPU_percent,Min_CPU_percent,Max_CPU_percent")?;
    Ok(())
}

fn write_csv_row(file: &mut File, prog: &Bpfprog, duration_secs: u64, stats: &RunningStats) -> Result<()> {
    if stats.sample_count == 0 {
        writeln!(file, "{},{},{},N/A,N/A,N/A,N/A,N/A,N/A,N/A,N/A",
                 prog.id, prog.name, prog.bpf_type)?;
        return Ok(());
    }
    
    let avg_events_per_sec = stats.avg_events_per_sec(duration_secs);
    let avg_runtime_ns = stats.avg_runtime_ns;
    let min_runtime_ns = if stats.min_runtime_ns == u64::MAX { 0 } else { stats.min_runtime_ns };
    let max_runtime_ns = stats.max_runtime_ns;
    let avg_cpu = stats.avg_cpu_percent;
    let min_cpu = if stats.min_cpu_percent.is_infinite() { 0.0 } else { stats.min_cpu_percent };
    let max_cpu = stats.max_cpu_percent;

    writeln!(file, "{},{},{},{},{},{},{},{},{:.3},{:.3},{:.3}",
             prog.id,
             prog.name,
             prog.bpf_type,
             stats.total_events,
             avg_events_per_sec,
             avg_runtime_ns,
             min_runtime_ns,
             max_runtime_ns,
             avg_cpu,
             min_cpu,
             max_cpu)?;
    
    Ok(())
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

fn profile_program(prog_id: u32, duration_secs: u64, freq: u64) -> Result<(Bpfprog, RunningStats)> {

    let initial_prog = find_program_by_id(prog_id)
                        .context(format!("BPF program with ID {} not found", prog_id))?;

    let mut stats = RunningStats::new();

    let sample_interval_ns = 1_000_000_000 / freq;

    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(duration_secs);

    let mut prev_runtime_ns = initial_prog.run_time_ns;
    let mut prev_run_cnt = initial_prog.run_cnt;
    let mut prev_sample_time = start_time;

    let mut nxt_sample_time = start_time + Duration::from_nanos(sample_interval_ns);

    loop {
        let now = Instant::now();
        
        if now >= end_time {
            break;
        }

        if now >= nxt_sample_time {
            let elapsed_ns = now.duration_since(prev_sample_time).as_nanos() as u64;

            if let Some(current_prog) = find_program_by_id(prog_id) {
                let runtime_delta = current_prog.run_time_ns.saturating_sub(prev_runtime_ns);
                let run_cnt_delta = current_prog.run_cnt.saturating_sub(prev_run_cnt);

                let cpu_percent = (runtime_delta as f64 / elapsed_ns as f64) * 100.0;

                stats.update(run_cnt_delta, runtime_delta, cpu_percent);

                prev_runtime_ns = current_prog.run_time_ns;
                prev_run_cnt = current_prog.run_cnt;
                prev_sample_time = now;

                nxt_sample_time = nxt_sample_time + Duration::from_nanos(sample_interval_ns);
            }
        }
    }

    Ok((initial_prog, stats))
}

fn profile_programs_parallel(prog_ids: Vec<u32>, duration_secs: u64, freq: u64, output_file: Option<String>) -> Result<()> {
    if prog_ids.is_empty() {
        return Err(anyhow::anyhow!("No program IDs provided"));
    }

    println!("Profiling {} BPF program(s) for {} seconds...\n", prog_ids.len(), duration_secs);

    let mut handles = vec![];

    for prog_id in prog_ids {

        let handle = thread::spawn(move || {
            profile_program(prog_id, duration_secs, freq)
        });
        handles.push(handle);
    }

    let mut results = vec![];
    for handle in handles {
        match handle.join() {
            Ok(Ok((prog, stats))) => results.push((prog, stats)),
            Ok(Err(e)) => eprintln!("Error profiling program: {}", e),
            Err(_) => eprintln!("Thread panicked"),
        }
    }

    if results.is_empty() {
        println!("No results collected");
        return Ok(());
    }

    results.sort_by(|a, b| {
        b.1.total_events.cmp(&a.1.total_events)
    });

    print_table_header();
    
    for (prog, stats) in &results {
        display_results(&prog, duration_secs, &stats);
    }

    print_table_footer();

    if let Some(output_path) = output_file {
        let mut file = File::create(&output_path)
            .context(format!("Failed to create output file: {}", output_path))?;
        
        write_csv_header(&mut file)?;
        
        for (prog, stats) in &results {
            write_csv_row(&mut file, prog, duration_secs, stats)?;
        }
        
        println!("\nResults exported to: {}", output_path);
    }

    Ok(())
}

fn get_all_program_ids() -> Vec<u32> {
    let iter = ProgInfoIter::default();
    iter.map(|prog| prog.id).collect()
}

fn list_programs() -> Result<()> {
    println!("┌────────┬──────────────────┬──────────────┐");
    println!("│ {:^6} │ {:^16} │ {:^12} │", "ID", "Name", "Type");
    println!("├────────┼──────────────────┼──────────────┤");

    let iter = ProgInfoIter::default();
    let mut cnt = 0;
    for prog in iter {
        let prog_name = match prog.name.to_str() {
            Ok(name) if !name.is_empty() => name.to_string(),
            _ => "unknown".to_string(),
        };

        let bpf_type = format!("{:?}", prog.ty);

        let display_name = if prog_name.len() > 16 {
            format!("{}...", &prog_name[..13])
        } else {
            prog_name
        };
        
        let display_type = if bpf_type.len() > 12 {
            format!("{}...", &bpf_type[..9])
        } else {
            bpf_type
        };

        println!("│ {:^6} │ {:^16} │ {:^12} │", prog.id, display_name, display_type);

        cnt += 1;
    }

    println!("└────────┴──────────────────┴──────────────┘");
    println!("Total: {} BPF programs", cnt);
    Ok(())
}

fn main() -> Result<()> {

    let cli = Cli::parse();

    match cli.command {
        Commands::Prog { action } => match action {
            ProgAction::Id { prog_ids, duration, freq, output } => {
                unsafe {
                    libbpf_sys::bpf_enable_stats(libbpf_sys::BPF_STATS_RUN_TIME);
                }
                profile_programs_parallel(prog_ids, duration, freq, output)?;
            }
            ProgAction::All { duration, freq, output } => {
                unsafe {
                    libbpf_sys::bpf_enable_stats(libbpf_sys::BPF_STATS_RUN_TIME);
                }
                let bpf_prog_ids = get_all_program_ids();
                profile_programs_parallel(bpf_prog_ids, duration, freq, output)?;
            }
        },
        Commands::List => {
            list_programs()?;
        }
    }

    Ok(())
}
