use anyhow::Result;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, ErrorKind},
};

#[derive(Debug)]
pub enum VirtualizationType {
    KVM,
    Amazon,
    QEMU,
    Bochs,
    Xen,
    UML,
    VMware,
    Oracle,
    Microsoft,
    ZVM,
    Parallels,
    Bhyve,
    QNX,
    ACRN,
    PowerVM,
    Apple,
    SRE,
    Other,
}

fn detect_vm_cpuid() -> Result<Option<VirtualizationType>> {
    // TODO detect_vm_cpuid
    Ok(None)
}

fn detect_vm_device_tree() -> Result<Option<VirtualizationType>> {
    Ok(
        match fs::read_to_string("/proc/device-tree/hypervisor/compatible") {
            Ok(s) if s == "linux,kvm" => Some(VirtualizationType::KVM),
            Ok(s) if s.contains("xen") => Some(VirtualizationType::Xen),
            Ok(s) if s.contains("vmware") => Some(VirtualizationType::VMware),
            Ok(_) => Some(VirtualizationType::Other),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                match fs::try_exists("/proc/device-tree/ibm,partition-name").unwrap_or(false)
                    && fs::try_exists("/proc/device-tree/hmc-managed?").unwrap_or(false)
                    && !fs::try_exists("/proc/device-tree/chosen/qemu,graphic-width")
                        .unwrap_or(false)
                {
                    true => Some(VirtualizationType::PowerVM),
                    false => match fs::read_dir("/proc/device-tree") {
                        Ok(mut iter) => match iter
                            .find_map(|entry| {
                                Some(
                                    entry
                                        .ok()?
                                        .file_name()
                                        .to_str()
                                        .filter(|name| name.contains("fw-cfg"))
                                        .is_some(),
                                )
                            })
                            .is_some()
                        {
                            true => Some(VirtualizationType::QEMU),
                            false => match fs::read_to_string("/proc/device-tree/compatible") {
                                Ok(s) if s == "qemu,pseries" => Some(VirtualizationType::QEMU),
                                Ok(_) => None,
                                Err(e) if e.kind() == ErrorKind::NotFound => None,
                                Err(e) => Err(e)?,
                            },
                        },
                        Err(e) if e.kind() == ErrorKind::NotFound => None,
                        Err(e) => Err(e)?,
                    },
                }
            }
            Err(e) => Err(e)?,
        },
    )
}

fn detect_vm_dmi_vendor() -> Result<Option<VirtualizationType>> {
    for path in [
        "/sys/class/dmi/id/product_name",
        "/sys/class/dmi/id/sys_vendor",
        "/sys/class/dmi/id/board_vendor",
        "/sys/class/dmi/id/bios_vendor",
        "/sys/class/dmi/id/product_version",
    ] {
        match fs::read_to_string(path) {
            Ok(s) => {
                for (vendor, vm) in [
                    ("KVM", VirtualizationType::KVM),
                    ("OpenStack", VirtualizationType::KVM),
                    ("KubeVirt", VirtualizationType::KVM),
                    ("Amazon EC2", VirtualizationType::Amazon),
                    ("QEMU", VirtualizationType::QEMU),
                    ("VMware", VirtualizationType::VMware),
                    ("VMW", VirtualizationType::VMware),
                    ("innotek GmbH", VirtualizationType::Oracle),
                    ("VirtualBox", VirtualizationType::Oracle),
                    ("Xen", VirtualizationType::Xen),
                    ("Bochs", VirtualizationType::Bochs),
                    ("Parallels", VirtualizationType::Parallels),
                    ("BHYVE", VirtualizationType::Bhyve),
                    ("Hyper-V", VirtualizationType::Microsoft),
                    ("Apple Virtualization", VirtualizationType::Apple),
                ] {
                    if s.starts_with(vendor) {
                        return Ok(Some(vm));
                    }
                }
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => Err(e)?,
        }
    }
    Ok(None)
}

fn detect_vm_smbios_impl() -> Result<Option<bool>> {
    Ok(fs::read("/sys/firmware/dmi/entries/0-0/raw")?
        .get(19)
        .map(|x| ((x >> 4) & 1) == 1))
}

fn detect_vm_smbios() -> Option<bool> {
    detect_vm_smbios_impl().unwrap_or(None)
}

fn detect_vm_dmi() -> Result<Option<VirtualizationType>> {
    Ok(match detect_vm_dmi_vendor()? {
        Some(VirtualizationType::Amazon) => match detect_vm_smbios() {
            Some(true) => Some(VirtualizationType::Amazon),
            Some(false) => None,
            None => {
                match fs::read_to_string("/sys/class/dmi/id/product_name") {
                    Ok(s) => {
                        let s = s.lines().next().unwrap_or_default();
                        (s.contains(".metal-") || s.ends_with(".metal")) // TODO
                            .then_some(VirtualizationType::Amazon)
                    }
                    Err(_) => Some(VirtualizationType::Amazon),
                }
            }
        },
        None if detect_vm_smbios().unwrap_or(false) => Some(VirtualizationType::Other),
        vm @ _ => vm,
    })
}

fn detect_vm_xen_dom0() -> Result<bool> {
    Ok(
        match fs::read_to_string("/sys/hypervisor/properties/features") {
            Ok(s) => (u64::from_str_radix(&s, 16)? >> 11) & 1 == 1,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                match fs::read_to_string("/proc/xen/capabilities") {
                    Ok(s) => s.contains("control_d"),
                    Err(e) if e.kind() == ErrorKind::NotFound => false,
                    Err(e) => Err(e)?,
                }
            }
            Err(e) => Err(e)?,
        },
    )
}

fn detect_vm_xen() -> Result<Option<VirtualizationType>> {
    Ok(fs::try_exists("/proc/xen")?.then_some(VirtualizationType::Xen))
}

fn detect_vm_hypervisor() -> Result<Option<VirtualizationType>> {
    Ok(match fs::read_to_string("/sys/hypervisor/type") {
        Ok(s) if s == "xen" => Some(VirtualizationType::Xen),
        Ok(_) => Some(VirtualizationType::Other),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => Err(e)?,
    })
}

fn detect_vm_uml() -> Result<Option<VirtualizationType>> {
    Ok(BufReader::new(File::open("/proc/cpuinfo")?)
        .lines()
        .find_map(|x| Some(x.ok()?.strip_prefix("vendor_id\t: User Mode Linux")?.len()))
        .is_some()
        .then_some(VirtualizationType::UML))
}

fn detect_vm_zvm() -> Result<Option<VirtualizationType>> {
    Ok(match File::open("/proc/sysinfo") {
        Ok(f) => BufReader::new(f).lines().find_map(|x| {
            match x
                .ok()?
                .strip_prefix("VM00 Control Program")?
                .trim_start_matches(" \t")
                .strip_prefix(":")?
                .trim_start_matches(" \t")
                .trim_start_matches("0")
                .split_whitespace()
                .next()
            {
                Some(x) if x == "z/VM" => Some(VirtualizationType::ZVM),
                Some(_) => Some(VirtualizationType::KVM),
                None => None,
            }
        }),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => Err(e)?,
    })
}

pub fn detect_vm() -> Result<Option<VirtualizationType>> {
    let dmi = detect_vm_dmi();
    // eprintln!("dmi: {dmi:?}");
    match &dmi {
        Ok(Some(VirtualizationType::Oracle))
        | Ok(Some(VirtualizationType::Xen))
        | Ok(Some(VirtualizationType::Amazon))
        | Ok(Some(VirtualizationType::Parallels)) => {
            return dmi;
        }
        _ => {}
    }

    if let uml @ Some(_) = detect_vm_uml()? {
        // eprintln!("uml: {uml:?}");
        return Ok(uml);
    }

    let mut xen_dom0 = false;
    if let xen @ Some(VirtualizationType::Xen) = detect_vm_xen()? {
        // eprintln!("detected xen");
        xen_dom0 = detect_vm_xen_dom0()?;
        // eprintln!("xen dom0 is {xen_dom0}");
        if !xen_dom0 {
            return Ok(xen);
        }
    }

    let mut other = false;
    // eprintln!("CPUID");
    match detect_vm_cpuid()? {
        Some(VirtualizationType::Other) => {
            other = true;
            // eprintln!("other");
        }
        vm @ Some(_) => {
            // eprintln!("some: {vm:?}");
            return Ok(vm);
        }
        vm @ None if xen_dom0 => {
            // eprintln!("none with dom0: {vm:?} {xen_dom0:?}");
            return Ok(vm);
        }
        _ => {}
    }

    // eprintln!("dmi is {dmi:?}");
    match dmi? {
        Some(VirtualizationType::Other) => {
            other = true;
            // eprintln!("other")
        }
        dmi @ Some(_) => {
            // eprintln!("some: {dmi:?}");
            return Ok(dmi);
        }
        _ => {}
    }

    // eprintln!("hyper");
    match detect_vm_hypervisor()? {
        Some(VirtualizationType::Other) => {
            other = true;
            // eprintln!("other");
        }
        vm @ Some(_) => {
            // eprintln!("some: {vm:?}");
            return Ok(vm);
        }
        _ => {}
    }

    // eprintln!("devtree");
    match detect_vm_device_tree()? {
        Some(VirtualizationType::Other) => {
            other = true;
            // eprintln!("other");
        }
        vm @ Some(_) => {
            // eprintln!("some: {vm:?}");
            return Ok(vm);
        }
        _ => {}
    }

    if let zvm @ Some(_) = detect_vm_zvm()? {
        // eprintln!("some zvm: {zvm:?}");
        return Ok(zvm);
    }

    // eprintln!("other {other:?} then some VT-Other");
    Ok(other.then_some(VirtualizationType::Other))
}

pub enum ContainerType {
    SystemdNspawn,
    LxcLibvirt,
    Lxc,
    OpenVZ,
    Docker,
    Podman,
    RKT,
    WSL,
    Proot,
    Pouch,
    Other,
}

pub fn detect_container() -> Result<Option<ContainerType>> {
    // TODO detect_container
    Ok(None)
}
