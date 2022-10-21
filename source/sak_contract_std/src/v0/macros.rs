#[macro_export]
macro_rules! contract_bootstrap {
    () => {
        #[link(wasm_import_module = "host")]
        extern "C" {
            fn HOST__log(param1: i32, param2: i32) -> i32;

            fn HOST__get_mrs_data(param1: *mut u8, param2: u32, ptr_ret_len: *mut u32) -> i32;

            fn HOST__get_latest_return_len(p1: i32, p2: i32) -> i32;
        }

        /// Allocate memory into the module's linear memory
        /// and return the offset to the start of the block.
        #[no_mangle]
        pub extern "C" fn CTR__alloc(len: usize) -> *mut u8 {
            // create a new mutable buffer with capacity `len`
            let mut buf = Vec::with_capacity(len);
            // take a mutable pointer to the buffer
            let ptr = buf.as_mut_ptr();
            // take ownership of the memory block and
            // ensure the its destructor is not
            // called when the object goes out of scope
            // at the end of the function
            std::mem::forget(buf);
            // return the pointer so the runtime
            // can write data at this offset
            return ptr;
        }

        #[no_mangle]
        pub unsafe extern "C" fn CTR__dealloc(ptr: *mut u8, size: usize) {
            let data = Vec::from_raw_parts(ptr, size, size);

            std::mem::drop(data);
        }

        #[no_mangle]
        pub unsafe extern "C" fn CTR__init() -> (*mut u8, i32) {
            let storage: Result<$crate::Storage, $crate::ContractError> = init();

            let mut storage = $crate::return_err_2!(storage);

            let storage_ptr = storage.as_mut_ptr();
            let storage_len = storage.len();

            std::mem::forget(storage);

            (storage_ptr, storage_len as i32)
        }

        #[no_mangle]
        pub unsafe extern "C" fn CTR__execute(
            request_ptr: *mut u8,
            request_len: usize,
        ) -> (*mut u8, i32, *mut u8, i32) {
            let request = $crate::parse_request!(request_ptr, request_len);

            let mrs = __make_mrs_storage_param();

            let ctx = ContractCtx { mrs };

            let result: Result<$crate::InvokeResult, $crate::ContractError> = query(ctx, request);

            {
                let mut result: $crate::InvokeResult =
                    $crate::return_err_4!(result, "something failed");

                let result_ptr = result.as_mut_ptr();
                let result_len = result.len();

                std::mem::forget(result);

                let mut empty_vec = Vec::new();
                let empty_vec_ptr = empty_vec.as_mut_ptr();
                let empty_vec_len = empty_vec.len();

                return (
                    result_ptr,
                    result_len as i32,
                    empty_vec_ptr,
                    empty_vec_len as i32,
                );
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn CTR__update(
            request_ptr: *mut u8,
            request_len: usize,
        ) -> (*mut u8, i32, *mut u8, i32) {
            let request = $crate::parse_request!(request_ptr, request_len);

            let mrs = __make_mrs_storage_param();

            let ctx = ContractCtx { mrs };

            let result: Result<$crate::InvokeResult, $crate::ContractError> = update(ctx, request);

            {
                let mut result: $crate::InvokeResult =
                    $crate::return_err_4!(result, "serde result parsing fail");

                let result_ptr = result.as_mut_ptr();
                let result_len = result.len();

                let mut storage = vec![];
                let storage_ptr = storage.as_mut_ptr();
                let storage_len = storage.len();

                std::mem::forget(storage);

                (
                    storage_ptr,
                    storage_len as i32,
                    result_ptr,
                    result_len as i32,
                )
            }
        }

        $crate::define_contract_ctx!();
    };
}

#[macro_export]
macro_rules! define_contract_ctx {
    () => {
        pub struct ContractCtx {
            mrs: _MRS,
        }

        impl ContractCtx {
            unsafe fn get_mrs_data(&self, key: &String) -> Vec<u8> {
                let key_len = key.len();
                let key_ptr = CTR__alloc(key_len);
                key_ptr.copy_from(key.as_ptr(), key_len);

                let ret_len_ptr = CTR__alloc($crate::RET_LEN_SIZE);
                let ret_ptr = HOST__get_mrs_data(key_ptr, key_len as u32, ret_len_ptr as *mut u32);
                // let ret_len = {
                //     let bytes: [u8; $crate::RET_LEN_SIZE] =
                //         std::slice::from_raw_parts(ret_len_ptr as *mut u8, $crate::RET_LEN_SIZE)
                //             .try_into()
                //             .unwrap();
                //     u32::from_be_bytes(bytes)
                // };

                // HOST__log(ret_len as i32, 135);

                // let data =
                //     Vec::from_raw_parts(ret_ptr as *mut u8, ret_len as usize, ret_len as usize);
                // data

                vec![]
            }

            unsafe fn put_mrs_data(&self, arg: usize) -> usize {
                0
            }
        }
    };
}

#[macro_export]
macro_rules! parse_request {
    ($ptr: expr, $len: expr) => {{
        let request_vec = Vec::from_raw_parts($ptr, $len, $len);
        let maybe_req = serde_json::from_slice(&request_vec);
        let req: sak_contract_std::CtrRequest =
            $crate::return_err_4!(maybe_req, "something failed");
        req
    }};
}

#[macro_export]
macro_rules! return_err_2 {
    ($obj: expr) => {
        match $obj {
            Ok(r) => r,
            Err(err) => {
                let mut err = sak_contract_std::make_error_vec(err.into(), "");

                let err_ptr = err.as_mut_ptr();
                let err_len = err.len();

                std::mem::forget(err);

                return (err_ptr, err_len as i32);
            }
        }
    };
}

#[macro_export]
macro_rules! return_err_4 {
    ($obj: expr, $msg: expr) => {
        match $obj {
            Ok(r) => r,
            Err(err) => {
                let mut err = sak_contract_std::make_error_vec(err.into(), $msg);

                let err_ptr = err.as_mut_ptr();
                let err_len = err.len();

                std::mem::forget(err);

                let mut empty_vec = Vec::new();
                let empty_vec_ptr = empty_vec.as_mut_ptr();
                let empty_vec_len = empty_vec.len();

                return (err_ptr, err_len as i32, empty_vec_ptr, empty_vec_len as i32);
            }
        }
    };
}
