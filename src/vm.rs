use crate::execute::{execute_cmd, get_permission_valid, CommandResult};
use std::net::Ipv6Addr;

pub struct Vm {
    pub name: String,
    pub vnet: String,
    pub ips: Vec<String>,
    pub disk: String,
    pub macs: Vec<String>,
    pub config_xml_file: String,
    raw_output: String,
    pub state: String
}

impl Vm {
    // Constructor for Vm struct
    pub fn new(vm_name: String) -> Self {
        let mut vm_instance = Vm {
            name: vm_name,
            vnet: String::new(),
            ips: Vec::new(),
            disk: String::new(),
            macs: Vec::new(),
            config_xml_file: String::new(),
            raw_output: String::new(),
            state: String::new(),
        };

        // Check requirements, e.g., permissions
        if !vm_instance.check_requirements() {
            panic!("Insufficient permissions or other requirements not met.");
        }

        // Get VM information and update fields
        vm_instance.raw_output = vm_instance.get_vm_info();
        vm_instance.vnet = vm_instance.get_vnet();
        vm_instance.macs = vm_instance.get_macs();
        vm_instance.ips = vm_instance.get_ips();
        vm_instance.disk = vm_instance.get_disk();
        vm_instance.config_xml_file = vm_instance.get_xml_file();
        vm_instance.state = vm_instance.get_state();

        vm_instance
    }

    // Get VM information using the virsh command
    pub fn get_vm_info(&self) -> String {
        let command = format!("virsh domiflist {}", self.name);
        let result: CommandResult = execute_cmd(&command);

        if !result.stderr.is_empty() {
            eprintln!("Error: {}", result.stderr);
        }

        result.stdout
    }

    // Extract network interface name from raw_output
    pub fn get_vnet(&self) -> String {
        self.parse_result("Interface")
    }

    // Extract MAC addresses from raw_output
    pub fn get_macs(&self) -> Vec<String> {
        self.raw_output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 && parts[4].contains(':') {
                    Some(parts[4].to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    // Extract IP addresses using the MAC addresses
    pub fn get_ips(&self) -> Vec<String> {
        let mut ips = Vec::new();

        for mac in &self.macs {
            // Try to get both IPv4 and IPv6 addresses using ip neigh show
            let ip_command = format!("ip neigh show | grep '{}' | awk '{{print $1}}'", mac);
            let ip_result: CommandResult = execute_cmd(&ip_command);
            
            if ip_result.status == 0 && !ip_result.stdout.trim().is_empty() {
                ips.extend(ip_result.stdout.lines().map(|s| s.trim().to_string()));
            }
        }

        // If no IPs found or only link-local IPs, try virsh domifaddr
        if ips.is_empty() || ips.iter().all(|ip| ip.starts_with("fe80::")) {
            let domifaddr_command = format!("virsh domifaddr {} | tail -n +3 | awk '{{print $4}}' | cut -d'/' -f1", self.name);
            let domifaddr_result: CommandResult = execute_cmd(&domifaddr_command);
            
            if domifaddr_result.status == 0 && !domifaddr_result.stdout.trim().is_empty() {
                let domifaddr_ips: Vec<String> = domifaddr_result.stdout.lines()
                    .map(|s| s.trim().to_string())
                    .filter(|ip| !ip.starts_with("fe80::"))
                    .collect();
                if !domifaddr_ips.is_empty() {
                    ips = domifaddr_ips;
                }
            }
        }

        // If still no non-link-local IPs found, try to get them from the XML configuration
        if ips.is_empty() || ips.iter().all(|ip| ip.starts_with("fe80::")) {
            let xml_command = format!("virsh dumpxml {} | grep -E '<interface|<ip address=' | sed -n '/<interface/,/<\\/interface>/p'", self.name);
            let xml_result: CommandResult = execute_cmd(&xml_command);
            
            if xml_result.status == 0 && !xml_result.stdout.trim().is_empty() {
                let xml_ips: Vec<String> = xml_result.stdout.lines()
                    .filter(|line| line.contains("<ip address="))
                    .filter_map(|line| line.split('\'').nth(1))
                    .filter(|ip| !ip.starts_with("fe80::"))
                    .map(|ip| ip.to_string())
                    .collect();
                if !xml_ips.is_empty() {
                    ips = xml_ips;
                }
            }
        }

        if ips.is_empty() {
            ips.push("no_ip_found".to_string());
        }
        ips
    }

    // Extract disk information using virsh domblklist
    pub fn get_disk(&self) -> String {
        let command = format!("virsh domblklist {}", self.name);
        let result: CommandResult = execute_cmd(&command);

        if !result.stderr.is_empty() {
            eprintln!("Error: {}", result.stderr);
        }

        // Parse the disk information
        let disks = result.stdout
            .lines()
            .skip(2)
            .map(|line| line.split_whitespace().nth(1).unwrap_or("").to_string())
            .collect::<Vec<String>>()
            .join(", ");

        if disks.is_empty() {
            "no_disk_found".to_string()
        } else {
            disks
        }
    }

    // Extract XML configuration file path using virsh dumpxml
    pub fn get_xml_file(&self) -> String {
        let command = format!("virsh dumpxml {}", self.name);
        let result: CommandResult = execute_cmd(&command);

        if !result.stderr.is_empty() {
            eprintln!("Error: {}", result.stderr);
        }

        // Save the XML content to a temporary file and return the path
        let xml_content = result.stdout;
        let xml_path = format!("/tmp/{}_config.xml", self.name);
        std::fs::write(&xml_path, xml_content).expect("Unable to write XML file");

        xml_path
    }

    // Extract VM state
    pub fn get_state(&self) -> String {
        let command = format!("virsh domstate {}", self.name);
        let result: CommandResult = execute_cmd(&command);

        if !result.stderr.is_empty() {
            eprintln!("Error executing domstate command: {}", result.stderr);
            return "unknown".to_string(); // Handle error by returning a default value
        }

        let state = result.stdout.trim();
        match state {
            // English states
            "running" => "up".to_string(),
            "shut off" => "down".to_string(),

            // French states
            "en cours d’exécution" => "up".to_string(),
            "arrêté" => "down".to_string(),
            "fermé" => "down".to_string(),

            // Unknown state
            _ => {
                eprintln!("Unexpected VM state: {}", state);
                "unknown".to_string() // Handle unexpected states
            },
        }
    }

    // Check if the current user has the necessary permissions
    fn check_requirements(&self) -> bool {
        get_permission_valid()
    }

    // Parse the raw_output to get the specific column based on the header
    fn parse_result(&self, header: &str) -> String {
        let lines: Vec<&str> = self.raw_output.lines().collect();

        // Find the index of the column header
        let headers: Vec<&str> = lines.get(0).unwrap_or(&"").split_whitespace().collect();
        let index = headers.iter().position(|&h| h == header);

        if let Some(index) = index {
            // Iterate over the lines (skip the first line with headers)
            for line in lines.iter().skip(2) {
                let columns: Vec<&str> = line.split_whitespace().collect();

                // Check if the line has the required column
                if let Some(value) = columns.get(index) {
                    return value.to_string();
                }
            }
        }

        "unknown".to_string()
    }
}
