/**
* Copy the elements from slice slice_to_copy into copy_into_slice, starting from offset
*/
pub fn copy_bytes_into(copy_into_slice: &mut [u8], slice_to_copy: &[u8], offset: usize) {
    for (idx, element) in slice_to_copy.iter().enumerate() {
        copy_into_slice[offset + idx] = *element;
    }
}
