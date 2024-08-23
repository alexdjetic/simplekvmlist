use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use crate::vms::Vms;
use crate::vm::Vm;
use clap::{ArgMatches};

pub fn show_vm(vm: &Vm) -> String {
    format!("> {} : {} <> {} {}", vm.name, vm.vnet, vm.ip, vm.state)
}

pub fn show_vm_full(vm: &Vm) -> String {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut output = format!(
        "> {} :\n- vnet: {}\n- ip: {}\n- disk: {}\n- mac: {}\n- config_xml_file: {}\n- state: ",
        vm.name, vm.vnet, vm.ip, vm.disk, vm.mac, vm.config_xml_file
    );

    match vm.state.as_str() {
        "up" => {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap();
            output.push_str(&format!("{}", vm.state));
        },
        "down" => {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))).unwrap();
            output.push_str(&format!("{}", vm.state));
        },
        _ => {
            output.push_str(&format!("{}", vm.state));
        }
    }

    stdout.reset().unwrap();
    output
}

pub fn show_vm_args(vm: &Vm, matches: &ArgMatches) {
    if matches.get_flag("full") { // if full flag is given
        println!("{}", show_vm_full(vm));
        println!();
    } else {
        println!("{}", show_vm(vm));
    }
}

pub fn cli_launch(matches: &ArgMatches) {
    // Create an instance of Vms
    let mut vms: Vms = Vms::new();
    
    if matches.get_flag("run") {
        for vm_name in vms.get_running_vm() {
            let vm = Vm::new(vm_name.clone());
            show_vm_args(&vm, &matches)
        }
    } else {
        for vm_name in vms.get_vm() {
            let vm = Vm::new(vm_name.clone());
            show_vm_args(&vm, &matches)
        }
    };
}
