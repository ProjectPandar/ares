const FROM: &str = "package/start.txt";
const TO: &str = "data/result.txt";

#[cfg(not(feature = "wasi"))]
#[link(wasm_import_module = "ares")]
unsafe extern "C" {
    fn file_read(path: *const u8, path_len: usize, output: *mut u8, capacity: usize) -> i32;
    fn file_write(path: *const u8, path_len: usize, input: *const u8, len: usize) -> i32;
}

#[unsafe(no_mangle)]
pub extern "C" fn read_write() -> i32 {
    #[cfg(feature = "wasi")]
    {
        let bytes = std::fs::read(FROM).unwrap();
        std::fs::write(TO, &bytes).unwrap();
        bytes.iter().map(|byte| i32::from(*byte)).sum()
    }
    #[cfg(not(feature = "wasi"))]
    {
        let mut bytes = [0; 32];
        // Both slices stay live and disjoint throughout the copy.
        let size = unsafe { file_read(FROM.as_ptr(), FROM.len(), bytes.as_mut_ptr(), bytes.len()) };
        assert!(size >= 0);
        let bytes = &bytes[..size as usize];
        // The host reads from live slices for the duration of the call.
        let written = unsafe { file_write(TO.as_ptr(), TO.len(), bytes.as_ptr(), bytes.len()) };
        assert_eq!(written, size);
        bytes.iter().map(|byte| i32::from(*byte)).sum()
    }
}
