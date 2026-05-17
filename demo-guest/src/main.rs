#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};
use hyperlight_common::flatbuffer_wrappers::{
    function_call::FunctionCall,
    function_types::{ParameterType, ParameterValue, ReturnType},
    guest_error::ErrorCode,
};
use hyperlight_common::flatbuffer_wrappers::util::get_flatbuffer_result;

use hyperlight_guest_bin::{
    guest_function::definition::{GuestFunc, GuestFunctionDefinition},
    guest_function::register::register_function,
    host_comm::call_host_function,
};

use hyperlight_guest::error::HyperlightGuestError;

fn dereference_raw_null_pointer(_: FunctionCall) -> hyperlight_guest::error::Result<Vec<u8>> {
    let null_pointer: *const u8 = core::ptr::null();
    let _res = unsafe { *null_pointer };
    Ok(get_flatbuffer_result(()))
}

fn print_output(message: &str) -> hyperlight_guest::error::Result<Vec<u8>> {
    let result: i32 = call_host_function(
        "HostPrint",
        Some(Vec::from(&[ParameterValue::String(message.to_string())])),
        ReturnType::Int,
    )?;

    Ok(get_flatbuffer_result(result))
}

fn simple_print_output(function_call: FunctionCall) -> hyperlight_guest::error::Result<Vec<u8>> {
    if let ParameterValue::String(message) = function_call.parameters.clone().unwrap()[0].clone() {
        print_output(&message)
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to simple_print_output".to_string(),
        ))
    }
}

#[no_mangle]
pub extern "C" fn hyperlight_main() {
    let simple_print_output_def = GuestFunctionDefinition::new(
        "PrintOutput".to_string(),
        Vec::from(&[ParameterType::String]),
        ReturnType::Int,
        simple_print_output as GuestFunc,
    );
    register_function(simple_print_output_def);

    let dereference_raw_null_pointer_def = GuestFunctionDefinition::new(
        "DereferenceRawNullPointer".to_string(),
        Vec::new(),
        ReturnType::Void,
        dereference_raw_null_pointer as GuestFunc,
    );
    register_function(dereference_raw_null_pointer_def);
}

#[no_mangle]
pub fn guest_dispatch_function(_: FunctionCall) -> hyperlight_guest::error::Result<Vec<u8>> {
    Ok(get_flatbuffer_result(()))
}
