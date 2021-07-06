#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    non_camel_case_types
)]
/*
 * Mayastor RESTful API
 *
 * The version of the OpenAPI document: v0
 *
 * Generated by: https://github.com/openebs/openapi-generator
 */

/// BlockDevice : Block device information

/// Block device information
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BlockDevice {
    /// identifies if device is available for use (ie. is not \"currently\" in  use)
    #[serde(rename = "available")]
    pub available: bool,
    /// list of udev generated symlinks by which device may be identified
    #[serde(rename = "devlinks")]
    pub devlinks: Vec<String>,
    /// major device number
    #[serde(rename = "devmajor")]
    pub devmajor: i32,
    /// minor device number
    #[serde(rename = "devminor")]
    pub devminor: i32,
    /// entry in /dev associated with device
    #[serde(rename = "devname")]
    pub devname: String,
    /// official device path
    #[serde(rename = "devpath")]
    pub devpath: String,
    /// currently \"disk\" or \"partition\"
    #[serde(rename = "devtype")]
    pub devtype: String,
    #[serde(rename = "filesystem")]
    pub filesystem: crate::models::BlockDeviceFilesystem,
    /// device model - useful for identifying mayastor devices
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "partition")]
    pub partition: crate::models::BlockDevicePartition,
    /// size of device in (512 byte) blocks
    #[serde(rename = "size")]
    pub size: i64,
}

impl BlockDevice {
    /// BlockDevice using only the required fields
    pub fn new(
        available: bool,
        devlinks: Vec<String>,
        devmajor: i32,
        devminor: i32,
        devname: String,
        devpath: String,
        devtype: String,
        filesystem: crate::models::BlockDeviceFilesystem,
        model: String,
        partition: crate::models::BlockDevicePartition,
        size: i64,
    ) -> BlockDevice {
        BlockDevice {
            available,
            devlinks,
            devmajor,
            devminor,
            devname,
            devpath,
            devtype,
            filesystem,
            model,
            partition,
            size,
        }
    }
    /// BlockDevice using all fields
    pub fn new_all(
        available: bool,
        devlinks: Vec<String>,
        devmajor: i32,
        devminor: i32,
        devname: String,
        devpath: String,
        devtype: String,
        filesystem: crate::models::BlockDeviceFilesystem,
        model: String,
        partition: crate::models::BlockDevicePartition,
        size: i64,
    ) -> BlockDevice {
        BlockDevice {
            available,
            devlinks,
            devmajor,
            devminor,
            devname,
            devpath,
            devtype,
            filesystem,
            model,
            partition,
            size,
        }
    }
}