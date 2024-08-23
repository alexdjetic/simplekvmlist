// main.rs
mod execute;
mod vm;
mod vms;
mod show_vm_cli;

use clap::{Arg, Command, ArgMatches};
use show_vm_cli::cli_launch;

fn parse_args() -> ArgMatches {
    Command::new("VM Manager")
        .version("1.0")
        .author("Your Name")
        .about("Manages virtual machines")
        .arg(
            Arg::new("full")
                .short('f')
                .long("full")
                .help("Show full information about VMs")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("run")
                .short('r')
                .long("run")
                .help("List only running VMs")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches()
}

fn main() {
    let matches = parse_args();
    cli_launch(&matches);
}
