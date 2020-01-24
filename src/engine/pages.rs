use std::mem;

const PAGE_SIZE: usize = 8 * 1024;
const USIZE_SIZE: usize = mem::size_of::<usize>();
const PAGE_DATA_SIZE: usize = PAGE_SIZE - mem::size_of::<i32>() - USIZE_SIZE*2;

#[derive(Clone)]
pub struct Page {
    id: i32,
    free_space_start: usize,
    free_space_end: usize,
    data: [u8; PAGE_DATA_SIZE],
}

impl Page {
    pub fn new(id: i32) -> Page {
        Page { id, data: [0; PAGE_DATA_SIZE], free_space_start: 0, free_space_end: PAGE_DATA_SIZE }
    }

    fn get_item_offset_and_sizes(&self) -> Vec<(usize, usize)> {
        let mut item_offsets = Vec::new();
        let mut current_offset = 0;
        while current_offset < self.free_space_start {
            let size_offset = current_offset + USIZE_SIZE;
            let size_end = size_offset + USIZE_SIZE;
            item_offsets.push((
                usize::from_be_bytes(self.data[current_offset..size_offset]),
                usize::from_be_bytes(self.data[size_offset..size_end]),
            ));
            current_offset = size_end;
        }
        item_offsets
    }

    fn get_items(&self) -> Vec<Item> {
        
    }
}

pub struct Item {
    id: i32,
    nulls: u16,
    data: Vec<u8>,
}

impl Item {

}