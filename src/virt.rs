use crate::file;
use anyhow::Result;
use std::{
    fs::{self, File},
    io::{BufRead as _, BufReader, Error as IoError, ErrorKind},
};

#[derive(Debug)]
pub enum VirtualizationType {
    Kvm,
    Amazon,
    Qemu,
    Bochs,
    Xen,
    Uml,
    VMware,
    Oracle,
    Microsoft,
    Zvm,
    Parallels,
    Bhyve,
    Qnx,
    Acrn,
    PowerVM,
    Apple,
    Sre,
    Other,
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
fn detect_vm_cpuid() -> Option<VirtualizationType> {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{__cpuid, __get_cpuid_max};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{__cpuid, __get_cpuid_max};

    // Check leaf 1 accessible
    let maxlevel = unsafe { __get_cpuid_max(0).0 };
    if maxlevel == 0 {
        return None;
    }

    // Get leaf 1, ecx, bit 31 -- Hypervisor bit
    let ecx = unsafe { __cpuid(1) }.ecx;
    let hv = ecx & 0x8000_0000;
    if hv == 0 {
        return None;
    }

    // Hypervisor bit ON ->check leaf 0x40000000 and use (ebx, ecx, edx) as string
    let cpuid_res = unsafe { __cpuid(0x4000_0000) };
    let vendor = [
        cpuid_res.ebx.to_le_bytes(),
        cpuid_res.ecx.to_le_bytes(),
        cpuid_res.edx.to_le_bytes(),
    ]
    .concat();

    Some(match &vendor[..12] {
        b"XenVMMXenVMM" => VirtualizationType::Xen,
        b"KVMKVMKVM" | b"Linux KVM Hv" => VirtualizationType::Kvm,
        b"TCGTCGTCGTCG" => VirtualizationType::Qemu,
        b"VMwareVMware" => VirtualizationType::VMware,
        b"Microsoft Hv" => VirtualizationType::Microsoft,
        b"bhyve bhyve " => VirtualizationType::Bhyve,
        b"QNXQVMBSQG" => VirtualizationType::Qnx,
        b"ACRNACRNACRN" => VirtualizationType::Acrn,
        b"SRESRESRESRE" => VirtualizationType::Sre,
        _ => VirtualizationType::Other,
    })
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
fn detect_vm_cpuid() -> Option<VirtualizationType> {
    None
}

#[expect(clippy::shadow_unrelated)] // send help please
fn detect_vm_device_tree() -> Result<Option<VirtualizationType>> {
    Ok(
        match fs::read_to_string("/proc/device-tree/hypervisor/compatible") {
            Ok(s) if s == "linux,kvm" => Some(VirtualizationType::Kvm),
            Ok(s) if s.contains("xen") => Some(VirtualizationType::Xen),
            Ok(s) if s.contains("vmware") => Some(VirtualizationType::VMware),
            Ok(_) => Some(VirtualizationType::Other),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                if fs::exists("/proc/device-tree/ibm,partition-name").unwrap_or(false)
                    && fs::exists("/proc/device-tree/hmc-managed?").unwrap_or(false)
                    && !fs::exists("/proc/device-tree/chosen/qemu,graphic-width").unwrap_or(false)
                {
                    Some(VirtualizationType::PowerVM)
                } else {
                    match file::exists_that("/proc/device-tree", |name| name.contains("fw-cfg")) {
                        Ok(true) => Some(VirtualizationType::Qemu),
                        Ok(false) => match fs::read_to_string("/proc/device-tree/compatible") {
                            Ok(s) if s == "qemu,pseries" => Some(VirtualizationType::Qemu),
                            Ok(_) => None,
                            Err(e) if e.kind() == ErrorKind::NotFound => None,
                            Err(e) => return Err(e.into()),
                        },
                        Err(e)
                            if e.is::<IoError>()
                                && e.downcast_ref::<IoError>().unwrap().kind()
                                    == ErrorKind::NotFound =>
                        {
                            None
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
            Err(e) => return Err(e.into()),
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
                    ("KVM", VirtualizationType::Kvm),
                    ("OpenStack", VirtualizationType::Kvm),
                    ("KubeVirt", VirtualizationType::Kvm),
                    ("Amazon EC2", VirtualizationType::Amazon),
                    ("QEMU", VirtualizationType::Qemu),
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
            Err(e) => return Err(e.into()),
        }
    }
    Ok(None)
}

fn detect_vm_smbios_impl() -> Result<Option<bool>> {
    Ok(fs::read("/sys/firmware/dmi/entries/0-0/raw")?
        .get(19)
        .map(|x| ((x >> 4_i32) & 1) == 1))
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
                        #[expect(clippy::case_sensitive_file_extension_comparisons)]
                        (s.contains(".metal-") || s.ends_with(".metal")) // TODO
                            .then_some(VirtualizationType::Amazon)
                    }
                    Err(_) => Some(VirtualizationType::Amazon),
                }
            }
        },
        None if detect_vm_smbios().unwrap_or(false) => Some(VirtualizationType::Other),
        vm => vm,
    })
}

#[expect(clippy::shadow_unrelated)] // send help please
fn detect_vm_xen_dom0() -> Result<bool> {
    Ok(
        match fs::read_to_string("/sys/hypervisor/properties/features") {
            Ok(s) => (u64::from_str_radix(&s, 16)? >> 11) & 1 == 1,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                match fs::read_to_string("/proc/xen/capabilities") {
                    Ok(s) => s.contains("control_d"),
                    Err(e) if e.kind() == ErrorKind::NotFound => false,
                    Err(e) => return Err(e.into()),
                }
            }
            Err(e) => return Err(e.into()),
        },
    )
}

fn detect_vm_xen() -> Result<Option<VirtualizationType>> {
    Ok(fs::exists("/proc/xen")?.then_some(VirtualizationType::Xen))
}

fn detect_vm_hypervisor() -> Result<Option<VirtualizationType>> {
    Ok(match fs::read_to_string("/sys/hypervisor/type") {
        Ok(s) if s == "xen" => Some(VirtualizationType::Xen),
        Ok(_) => Some(VirtualizationType::Other),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    })
}

fn detect_vm_uml() -> Result<Option<VirtualizationType>> {
    Ok(BufReader::new(File::open("/proc/cpuinfo")?)
        .lines()
        .find_map(|x| Some(x.ok()?.strip_prefix("vendor_id\t: User Mode Linux")?.len()))
        .is_some()
        .then_some(VirtualizationType::Uml))
}

fn detect_vm_zvm() -> Result<Option<VirtualizationType>> {
    Ok(match File::open("/proc/sysinfo") {
        Ok(f) => BufReader::new(f).lines().find_map(|x| {
            match x
                .ok()?
                .strip_prefix("VM00 Control Program")?
                .trim_start_matches(" \t")
                .strip_prefix(':')?
                .trim_start_matches(" \t")
                .trim_start_matches('0')
                .split_whitespace()
                .next()
            {
                Some("z/VM") => Some(VirtualizationType::Zvm),
                Some(_) => Some(VirtualizationType::Kvm),
                None => None,
            }
        }),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    })
}

pub fn detect_vm() -> Result<Option<VirtualizationType>> {
    let dmi = detect_vm_dmi();
    // eprintln!("dmi: {dmi:?}");
    if let Ok(Some(
        VirtualizationType::Oracle
        | VirtualizationType::Xen
        | VirtualizationType::Amazon
        | VirtualizationType::Parallels,
    )) = &dmi
    {
        return dmi;
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
    match detect_vm_cpuid() {
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
    Rkt,
    Wsl,
    Proot,
    Pouch,
    Other,
}

fn running_in_cgroupns() -> Result<bool> {
    if fs::exists("/proc/self/ns/cgroup").is_err() {
        return Ok(false);
    }

    // Only cgroup v2 is supported right now, so no check if it _is_ v2 is needed
    Ok(fs::exists("/sys/fs/cgroup/cgroup.events")?
        && (fs::exists("/sys/fs/cgroup/cgroup.type")?
            || !fs::exists("/sys/kernel/cgroup/features")?))
}

fn detect_container_files() -> Option<ContainerType> {
    if let Ok(true) = fs::exists("/run/.containerenv") {
        return Some(ContainerType::Podman);
    }
    if let Ok(true) = fs::exists("/.dockerenv") {
        return Some(ContainerType::Docker);
    }
    None
}

fn translate_name(name: &str) -> ContainerType {
    match name {
        "oci" => detect_container_files().unwrap_or(ContainerType::Other),
        "lxc" => ContainerType::Lxc,
        "lxc-libvirt" => ContainerType::LxcLibvirt,
        "systemd-nspawn" => ContainerType::SystemdNspawn,
        "docker" => ContainerType::Docker,
        "podman" => ContainerType::Podman,
        "rkt" => ContainerType::Rkt,
        "wsl" => ContainerType::Wsl,
        "proot" => ContainerType::Proot,
        "pouch" => ContainerType::Pouch,
        _ => ContainerType::Other,
    }
}

pub fn detect_container() -> Result<Option<ContainerType>> {
    if let (Ok(true), Ok(false)) = (fs::exists("/proc/vz"), fs::exists("/proc/bc")) {
        return Ok(Some(ContainerType::OpenVZ));
    }

    if let Ok(s) = fs::read_to_string("/proc/sys/kernel/osrelease")
        && (s.contains("Microsoft") || s.contains("WSL"))
    {
        return Ok(Some(ContainerType::Wsl));
    }

    if let Ok(file) = File::open("/proc/self/status") {
        if let Some(pid) = BufReader::new(file).lines().find_map(|line| {
            line.ok()?
                .strip_prefix("TracerPid:\t")?
                .split_whitespace()
                .map(|x| x.parse::<usize>().ok())
                .next()?
        }) {
            if let Ok(s) = fs::read_to_string(format!("/proc/{pid}/comm"))
                && s.starts_with("proot")
            {
                return Ok(Some(ContainerType::Proot));
            }
        }
    }

    match fs::read_to_string("/run/host/container-daemon") {
        Ok(s) => {
            return Ok(Some(translate_name(&s)));
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }

    match fs::read_to_string("/run/systemd/container") {
        Ok(s) => {
            return Ok(Some(translate_name(&s)));
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }

    if let Ok(file) = File::open("/proc/1/environ") {
        if let Some(name) = BufReader::new(file).split(0).find_map(|line| {
            String::from_utf8(line.ok()?.strip_prefix(b"container=")?.to_vec()).ok()
        }) {
            return Ok(Some(translate_name(&name)));
        }
    }

    if let ct @ Some(_) = detect_container_files() {
        return Ok(ct);
    }

    if let Ok(true) = running_in_cgroupns() {
        return Ok(Some(ContainerType::Other));
    }

    Ok(None)
}
