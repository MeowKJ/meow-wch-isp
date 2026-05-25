use anyhow::Result;
use serde::Serialize;
use wchisp::device::{ChipDB, ChipFamily, ConfigRegister, RegisterField};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Supported,
    Unsupported,
    Unknown,
}

impl From<Option<bool>> for CapabilityState {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => CapabilityState::Supported,
            Some(false) => CapabilityState::Unsupported,
            None => CapabilityState::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TransportCapabilities {
    pub usb: CapabilityState,
    pub serial: CapabilityState,
    pub net: CapabilityState,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryRegionInfo {
    pub name: String,
    pub kind: String,
    pub start: Option<u32>,
    pub size: u32,
    pub readable: CapabilityState,
    pub writable: CapabilityState,
    pub erasable: CapabilityState,
    pub dangerous: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigFieldInfo {
    pub name: String,
    pub bit_range: Vec<u8>,
    pub description: String,
    pub values: Vec<ConfigValueInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigValueInfo {
    pub value: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigRegisterInfo {
    pub offset: usize,
    pub name: String,
    pub description: String,
    pub reset: Option<u32>,
    pub fields: Vec<ConfigFieldInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceVariantInfo {
    pub name: String,
    pub chip_id: u8,
    pub chip_id_hex: String,
    pub alt_chip_ids: Vec<u8>,
    pub flash_size: u32,
    pub eeprom_size: u32,
    pub eeprom_start_addr: Option<u32>,
    pub transports: TransportCapabilities,
    pub memory_regions: Vec<MemoryRegionInfo>,
    pub config_register_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceFamilyInfo {
    pub name: String,
    pub mcu_type: u8,
    pub device_type: u8,
    pub device_type_hex: String,
    pub description: String,
    pub transports: TransportCapabilities,
    pub config_registers: Vec<ConfigRegisterInfo>,
    pub variants: Vec<DeviceVariantInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceCatalog {
    pub source: String,
    pub family_count: usize,
    pub variant_count: usize,
    pub families: Vec<DeviceFamilyInfo>,
}

pub fn load_catalog() -> Result<DeviceCatalog> {
    let db = ChipDB::load()?;
    let families: Vec<DeviceFamilyInfo> = db.families.iter().map(family_info).collect();
    let variant_count = families.iter().map(|family| family.variants.len()).sum();

    Ok(DeviceCatalog {
        source: "vendor-wchisp/devices".into(),
        family_count: families.len(),
        variant_count,
        families,
    })
}

pub fn catalog_json_pretty() -> Result<String> {
    Ok(serde_json::to_string_pretty(&load_catalog()?)?)
}

fn family_info(family: &ChipFamily) -> DeviceFamilyInfo {
    let transports = TransportCapabilities {
        usb: family.support_usb().into(),
        serial: family.support_serial().into(),
        net: family.support_net().into(),
    };
    let config_registers: Vec<ConfigRegisterInfo> = family
        .config_registers
        .iter()
        .map(config_register_info)
        .collect();
    let variants = family
        .variants
        .iter()
        .map(|variant| {
            let variant_usb = variant.support_usb().or(family.support_usb());
            let variant_serial = variant.support_serial().or(family.support_serial());
            let variant_net = variant.support_net().or(family.support_net());
            let config_count = if variant.config_registers.is_empty() {
                family.config_registers.len()
            } else {
                variant.config_registers.len()
            };

            DeviceVariantInfo {
                name: variant.name.clone(),
                chip_id: variant.chip_id,
                chip_id_hex: format!("0x{:02X}", variant.chip_id),
                alt_chip_ids: variant.alt_chip_ids().to_vec(),
                flash_size: variant.flash_size,
                eeprom_size: variant.eeprom_size,
                eeprom_start_addr: (variant.eeprom_size > 0).then_some(variant.eeprom_start_addr),
                transports: TransportCapabilities {
                    usb: variant_usb.into(),
                    serial: variant_serial.into(),
                    net: variant_net.into(),
                },
                memory_regions: memory_regions(
                    variant.flash_size,
                    variant.eeprom_size,
                    variant.eeprom_start_addr,
                ),
                config_register_count: config_count,
            }
        })
        .collect();

    DeviceFamilyInfo {
        name: family.name.clone(),
        mcu_type: family.mcu_type,
        device_type: family.device_type,
        device_type_hex: format!("0x{:02X}", family.device_type),
        description: family.description.clone(),
        transports,
        config_registers,
        variants,
    }
}

fn memory_regions(
    flash_size: u32,
    eeprom_size: u32,
    eeprom_start_addr: u32,
) -> Vec<MemoryRegionInfo> {
    let mut regions = vec![MemoryRegionInfo {
        name: "Code Flash".into(),
        kind: "code_flash".into(),
        start: Some(0),
        size: flash_size,
        readable: CapabilityState::Unknown,
        writable: CapabilityState::Supported,
        erasable: CapabilityState::Supported,
        dangerous: true,
    }];

    if eeprom_size > 0 {
        regions.push(MemoryRegionInfo {
            name: "Data Flash / EEPROM".into(),
            kind: "data_flash".into(),
            start: Some(eeprom_start_addr),
            size: eeprom_size,
            readable: CapabilityState::Supported,
            writable: CapabilityState::Supported,
            erasable: CapabilityState::Supported,
            dangerous: true,
        });
    }

    regions
}

fn config_register_info(register: &ConfigRegister) -> ConfigRegisterInfo {
    ConfigRegisterInfo {
        offset: register.offset,
        name: register.name.clone(),
        description: register.description().to_string(),
        reset: register.reset,
        fields: register.fields.iter().map(config_field_info).collect(),
    }
}

fn config_field_info(field: &RegisterField) -> ConfigFieldInfo {
    ConfigFieldInfo {
        name: field.name.clone(),
        bit_range: field.bit_range.clone(),
        description: field.description.clone(),
        values: field
            .explaination
            .iter()
            .map(|(value, description)| ConfigValueInfo {
                value: value.clone(),
                description: description.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_expected_wch_database_shape() {
        let catalog = load_catalog().expect("catalog should load");
        assert_eq!(catalog.family_count, 16);
        assert_eq!(catalog.variant_count, 85);
        assert!(catalog
            .families
            .iter()
            .any(|family| family.device_type_hex == "0x22"
                && family
                    .variants
                    .iter()
                    .any(|variant| variant.name == "CH592")));
    }
}
