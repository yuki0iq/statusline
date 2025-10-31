use crate::file;
use anyhow::Result;
use std::{
    fs::File,
    io::{BufRead as _, BufReader, ErrorKind},
    path::Path,
};

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
fn detect_vm_cpuid() -> Option<bool> {
    let cpuid = raw_cpuid::CpuId::new();
    let vendor_info = cpuid.get_vendor_info()?;
    match vendor_info.as_str() {
        // Known to only belong to hypervisors
        "XenVMMXenVMM" | "KVMKVMKVM" | "Linux KVM Hv" | "TCGTCGTCGTCG" | "VMwareVMware"
        | "Microsoft Hv" | "bhyve bhyve " | "QNXQVMBSQG" | "ACRNACRNACRN" | "SRESRESRESRE"
        | "MicrosoftXTA" | "VirtualApple" | "PowerVM Lx86" | "Neko Project" => Some(true),

        // Known to only belong to hardware
        "GenuineIntel" | "AuthenticAMD" | "CentaurHauls" | "CyrixInstead" | "GenuineIotel"
        | "TransmetaCPU" | "GenuineTMx86" | "Geode by NSC" | "NexGenDriven" | "RiseRiseRise"
        | "SiS SiS SiS " | "UMC UMC UMC " | "Vortex86 SoC" | "  Shanghai  " | "HygonGenuine"
        | "Genuine  RDC" | "E2K MACHINE" | "VIA VIA VIA " | "AMD ISBETTER" => Some(false),

        other => {
            eprintln!("CPUID returned unknown vendor info: {other:?}");
            None
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
fn detect_vm_cpuid() -> Option<bool> {
    None
}

fn detect_vm_device_tree() -> Result<Option<bool>> {
    if std::fs::exists("/proc/device-tree/hypervisor/compatible")? {
        return Ok(Some(true));
    }

    if std::fs::exists("/proc/device-tree/ibm,partition-name")?
        && std::fs::exists("/proc/device-tree/hmc-managed?")?
        && !std::fs::exists("/proc/device-tree/chosen/qemu,graphic-width")?
    {
        return Ok(Some(true));
    }

    match file::exists_that("/proc/device-tree", |name| name.contains("fw-cfg")) {
        Ok(true) => return Ok(Some(true)),
        Ok(false) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    }

    match std::fs::read_to_string("/proc/device-tree/compatible") {
        Ok(s) if s == "qemu,pseries" => Ok(Some(true)),
        Ok(_) => Ok(Some(false)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn detect_vm_dmi_vendor_path(path: &Path) -> Result<bool> {
    let name = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.into()),
    };

    for vendor in [
        "KVM",
        "OpenStack",
        "KubeVirt",
        "Amazon EC2",
        "QEMU",
        "VMware",
        "VMW",
        "innotek GmbH",
        "VirtualBox",
        "Xen",
        "Bochs",
        "Parallels",
        "BHYVE",
        "Hyper-V",
        "Apple Virtualization",
    ] {
        if name.starts_with(vendor) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn detect_vm_dmi_vendor() -> Result<bool> {
    for path in [
        "/sys/class/dmi/id/product_name",
        "/sys/class/dmi/id/sys_vendor",
        "/sys/class/dmi/id/board_vendor",
        "/sys/class/dmi/id/bios_vendor",
        "/sys/class/dmi/id/product_version",
    ] {
        if detect_vm_dmi_vendor_path(path.as_ref())? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn detect_vm_smbios() -> Result<bool> {
    // See 7.1.2.2 "BIOS Characteristics Extension Byte 2" at [SMBIOS spec]
    // [SMBIOS spec]: https://www.dmtf.org/sites/default/files/standards/documents/DSP0134_3.4.0.pdf
    Ok(std::fs::read("/sys/firmware/dmi/entries/0-0/raw")?
        .get(0x13)
        .is_some_and(|x| ((x >> 4_i32) & 1) == 1))
}

fn detect_vm_dmi_metal() -> Result<bool> {
    let name = std::fs::read_to_string("/sys/class/dmi/id/product_name")?;
    Ok(name.contains(".metal-") || name.ends_with(".metal"))
}

fn detect_vm_xen_dom0() -> Result<bool> {
    match std::fs::read_to_string("/sys/hypervisor/properties/features") {
        Ok(s) => return Ok((u64::from_str_radix(&s, 16)? >> 11) & 1 == 1),
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }

    match std::fs::read_to_string("/proc/xen/capabilities") {
        Ok(s) => Ok(s.contains("control_d")),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

pub fn detect_vm() -> Result<bool> {
    if let Ok(file) = File::open("/proc/cpuinfo")
        && BufReader::new(file)
            .lines()
            .map_while(Result::ok)
            .any(|line| line.starts_with("vendor_id\t: User Mode Linux"))
    {
        return Ok(true);
    }

    if let Some(res) = detect_vm_cpuid() {
        return Ok(res);
    }

    if std::fs::exists("/proc/xen")? && !detect_vm_xen_dom0()?
        || std::fs::exists("/sys/hypervisor/type")?
        || std::fs::exists("/proc/sysinfo")?
        || detect_vm_smbios()?
        || detect_vm_dmi_vendor()?
        || detect_vm_dmi_metal()?
    {
        return Ok(true);
    }

    if let Some(res) = detect_vm_device_tree()? {
        return Ok(res);
    }

    Ok(false)
}

pub fn detect_container() -> Result<bool> {
    if std::fs::exists("/proc/vz")? && !std::fs::exists("/proc/bc")?
        || std::fs::exists("/run/host/container-daemon")?
        || std::fs::exists("/run/systemd/container")?
        || std::fs::exists("/run/.containerenv")?
        || std::fs::exists("/.dockerenv")?
    {
        return Ok(true);
    }

    if let Ok(s) = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        && (s.contains("Microsoft") || s.contains("WSL"))
    {
        return Ok(true);
    }

    if let Ok(file) = File::open("/proc/self/status")
        && let Some(pid) = BufReader::new(file).lines().find_map(|line| {
            line.ok()?
                .strip_prefix("TracerPid:\t")?
                .parse::<usize>()
                .ok()
        })
        && let Ok(s) = std::fs::read_to_string(format!("/proc/{pid}/comm"))
        && s.starts_with("proot")
    {
        return Ok(true);
    }

    if let Ok(file) = File::open("/proc/1/environ")
        && BufReader::new(file)
            .split(0)
            .map_while(Result::ok)
            .any(|line| line.starts_with(b"container="))
    {
        return Ok(true);
    }

    Ok(false)
}
