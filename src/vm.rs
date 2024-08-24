use crate::execute::{execute_cmd, get_permission_valid, CommandResult};

pub struct Vm {
    pub name: String,
    pub vnet: String,
    pub ip: String,
    pub disk: String,
    pub mac: String,
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
            ip: String::new(),
            disk: String::new(),
            mac: String::new(),
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
        vm_instance.mac = vm_instance.get_mac();
        vm_instance.ip = vm_instance.get_ip();
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

    // Extract MAC address from raw_output
    pub fn get_mac(&self) -> String {
        self.parse_result("MAC")
    }

    // Extract IP address using the MAC address
    pub fn get_ip(&self) -> String {
        if self.mac.is_empty() {
            return "unknown".to_string();
        }

        // Construct command to get IP based on MAC address
        let command = format!(
            "ip neigh show | awk -v mac='{}' '$5 == mac {{print $1}}'",
            self.mac
        );

        let result: CommandResult = execute_cmd(&command);

        if result.status != 0 {
            "example_ip".to_string()
        } else {
            // Split the stdout by newline and filter out any empty lines
            let ips: Vec<String> = result
                .stdout
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .map(|line| line.to_string())
                .collect();

            // Join the IPs with a separator, e.g., a comma
            if ips.is_empty() {
                "no_ip_found".to_string()
            } else {
                ips.join(", ")
            }
        }
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
