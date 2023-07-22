use std::{
    fs::{read_to_string, File},
    io::{BufRead, BufReader},
    path::Path,
};

pub enum Chassis {
    Desktop,
    Server,
    Laptop,
    Convertible,
    Tablet,
    Handset,
    Watch,
    Embedded,
    Virtual,
    Container,
    Unknown,
}

impl Chassis {
    // TODO
    pub fn icon(&self) -> &str {
        match self {
            Chassis::Laptop => "💻",
            Chassis::Desktop => "🖥",
            Chassis::Server => "🖳 ",
            Chassis::Tablet => "具",
            Chassis::Watch => "⌚️",
            Chassis::Handset => "🕻",
            Chassis::Virtual => "🖴 ",
            Chassis::Container => "☐ ",
            Chassis::Convertible => "󰒋 ",
            _ => "??",
        }
    }
}

impl From<&str> for Chassis {
    fn from(s: &str) -> Chassis {
        match s {
            "desktop" => Chassis::Desktop,
            "server" => Chassis::Server,
            "laptop" => Chassis::Laptop,
            "convertible" => Chassis::Convertible,
            "tablet" => Chassis::Tablet,
            "handset" => Chassis::Handset,
            "watch" => Chassis::Watch,
            "embedded" => Chassis::Embedded,
            "vm" => Chassis::Virtual,
            "container" => Chassis::Container,
            _ => Chassis::Unknown,
        }
    }
}

impl Chassis {
    pub fn get() -> Chassis {
        // Get chassis type --- from `hostnamed` source code

        None.or_else(Chassis::try_machine_info)
            .or_else(Chassis::try_dmi_id)
            .or_else(Chassis::try_virtualization)
            .or_else(Chassis::try_dmi_type)
            .or_else(Chassis::try_acpi_profile)
            .or_else(Chassis::try_devtree_type)
            .unwrap_or(Chassis::Unknown)
    }

    fn try_machine_info() -> Option<Chassis> {
        /*
        /etc/machine-info into lines
        find-map
        - trim-ws
        | take "CHASSIS"
        | trim-left-ws
        | take "="
        | trim-left-ws
        | unquote
        | chassis_str
        */
        BufReader::new(File::open("/etc/machine-info").ok()?)
            .lines()
            .find_map(|x| {
                Some(Chassis::from(
                    x.ok()?
                        .trim()
                        .strip_prefix("CHASSIS")?
                        .trim_start()
                        .strip_prefix("=")?
                        .trim_start(),
                ))
            })
    }

    fn try_dmi_id() -> Option<Chassis> {
        /*
        TODO /sys/class/dmi/id @ID_CHASSIS
        */
        None
    }

    fn try_virtualization() -> Option<Chassis> {
        /*
        TODO detect_virtualization() -> (vm, container)
        */
        None
    }

    fn try_dmi_type() -> Option<Chassis> {
        /*
        /sys/class/dmi/id/chassis_type as u32 in hex
        3, 4, 6, 7, D, 23, 24 -> desktop
        8, 9, A, E -> laptop
        B -> handset
        11, 1C, 1D -> server
        1E -> tablet
        1F, 20 -> convertible
        21, 22 -> embedded
        */
        Some(match read_single_u32("/sys/class/dmi/id/chassis_type")? {
            0x03 | 0x04 | 0x06 | 0x07 | 0x0d | 0x23 | 0x24 => Chassis::Desktop,
            0x08 | 0x09 | 0x0a | 0x0e => Chassis::Laptop,
            0x0b => Chassis::Handset,
            0x11 | 0x1c | 0x1d => Chassis::Server,
            0x1e => Chassis::Tablet,
            0x1f | 0x20 => Chassis::Convertible,
            0x21 | 0x22 => Chassis::Embedded,
            _ => Chassis::Unknown,
        })
    }

    fn try_acpi_profile() -> Option<Chassis> {
        /*
        /sys/firmware/acpi/pm_profile as u32 in dec
        1, 3, 6 -> desktop
        2 -> laptop
        4, 5, 7 -> server
        8 -> tablet
        */
        Some(match read_single_u32("/sys/firmware/acpi/pm_profile")? {
            1 | 3 | 6 => Chassis::Desktop,
            2 => Chassis::Laptop,
            4 | 5 | 7 => Chassis::Server,
            8 => Chassis::Tablet,
            _ => Chassis::Unknown,
        })
    }

    fn try_devtree_type() -> Option<Chassis> {
        /*
        /proc/device-tree/chassis-type as chassis_str
        */
        Some(Chassis::from(
            read_to_string("/proc/device-tree/chassis-type")
                .ok()?
                .as_str(),
        ))
    }
}

fn read_single_u32<T: AsRef<Path> + ?Sized>(path: &T) -> Option<u32> {
    read_to_string(path).ok()?.trim().parse::<u32>().ok()
}
