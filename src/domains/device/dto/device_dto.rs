use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use simple_dto_mapper_derive::DtoFrom;
use utoipa::ToSchema;

use crate::domains::device::domain::model::{Device, DeviceOS, DeviceStatus};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, ToSchema, DtoFrom)]
#[dto(from = Device)]
pub struct DeviceDto {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub name: String,
    pub device_os: DeviceOS,
    pub status: DeviceStatus,
    #[serde(with = "crate::common::ts_format::option")]
    pub registered_at: Option<DateTime<Utc>>,
    pub created_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub created_at: Option<DateTime<Utc>>,
    pub modified_by: Option<uuid::Uuid>,
    #[serde(with = "crate::common::ts_format::option")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(PartialEq, Debug, Deserialize, serde::Serialize, ToSchema)]
pub struct CreateDeviceDto {
    pub name: String,
    pub user_id: uuid::Uuid,
    pub device_os: DeviceOS,
    pub status: DeviceStatus,
    #[serde(with = "crate::common::ts_format::option")]
    pub registered_at: Option<DateTime<Utc>>,
    pub modified_by: uuid::Uuid,
}

#[derive(PartialEq, Debug, Deserialize, serde::Serialize, ToSchema)]
pub struct UpdateDeviceDto {
    pub name: Option<String>,
    pub user_id: Option<uuid::Uuid>,
    pub device_os: Option<DeviceOS>,
    pub status: Option<DeviceStatus>,
    #[serde(with = "crate::common::ts_format::option")]
    pub registered_at: Option<DateTime<Utc>>,
    pub modified_by: uuid::Uuid,
}

#[derive(Debug, Deserialize, serde::Serialize, ToSchema)]
pub struct UpdateManyDevicesDto {
    pub devices: Vec<UpdateDeviceDtoWithIdDto>,
}

#[derive(PartialEq, Debug, Deserialize, serde::Serialize, ToSchema)]
pub struct UpdateDeviceDtoWithIdDto {
    pub id: Option<uuid::Uuid>,
    pub name: String,
    pub device_os: DeviceOS,
    pub status: DeviceStatus,
}
