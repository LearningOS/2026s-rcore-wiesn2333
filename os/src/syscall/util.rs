use crate::{
    mm::{translated_byte_buffer, UserBuffer},
    task::current_user_token,
};

pub fn write_vaddr<T>(dst: *mut u8, var: T) -> isize {
    let var_bytes = unsafe {
        core::slice::from_raw_parts(&var as *const T as *const u8, core::mem::size_of::<T>())
    };

    let buffers = UserBuffer::new(translated_byte_buffer(
        current_user_token(),
        dst as *const u8,
        var_bytes.len(),
    ))
    .into_iter();

    for (dst, src) in buffers.zip(var_bytes) {
        unsafe {
            dst.write(*src);
        }
    }
    var_bytes.len() as isize
}
