pub fn as_slice<T, U: bytemuck::Pod>(value: &T) -> &[U] {
    let pc: *const T = value;
    let pc: *const u8 = pc as *const u8;
    let data = unsafe { std::slice::from_raw_parts(pc, std::mem::size_of::<T>()) };
    bytemuck::cast_slice(&data)
}