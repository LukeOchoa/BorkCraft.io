mod borkcraft;
mod errors;
mod login;
mod pages;

pub use borkcraft::*;
use borkcraft_app::SessionInformation;
use errors::client_errors::ErrorMessage;
pub use pages::nether_portals::*;

use serde::Serialize;
pub fn to_vec8(cereal: &impl Serialize) -> Vec<u8> {
    serde_json::to_vec(cereal).unwrap()
}
pub enum ResponseResult {
    Text(String),
    SessionInformation(SessionInformation),
    NetherPortal(NetherPortal),
}
pub fn ureq_did_request_go_through_f(
    did_request_go_through: Result<ureq::Response, ureq::Error>,
    job: Box<dyn Fn(ureq::Response) -> Result<ResponseResult, String>>,
) -> Result<ResponseResult, ErrorMessage> {
    let failed_to_convert_response =
        "Failed to convert response to a variant of type Enum ResponseResult...: error ==";
    let bad_server_response_error = "network was sent but denied by the server... Maybe wrong key was given? [retrieving nether portal keys]: error ==";
    let _no_connection_error =
        "network request was not able to connect... [retrieving nether portal keys2]: error ==";

    match did_request_go_through {
        Ok(response) => match response.status() {
            200..=299 => match job(response) {
                Ok(result) => return Ok(result),
                Err(error) => {
                    return Err(ErrorMessage::pure_error_message(Some(format!(
                        "{}{}",
                        failed_to_convert_response, error
                    ))))
                }
            },
            _ => {
                let error_string =
                    format!("{}{}", bad_server_response_error, response.status_text());
                return Err(ErrorMessage::pure_error_message(Some(error_string)));
            }
        },
        Err(error) => {
            println!("Proc: {}", error.kind());
            let error_string = format!("{}: {}", error.kind(), error.to_string());
            return Err(ErrorMessage::pure_error_message(Some(error_string)));
        }
    }
}
pub fn match_out_nether_portal_keys_to_string2(
    did_request_go_through: Result<ureq::Response, ureq::Error>,
) -> Result<String, ErrorMessage> {
    let result = ureq_did_request_go_through_f(
        did_request_go_through,
        Box::new(
            |response: ureq::Response| -> Result<ResponseResult, String> {
                match response.into_string() {
                    Ok(string) => return Ok(ResponseResult::Text(string)),
                    Err(error) => return Err(error.to_string()),
                }
            },
        ),
    );
    match result {
        Ok(response_result) => {
            //let mut some_string = String::new();
            if let ResponseResult::Text(string) = response_result {
                //some_string = string;
                return Ok(string);
            } else {
                panic!("Magical faires occured at line 82 in nether_portals.rs");
            }
            //return some_string;
        }
        Err(error) => return Err(error),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn
// }
