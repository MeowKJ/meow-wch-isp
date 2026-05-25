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
    pub firmware: FirmwarePlanInfo,
    pub steps: Vec<OperationPlanStep>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn plan_flash_from_file(path: &Path) -> Result<OperationPlan> {
    let data = wchisp::format::read_firmware_from_file(path)?;
    let original_size = data.len();
    let padded_size = padded_size(original_size);
    let sectors_to_erase = padded_size / SECTOR_SIZE + 1;
    let mut warnings = Vec::new();

    if original_size != padded_size {
        warnings.push(format!(
            "Firmware will be padded from {original_size} to {padded_size} bytes with 0xFF."
        ));
    }

    Ok(OperationPlan {
        ok: true,
        operation: OperationKind::Flash,
        apply_ready: false,
        transport: "usb:auto".into(),
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
        blockers: vec!["No live device validation has been attached to this plan yet.".into()],
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
    fn padded_size_uses_sector_boundary() {
        assert_eq!(padded_size(0), 0);
        assert_eq!(padded_size(1), 1024);
        assert_eq!(padded_size(1024), 1024);
        assert_eq!(padded_size(1025), 2048);
    }
}
