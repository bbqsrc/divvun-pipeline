use capnp::{message::ReaderOptions, serialize};
use divvun_schema::{
    capnp_message,
    interface::{self, PipelineInterface},
    module_metadata,
    string_capnp::string,
    util,
};
use lazy_static::lazy_static;
use std::{ffi::CStr, io::Cursor, os::raw::c_char};

#[no_mangle]
pub extern "C" fn pipeline_init(interface: *const PipelineInterface) -> bool {
    println!("pipeline_init reverse-string");
    interface::initialize(interface)
}

#[no_mangle]
pub extern "C" fn pipeline_run(
    command: *const c_char,
    input_count: usize,
    input: *const *const u8,
    input_sizes: *const usize,
    output: *mut *const u8,
    output_size: *mut usize,
) -> bool {
    println!("Hello, world from module!");
    let command = unsafe { CStr::from_ptr(command) }.to_string_lossy();
    println!("command = {}, input_count = {}", command, input_count);

    let input_sizes = unsafe { std::slice::from_raw_parts(input_sizes, input_count) };
    let input = unsafe { std::slice::from_raw_parts(input, input_count) };

    if input.len() == 0 {
        util::output_message(
            output,
            output_size,
            divvun_schema::capnp_error!(
                divvun_schema::error_capnp::pipeline_error::ErrorKind::ModuleError,
                "no input provided"
            ),
        )
        .unwrap();
        return false;
    }

    match &*command {
        "reverse" => {
            for i in 0..input_count {
                let message =
                    util::read_message::<string::Owned>(input[i], input_sizes[i]).unwrap();
                let string = message.get().unwrap();
                let result: String = string.get_string().unwrap().chars().rev().collect();
                println!(
                    "receives input {}, returning {}",
                    string.get_string().unwrap(),
                    result
                );

                util::output_message(
                    output,
                    output_size,
                    capnp_message!(string::Builder, builder => {
                        builder.set_string(&result);
                    }),
                )
                .unwrap();

                println!("returning from reverse");

                return true;
            }

            false
        }
        "reverse_resource" => {
            let message = util::read_message::<string::Owned>(input[0], input_sizes[0]).unwrap();
            let string = message.get().unwrap();
            let resource = string.get_string().unwrap();
            println!("loading resource {}", resource);
            let res = interface::load_resource(resource).unwrap();
            println!("res {:?}", res);

            let string = String::from_utf8_lossy(res.as_slice());
            let result: String = string.chars().rev().collect();
            println!("receives input {}, returning {}", string, result);

            util::output_message(
                output,
                output_size,
                capnp_message!(string::Builder, builder => {
                    builder.set_string(&result);
                }),
            )
            .unwrap();

            true
        }
        _ => {
            util::output_message(
                output,
                output_size,
                divvun_schema::capnp_error!(
                    divvun_schema::error_capnp::pipeline_error::ErrorKind::UnknownCommand,
                    &format!("unknown command {}", command)
                ),
            )
            .unwrap();

            false
        }
    }
}

#[no_mangle]
pub extern "C" fn pipeline_info(metadata: *mut *const u8, metadata_size: *mut usize) -> bool {
    lazy_static! {
        static ref MESSAGE: Vec<u8> = divvun_schema::util::message_to_vec(module_metadata! {
            name: "reverse-string",
            commands: {
                "reverse" => [divvun_schema::string_capnp::string::Builder] => divvun_schema::string_capnp::string::Builder,
                "reverse_resource" => [divvun_schema::string_capnp::string::Builder] => divvun_schema::string_capnp::string::Builder,
            }
        }).unwrap();
    }

    unsafe {
        *metadata = MESSAGE.as_ptr();
        *metadata_size = MESSAGE.len();
    }

    true
}
