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

    fn get_item_offsets(&self) -> Vec<usize> {
        let mut item_offsets = Vec::new();
        let mut current_offset = 0;
        while current_offset < self.free_space_start {
            item_offsets.push(usize::from_be_bytes(self.data[current_offset..USIZE_SIZE]));
            current_offset += USIZE_SIZE;
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