use crate::log_error;
use crate::mender_mcu_client::core::mender_api::{
    mender_api_get_authentication_token, MyTextCallback,
};
use crate::mender_mcu_client::core::mender_utils::{KeyStore, MenderResult, MenderStatus};
use crate::mender_mcu_client::platform::net::mender_http::{
    self, HttpMethod, MenderHttpResponseData,
};
use heapless::String as HString;
use serde_json_core::de::from_str;

#[allow(unused_labels)]
const MENDER_API_PATH_GET_DEVICE_CONFIGURATION: &str = "/api/devices/v1/deviceconfig/configuration";
const MENDER_API_PATH_PUT_DEVICE_CONFIGURATION: &str = "/api/devices/v1/deviceconfig/configuration";
const MAX_PAYLOAD_SIZE: usize = 256;

#[allow(dead_code)]
//#[cfg(not(feature = "mender_client_configure_storage"))]
pub async fn mender_configure_api_download_configuration_data() -> MenderResult<KeyStore> {
    // Prepare response data structure
    let mut response_data = MenderHttpResponseData::default();
    let mut status = 0;
    let my_text_callback = MyTextCallback;

    // Perform HTTP request
    let (_, jwt) = mender_api_get_authentication_token().await?;
    let ret = mender_http::mender_http_perform(
        Some(&jwt),
        MENDER_API_PATH_GET_DEVICE_CONFIGURATION,
        HttpMethod::Get,
        None,
        None,
        &my_text_callback,
        &mut response_data,
        &mut status,
        None,
    )
    .await;

    if ret.is_err() {
        log_error!("Unable to perform HTTP request");
        return Err(MenderStatus::Failed);
    }

    // Treatment depending on the status
    if status == 200 {
        if let Some(response_text) = response_data.text {
            // Parse JSON response
            match from_str::<KeyStore>(&response_text) {
                Ok((keystore, _)) => Ok((MenderStatus::Ok, keystore)),
                Err(_) => {
                    log_error!("Unable to parse configuration");
                    Err(MenderStatus::Failed)
                }
            }
        } else {
            log_error!("No response data");
            Err(MenderStatus::Failed)
        }
    } else {
        log_error!("Unexpected status code:", "status" => status);
        Err(MenderStatus::Failed)
    }
}

pub async fn mender_configure_api_publish_configuration_data(
    configuration: &KeyStore,
) -> MenderResult<()> {
    // Convert configuration to JSON string
    let payload: HString<MAX_PAYLOAD_SIZE> = match serde_json_core::ser::to_string(configuration) {
        Ok(p) => p,
        Err(_) => {
            log_error!("Unable to format configuration data");
            return Err(MenderStatus::Failed);
        }
    };

    // Prepare response data structure
    let mut response_data = MenderHttpResponseData::default();
    let mut status = 0;
    let my_text_callback = MyTextCallback;

    // Perform HTTP request
    let (_, jwt) = mender_api_get_authentication_token().await?;
    let ret = mender_http::mender_http_perform(
        Some(&jwt),
        MENDER_API_PATH_PUT_DEVICE_CONFIGURATION,
        HttpMethod::Put,
        Some(&payload),
        None,
        &my_text_callback,
        &mut response_data,
        &mut status,
        None,
    )
    .await;

    if ret.is_err() {
        log_error!("Unable to perform HTTP request");
        return Err(MenderStatus::Failed);
    }

    // Treatment depending on the status
    if status == 204 {
        Ok((MenderStatus::Ok, ()))
    } else {
        log_error!("Unexpected status code:", "status" => status);
        Err(MenderStatus::Failed)
    }
}
