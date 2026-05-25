use std::path::Path;

use anyhow::Result;
use serde::Serialize;

const SECTOR_SIZE: usize = 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Flash,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationRisk {
    ReadOnly,
    Write,
    Destructive,
}

#[derive(Debug, Clone, Serialize)]
pub struct FirmwarePlanInfo {
    pub path: String,
    pub original_size: usize,
    pub padded_size: usize,
    pub sector_size: usize,
    pub sectors_to_erase: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanDeviceInfo {
    pub name: String,
    pub chip_id: String,
    pub flash_size: u32,
    pub eeprom_size: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperationPlanStep {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub risk: OperationRisk,
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperationPlan {
    pub ok: bool,
    pub operation: OperationKind,
    pub apply_ready: bool,
    pub transport: String,
    pub device: Option<PlanDeviceInfo>,
    pub firmware: FirmwarePlanInfo,
    pub steps: Vec<OperationPlanStep>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn plan_flash_from_file(path: &Path) -> Result<OperationPlan> {
    plan_flash_from_file_with_device(path, None)
}

pub fn plan_flash_from_file_with_device(
    path: &Path,
    device: Option<PlanDeviceInfo>,
) -> Result<OperationPlan> {
    let data = wchisp::format::read_firmware_from_file(path)?;
    let original_size = data.len();
    let padded_size = padded_size(original_size);
    let sectors_to_erase = padded_size / SECTOR_SIZE + 1;
    let mut warnings = Vec::new();
    let mut blockers = Vec::new();

    if original_size != padded_size {
        warnings.push(format!(
            "Firmware will be padded from {original_size} to {padded_size} bytes with 0xFF."
        ));
    }

    match &device {
        Some(device) if padded_size > device.flash_size as usize => blockers.push(format!(
            "Firmware padded size {padded_size} exceeds {} flash size {}.",
            device.name, device.flash_size
        )),
        Some(_) => {}
        None => {
            blockers.push("No live device validation has been attached to this plan yet.".into())
        }
    }

    Ok(OperationPlan {
        ok: true,
        operation: OperationKind::Flash,
        apply_ready: blockers.is_empty(),
        transport: "usb:auto".into(),
        device,
        firmware: FirmwarePlanInfo {
            path: path.display().to_string(),
            original_size,
            padded_size,
            sector_size: SECTOR_SIZE,
            sectors_to_erase,
        },
        steps: vec![
            step(
                "connect",
                "Connect",
                "Open the selected WCH ISP transport.",
                OperationRisk::ReadOnly,
                true,
            ),
            step(
                "identify",
                "Identify chip",
                "Read chip id, flash size and data flash size before writes.",
                OperationRisk::ReadOnly,
                true,
            ),
            step(
                "validate",
                "Validate firmware",
                "Compare firmware size and target capacity after the chip is known.",
                OperationRisk::ReadOnly,
                true,
            ),
            step(
                "erase",
                "Erase code flash",
                &format!("Erase {sectors_to_erase} sector(s) before programming."),
                OperationRisk::Destructive,
                true,
            ),
            step(
                "program",
                "Program firmware",
                &format!("Write {padded_size} bytes to code flash."),
                OperationRisk::Write,
                true,
            ),
            step(
                "verify",
                "Verify firmware",
                "Read back and compare programmed bytes.",
                OperationRisk::ReadOnly,
                true,
            ),
            step(
                "reset",
                "Reset target",
                "Reset the chip after a successful verify.",
                OperationRisk::ReadOnly,
                false,
            ),
        ],
        blockers,
        warnings,
    })
}

pub fn plan_json_pretty(plan: &OperationPlan) -> Result<String> {
    Ok(serde_json::to_string_pretty(plan)?)
}

fn padded_size(size: usize) -> usize {
    let remain = size % SECTOR_SIZE;
    if remain == 0 {
        size
    } else {
        size + SECTOR_SIZE - remain
    }
}

fn step(
    id: &str,
    title: &str,
    detail: &str,
    risk: OperationRisk,
    blocking: bool,
) -> OperationPlanStep {
    OperationPlanStep {
        id: id.into(),
        title: title.into(),
        detail: detail.into(),
        risk,
        blocking,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_capacity_controls_apply_readiness() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "meowisp-plan-test-{}-{}.bin",
            std::process::id(),
            "capacity"
        ));
        std::fs::write(&path, [0u8; 1500]).expect("write firmware fixture");

        let small = PlanDeviceInfo {
            name: "TinyCH".into(),
            chip_id: "0x01".into(),
            flash_size: 1024,
            eeprom_size: 0,
        };
        let plan = plan_flash_from_file_with_device(&path, Some(small)).expect("plan");
        assert!(!plan.apply_ready);
        assert_eq!(plan.firmware.padded_size, 2048);
        assert_eq!(plan.blockers.len(), 1);

        let large = PlanDeviceInfo {
            name: "RoomyCH".into(),
            chip_id: "0x02".into(),
            flash_size: 4096,
            eeprom_size: 0,
        };
        let plan = plan_flash_from_file_with_device(&path, Some(large)).expect("plan");
        assert!(plan.apply_ready);
        assert!(plan.blockers.is_empty());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn padded_size_uses_sector_boundary() {
        assert_eq!(padded_size(0), 0);
        assert_eq!(padded_size(1), 1024);
        assert_eq!(padded_size(1024), 1024);
        assert_eq!(padded_size(1025), 2048);
    }
}
