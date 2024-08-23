use crate::vm::Vm;
use crate::execute::{execute_cmd, CommandResult};

pub struct Vms {
    pub vm: Vec<Vm>,
    pub number: i32,
}

impl Vms {
    pub fn new() -> Self {
        Self {
            vm: Vec::new(),
            number: 0,
        }
    }

    pub fn describe(&self) -> String {
        format!("number of vms: {}", self.number)
    }

    pub fn get_vm(&mut self) -> Vec<String> {
        self.clear();
        let result: CommandResult = execute_cmd("virsh list --all --name");
        let all_vm: Vec<String> = result.stdout.lines()
            .filter_map(|s| {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            }).collect();

        for vm_name in &all_vm {
            self.vm.push(Vm::new(vm_name.clone()));
            self.number += 1;
        }

        all_vm
    }

    pub fn get_running_vm(&mut self) -> Vec<String> {
        self.clear();
        let result: CommandResult = execute_cmd("virsh list --state-running --name");
        let all_vm: Vec<String> = result.stdout.lines()
            .filter_map(|s| {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            }).collect();

        for vm_name in &all_vm {
            self.vm.push(Vm::new(vm_name.clone()));
            self.number += 1;
        }

        all_vm
    }

    /// Finds VMs that match a given filter using shell commands for filtering.
    ///
    /// # Arguments
    /// * filter - A string containing a simple string to filter VM names.
    ///
    /// # Returns
    /// A vector of strings containing the names of VMs that match the filter.
    pub fn find_vms(&self, filter: &str) -> Vec<String> {
        // Determine the regex pattern based on the filter string
        let pattern = if filter.starts_with('^') || filter.contains('*') {
            filter.to_string()
        } else {
            format!(".*{}.*", filter)
        };

        // Create the command to execute
        let command = format!("virsh list --all --name | grep -E '{}'", pattern);

        // Print the command for debugging
        println!("Executing command: {}", command);

        // Execute the command and capture the result
        let result: CommandResult = execute_cmd(&command);

        // Process the command output
        let all_vm: Vec<String> = result.stdout.lines()
            .filter_map(|s| {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            })
            .collect();

        all_vm
    }

    fn clear(&mut self) {
        self.vm.clear();
        self.number = 0;
    }
}