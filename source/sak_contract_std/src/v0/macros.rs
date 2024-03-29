#[macro_export]
macro_rules! contract_bootstrap {
    () => {
        /// Allocate memory into the module's linear memory
        /// and return the offset to the start of the block.
        #[no_mangle]
        pub extern "C" fn alloc(len: usize) -> *mut u8 {
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
        pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: usize) {
            let data = Vec::from_raw_parts(ptr, size, size);

            std::mem::drop(data);
        }
    };
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
                let mut err =
                    sak_contract_std::make_error_vec(err.into(), $msg);

                let err_ptr = err.as_mut_ptr();
                let err_len = err.len();

                std::mem::forget(err);

                let mut empty_vec = Vec::new();
                let empty_vec_ptr = empty_vec.as_mut_ptr();
                let empty_vec_len = empty_vec.len();

                return (
                    err_ptr,
                    err_len as i32,
                    empty_vec_ptr,
                    empty_vec_len as i32,
                );
            }
        }
    };
}

#[macro_export]
macro_rules! define_init {
    () => {
        #[no_mangle]
        pub unsafe extern "C" fn init() -> (*mut u8, i32) {
            let storage: Result<
                sak_contract_std::Storage,
                sak_contract_std::ContractError,
            > = init2();

            let mut storage = sak_contract_std::return_err_2!(storage);

            let storage_ptr = storage.as_mut_ptr();
            let storage_len = storage.len();

            std::mem::forget(storage);

            (storage_ptr, storage_len as i32)
        }
    };
}

#[macro_export]
macro_rules! define_query {
    () => {
        #[no_mangle]
        pub unsafe extern "C" fn query(
            storage_ptr: *mut u8,
            storage_len: usize,
            request_ptr: *mut u8,
            request_len: usize,
        ) -> (*mut u8, i32) {
            let storage: Storage = Vec::from_raw_parts(
                storage_ptr, //
                storage_len,
                storage_len,
            );

            let request = Vec::from_raw_parts(
                request_ptr, //
                request_len,
                request_len,
            );

            let request = serde_json::from_slice(&request);

            let request: sak_contract_std::CtrRequest =
                sak_contract_std::return_err_2!(request);

            let result: Result<
                sak_contract_std::InvokeResult,
                sak_contract_std::ContractError,
            > = query2(request, storage);

            {
                let mut result: sak_contract_std::InvokeResult =
                    sak_contract_std::return_err_2!(result);

                let result_ptr = result.as_mut_ptr();
                let result_len = result.len();

                std::mem::forget(result);

                return (result_ptr, result_len as i32);
            }
        }
    };
}

#[macro_export]
macro_rules! define_execute {
    () => {
        #[no_mangle]
        pub unsafe extern "C" fn execute(
            storage_ptr: *mut u8,
            storage_len: usize,
            request_ptr: *mut u8,
            request_len: usize,
        ) -> (*mut u8, i32, *mut u8, i32) {
            let mut storage: sak_contract_std::Storage = Vec::from_raw_parts(
                storage_ptr, //
                storage_len,
                storage_len,
            );

            let request = Vec::from_raw_parts(
                request_ptr, //
                request_len,
                request_len,
            );

            let request = serde_json::from_slice(&request);

            let request: sak_contract_std::CtrRequest =
                sak_contract_std::return_err_4!(
                    request,
                    "serde request parsing fail"
                );

            let result: Result<
                sak_contract_std::InvokeResult,
                sak_contract_std::ContractError,
            > = execute2(request, &mut storage);

            {
                let mut result: sak_contract_std::InvokeResult =
                    sak_contract_std::return_err_4!(
                        result,
                        "serde result parsing fail"
                    );

                let result_ptr = result.as_mut_ptr();
                let result_len = result.len();

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
    };
}
