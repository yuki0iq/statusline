use crate::{virt, Icon, IconMode};
use std::{
    fs::{read_to_string, File},
    io::{BufRead, BufReader},
    path::Path,
};

/// Chassis type, according to hostnamectl
#[derive(Copy, Clone)]
pub enum Chassis {
    /// Desktops, nettops, etc
    Desktop,
    /// Servers (which are in server rack)
    Server,
    /// Laptops, notebooks
    Laptop,
    /// Convertible laptops (which can turn into tablets)
    Convertible,
    /// Tablets
    Tablet,
    /// Phone? should check original documentation again lmao
    Handset,
    /// Smart watches
    Watch,
    /// Embedded devices
    Embedded,
    /// Virtual machines
    Virtual,
    /// Containered environments
    Container,
    /// Something else
    Unknown,
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

impl Icon for Chassis {
    fn icon(&self, mode: &IconMode) -> &'static str {
        use Chassis::*;
        use IconMode::*;
        match self {
            Desktop => match mode {
                Text => "Desk",
                Icons | MinimalIcons => " ",
            },
            Server => match mode {
                Text => "Serv",
                Icons | MinimalIcons => "󰒋 ",
            },
            Laptop => match mode {
                Text => "Lapt",
                Icons | MinimalIcons => "󰌢 ",
            },
            Convertible => match mode {
                Text => "Conv",
                Icons | MinimalIcons => "󰊟 ", // TODO: probably this icon is not the best fit, but the best I could come up with at 2 AM
            },
            Tablet => match mode {
                Text => "Tabl",
                Icons | MinimalIcons => " ",
            },
            Handset => match mode {
                Text => "Hand",
                Icons | MinimalIcons => " ",
            },
            Watch => match mode {
                Text => "Watch",
                Icons | MinimalIcons => " ",
            },
            Embedded => match mode {
                Text => "Emb",
                Icons | MinimalIcons => " ",
            },
            Virtual => match mode {
                Text => "Virt",
                Icons | MinimalIcons => " ",
            },
            Container => match mode {
                Text => "Cont",
                Icons | MinimalIcons => " ",
            },
            Unknown => match mode {
                Text => "Unkn",
                Icons | MinimalIcons => "??", // TODO: find "unknown" icon
            },
        }
    }
}

impl Chassis {
    /// Gets chassis type from system information, as in systemd
    ///
    /// Containered and virtual environments are likely to be misdetected. You can try overriding
    /// this via `/etc/machine-info` or `hostnamectl set-chassis`...
    pub fn get() -> Chassis {
        None.or_else(Chassis::try_machine_info)
            .or_else(Chassis::try_container)
            .or_else(Chassis::try_udev)
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
                        .strip_prefix('=')?
                        .trim_start(),
                ))
            })
    }

    fn try_udev() -> Option<Chassis> {
        /*
        sd-device /sys/class/dmi/id points to /run/udev/data/+dmi:id
        hours wasted on this: about three,
          trying to read, understand and interpret
          systemd code without stracing, lmao
        find line "E:ID_CHASSIS=..."
        ---
        I can't be 100% sure this code works as NO machines I have acceess to
          have any chassis-related information in this file
        */
        BufReader::new(File::open("/run/udev/data/+dmi:id").ok()?)
            .lines()
            .find_map(|x| Some(Chassis::from(x.ok()?.strip_prefix("E:ID_CHASSIS=")?.trim())))
    }

    fn try_virtualization() -> Option<Chassis> {
        // No one knows if this works correctly
        virt::detect_vm().ok()?.map(|_| Chassis::Virtual)
    }

    fn try_container() -> Option<Chassis> {
        // No one knows if this works correctly
        virt::detect_container().ok()?.map(|_| Chassis::Container)
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
