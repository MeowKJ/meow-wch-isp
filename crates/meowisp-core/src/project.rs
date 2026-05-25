use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::catalog::{self, DeviceVariantInfo};
use crate::plan::{self, OperationPlan, PlanDeviceInfo};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectFile {
    pub project: ProjectSection,
    pub target: TargetSection,
    pub firmware: FirmwareSection,
    #[serde(default)]
    pub config: ConfigSection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectSection {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TargetSection {
    pub chip: String,
    #[serde(default)]
    pub transport: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FirmwareSection {
    pub path: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub verify: Option<bool>,
    #[serde(default)]
    pub reset_after_flash: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConfigSection {
    #[serde(default = "default_config_mode")]
    pub mode: String,
    #[serde(default)]
    pub bits: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectPlan {
    pub ok: bool,
    pub operation: String,
    pub project_file: String,
    pub project: ProjectSummary,
    pub target: TargetPlan,
    pub firmware: ProjectFirmwarePlan,
    pub config: ProjectConfigPlan,
    pub flash_plan: Option<OperationPlan>,
    pub apply_ready: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectSummary {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TargetPlan {
    pub chip: String,
    pub transport: String,
    pub matched: bool,
    pub family: Option<String>,
    pub device_type: Option<String>,
    pub chip_id: Option<String>,
    pub flash_size: Option<u32>,
    pub eeprom_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectFirmwarePlan {
    pub path: String,
    pub resolved_path: String,
    pub format: String,
    pub exists: bool,
    pub verify: bool,
    pub reset_after_flash: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectConfigPlan {
    pub mode: String,
    pub requested_bit_count: usize,
    pub bits: Vec<ProjectConfigBitPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectConfigBitPlan {
    pub name: String,
    pub value: toml::Value,
    pub known: bool,
    pub register: Option<String>,
    pub bit_range: Option<Vec<u8>>,
}

pub fn load_project(path: &Path) -> Result<ProjectFile> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read project file {}", path.display()))?;
    toml::from_str(&text)
        .with_context(|| format!("failed to parse project file {}", path.display()))
}

pub fn plan_project_from_file(path: &Path) -> Result<ProjectPlan> {
    let project = load_project(path)?;
    plan_project(path, project)
}

pub fn project_plan_json_pretty(plan: &ProjectPlan) -> Result<String> {
    Ok(serde_json::to_string_pretty(plan)?)
}

fn plan_project(path: &Path, project: ProjectFile) -> Result<ProjectPlan> {
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let firmware_path = resolve_project_path(base_dir, &project.firmware.path);
    let catalog = catalog::load_catalog().context("failed to load device catalog")?;
    let target_match = find_variant(&catalog, &project.target.chip);
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    let target = match &target_match {
        Some((family, variant)) => TargetPlan {
            chip: project.target.chip.clone(),
            transport: project
                .target
                .transport
                .clone()
                .unwrap_or_else(|| "usb".into()),
            matched: true,
            family: Some(family.name.clone()),
            device_type: Some(family.device_type_hex.clone()),
            chip_id: Some(variant.chip_id_hex.clone()),
            flash_size: Some(variant.flash_size),
            eeprom_size: Some(variant.eeprom_size),
        },
        None => {
            blockers.push(format!(
                "Target chip {} was not found in the device catalog.",
                project.target.chip
            ));
            TargetPlan {
                chip: project.target.chip.clone(),
                transport: project
                    .target
                    .transport
                    .clone()
                    .unwrap_or_else(|| "usb".into()),
                matched: false,
                family: None,
                device_type: None,
                chip_id: None,
                flash_size: None,
                eeprom_size: None,
            }
        }
    };

    let firmware_exists = firmware_path.exists();
    if !firmware_exists {
        blockers.push(format!(
            "Firmware file {} does not exist.",
            firmware_path.display()
        ));
    }

    let firmware = ProjectFirmwarePlan {
        path: project.firmware.path.clone(),
        resolved_path: firmware_path.display().to_string(),
        format: project
            .firmware
            .format
            .clone()
            .unwrap_or_else(|| "bin".into()),
        exists: firmware_exists,
        verify: project.firmware.verify.unwrap_or(true),
        reset_after_flash: project.firmware.reset_after_flash.unwrap_or(true),
    };

    if firmware.format != "bin" {
        blockers.push(format!(
            "Firmware format {} is not supported yet; use bin.",
            firmware.format
        ));
    }

    let config = build_config_plan(
        &project.config,
        target_match.as_ref().map(|(_, variant)| *variant),
    );
    for bit in &config.bits {
        if !bit.known {
            blockers.push(format!(
                "Config bit {} is not known for target {}.",
                bit.name, project.target.chip
            ));
        }
    }

    let flash_plan = if firmware_exists && firmware.format == "bin" {
        let device = target_match.as_ref().map(|(_, variant)| PlanDeviceInfo {
            name: variant.name.clone(),
            chip_id: variant.chip_id_hex.clone(),
            flash_size: variant.flash_size,
            eeprom_size: variant.eeprom_size,
        });
        match plan::plan_flash_from_file_with_device(&firmware_path, device) {
            Ok(plan) => {
                warnings.extend(plan.warnings.clone());
                blockers.extend(plan.blockers.clone());
                Some(plan)
            }
            Err(err) => {
                blockers.push(format!("Failed to build firmware plan: {err}"));
                None
            }
        }
    } else {
        None
    };

    let apply_ready =
        blockers.is_empty() && flash_plan.as_ref().is_some_and(|plan| plan.apply_ready);

    Ok(ProjectPlan {
        ok: true,
        operation: "project.plan".into(),
        project_file: path.display().to_string(),
        project: ProjectSummary {
            name: project.project.name,
        },
        target,
        firmware,
        config,
        flash_plan,
        apply_ready,
        blockers,
        warnings,
    })
}

fn resolve_project_path(base_dir: &Path, path: &str) -> PathBuf {
    let input = Path::new(path);
    if input.is_absolute() {
        input.to_path_buf()
    } else {
        base_dir.join(input)
    }
}

fn find_variant<'a>(
    catalog: &'a catalog::DeviceCatalog,
    chip: &str,
) -> Option<(&'a catalog::DeviceFamilyInfo, &'a DeviceVariantInfo)> {
    catalog.families.iter().find_map(|family| {
        family
            .variants
            .iter()
            .find(|variant| variant.name.eq_ignore_ascii_case(chip))
            .map(|variant| (family, variant))
    })
}

fn build_config_plan(
    config: &ConfigSection,
    variant: Option<&DeviceVariantInfo>,
) -> ProjectConfigPlan {
    let known_fields = variant.map(known_config_fields).unwrap_or_default();
    let bits = config
        .bits
        .iter()
        .map(|(name, value)| {
            let field = known_fields.get(name);
            ProjectConfigBitPlan {
                name: name.clone(),
                value: value.clone(),
                known: field.is_some(),
                register: field.map(|field| field.0.clone()),
                bit_range: field.map(|field| field.1.clone()),
            }
        })
        .collect();

    ProjectConfigPlan {
        mode: config.mode.clone(),
        requested_bit_count: config.bits.len(),
        bits,
    }
}

fn known_config_fields(variant: &DeviceVariantInfo) -> BTreeMap<String, (String, Vec<u8>)> {
    let mut fields = BTreeMap::new();
    let mut seen = BTreeSet::new();

    for register in &variant.config_registers {
        for field in &register.fields {
            if seen.insert(field.name.clone()) {
                fields.insert(
                    field.name.clone(),
                    (register.name.clone(), field.bit_range.clone()),
                );
            }
        }
    }

    fields
}

fn default_config_mode() -> String {
    "check".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_plan_blocks_missing_firmware() {
        let path = std::env::temp_dir().join(format!(
            "meowisp-project-missing-{}-{}.toml",
            std::process::id(),
            "firmware"
        ));
        std::fs::write(
            &path,
            r#"
[project]
name = "Missing firmware"

[target]
chip = "CH592"

[firmware]
path = "missing.bin"

[config.bits]
CFG_DEBUG_EN = true
"#,
        )
        .expect("write project fixture");

        let plan = plan_project_from_file(&path).expect("project plan");
        assert!(plan.target.matched);
        assert!(!plan.firmware.exists);
        assert!(!plan.apply_ready);
        assert!(plan
            .blockers
            .iter()
            .any(|blocker| blocker.contains("does not exist")));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn project_plan_validates_against_catalog_chip_capacity() {
        let dir = std::env::temp_dir();
        let stem = format!("meowisp-project-ready-{}", std::process::id());
        let project_path = dir.join(format!("{stem}.toml"));
        let firmware_path = dir.join(format!("{stem}.bin"));
        std::fs::write(&firmware_path, [0x5Au8; 2048]).expect("write firmware fixture");
        std::fs::write(
            &project_path,
            format!(
                r#"
[project]
name = "Ready firmware"

[target]
chip = "CH592"
transport = "usb"

[firmware]
path = "{}"
format = "bin"
verify = true
reset_after_flash = true

[config.bits]
CFG_DEBUG_EN = true
"#,
                firmware_path.display()
            ),
        )
        .expect("write project fixture");

        let plan = plan_project_from_file(&project_path).expect("project plan");
        assert!(plan.target.matched);
        assert!(plan.firmware.exists);
        assert!(plan.apply_ready);
        assert!(plan.flash_plan.is_some());
        assert!(plan.config.bits.iter().all(|bit| bit.known));

        let _ = std::fs::remove_file(project_path);
        let _ = std::fs::remove_file(firmware_path);
    }
}
