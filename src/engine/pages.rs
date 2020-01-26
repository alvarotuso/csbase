use bit_vec::BitVec;
use std::convert::TryInto;
use std::mem;

use crate::engine::asl;
use crate::engine::errors::PagingError;
use crate::engine::utils::copy_bytes_into;

pub const PAGE_SIZE: usize = 8 * 1024;
const USIZE_SIZE: usize = mem::size_of::<usize>();
const U32_SIZE: usize = mem::size_of::<u32>();
const PAGE_DATA_SIZE: usize = PAGE_SIZE - U32_SIZE - USIZE_SIZE*2;

/**
* TryFrom trait copied from the std lib and implemented specifically for the page data size
* This is needed because this trait is only supported for arrays of size up to 32
*/
fn try_page_from_slice(slice: &[u8]) -> Result<&[u8; PAGE_DATA_SIZE], PagingError> {
    if slice.len() == PAGE_DATA_SIZE {
        let ptr = slice.as_ptr() as *const [u8; PAGE_DATA_SIZE];
        unsafe { Ok(&*ptr) }
    } else {
        Err(PagingError::InvalidSliceLength)
    }
}

#[derive(Clone)]
pub struct Page {
    pub id: u32,
    free_space_start: usize,
    free_space_end: usize,
    data: [u8; PAGE_DATA_SIZE],
}

impl Page {
    pub fn new(id: u32) -> Page {
        Page { id, data: [0; PAGE_DATA_SIZE], free_space_start: 0, free_space_end: PAGE_DATA_SIZE }
    }

    /**
    * Create a page struct from raw bytes, possibly coming from a file
    */
    pub fn from_bytes(bytes: &[u8; PAGE_SIZE]) -> Page {
        let mut offset = 0;
        let id = u32::from_be_bytes(bytes[offset..offset + U32_SIZE].try_into().unwrap());
        offset += U32_SIZE;
        let free_space_start = usize::from_be_bytes(bytes[offset..offset + USIZE_SIZE].try_into().unwrap());
        offset += USIZE_SIZE;
        let free_space_end = usize::from_be_bytes(bytes[offset..offset + USIZE_SIZE].try_into().unwrap());
        offset += USIZE_SIZE;
        let data: [u8; PAGE_DATA_SIZE] = *try_page_from_slice(&bytes[offset..offset + PAGE_DATA_SIZE]).unwrap();
        Page {
            id,
            free_space_start,
            free_space_end,
            data,
        }
    }

    /**
    * Return a byte representation of this page
    */
    pub fn to_bytes(&self) -> [u8; PAGE_SIZE] {
        let mut bytes = [0u8; PAGE_SIZE];
        let mut offset = 0;
        copy_bytes_into(&mut bytes, &self.id.to_be_bytes(), offset);
        offset += U32_SIZE;
        copy_bytes_into(&mut bytes, &self.free_space_start.to_be_bytes(), offset);
        offset += USIZE_SIZE;
        copy_bytes_into(&mut bytes, &self.free_space_end.to_be_bytes(), offset);
        offset += USIZE_SIZE;
        copy_bytes_into(&mut bytes, &self.data, offset);
        bytes
    }

    fn get_item_offset_and_sizes(&self) -> Vec<(usize, usize)> {
        let mut item_offsets = Vec::new();
        let mut current_offset = 0;
        while current_offset < self.free_space_start {
            let size_offset = current_offset + USIZE_SIZE;
            let size_end = size_offset + USIZE_SIZE;
            item_offsets.push((
                usize::from_be_bytes(self.data[current_offset..size_offset].try_into().unwrap()),
                usize::from_be_bytes(self.data[size_offset..size_end].try_into().unwrap()),
            ));
            current_offset = size_end;
        }
        item_offsets
    }

    pub fn get_items(&self) -> Vec<Item> {
        self.get_item_offset_and_sizes().iter().map(
            |(offset, size)| {
                Item::from_page_data(&self.data[*offset..*offset + *size])
            }
        ).collect()
    }

    fn get_free_space(&self) -> usize {
        self.free_space_end - self.free_space_start
    }

    pub fn add_item(&mut self, item: &Item) -> Result<(), PagingError> {
        let item_data = item.to_page_data();
        let item_size = item_data.len();
        if item_size + USIZE_SIZE*2 > self.get_free_space() {
            Err(PagingError::NotEnoughSpace)
        } else {
            let item_offset = &self.free_space_end - item_size;
            copy_bytes_into(&mut self.data, &item_offset.to_be_bytes(), self.free_space_start);
            copy_bytes_into(&mut self.data, &item_size.to_be_bytes(), self.free_space_start + USIZE_SIZE);
            self.free_space_start += USIZE_SIZE*2;
            copy_bytes_into(&mut self.data, item_data.as_slice(), item_offset);
            self.free_space_end = item_offset;
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    number_of_fields: usize,
    null_map: BitVec,
    field_data: Vec<u8>,
}

impl Item {
    /**
    * Get the null map length in bytes
    */
    pub fn get_null_map_length(number_of_fields: &usize) -> usize {
        number_of_fields / 8 + if number_of_fields % 8 == 0 { 0 } else { 1 } // compute math ceil
    }

    /**
    * Build an item from page data
    */
    pub fn from_page_data(page_data: &[u8]) -> Item {
        let number_of_fields = usize::from_be_bytes(page_data[0..USIZE_SIZE].try_into().unwrap());
        let null_map_length = Item::get_null_map_length(&number_of_fields);
        Item {
            number_of_fields,
            null_map: BitVec::from_bytes(&page_data[USIZE_SIZE..USIZE_SIZE + null_map_length]),
            field_data: page_data[USIZE_SIZE + null_map_length..page_data.len()].to_vec(),
        }
    }

    /**
    * Build an item from a record
    */
    pub fn from_record(record: &asl::Record) -> Item {
        let number_of_fields = record.values.len();
        let mut null_map = BitVec::from_elem(number_of_fields, false);
        let mut field_data = Vec::new();
        for (idx, value) in record.values.iter().enumerate() {
            let value_bytes = match value {
                asl::Value::Str(s) => {
                    let mut size_bytes = s.len().to_be_bytes().to_vec();
                    size_bytes.extend(value.to_be_bytes());
                    Some(size_bytes)
                },
                asl::Value::Int(_) => Some(value.to_be_bytes()),
                asl::Value::Float(_) => Some(value.to_be_bytes()),
                asl::Value::Bool(_) => Some(value.to_be_bytes()),
                asl::Value::Null => None,
            };
            if let Some(bytes) = value_bytes {
                field_data.extend(bytes);
            }
            if let asl::Value::Null = value {
                null_map.set(idx, true);
            }
        }
        Item {
            number_of_fields,
            null_map,
            field_data,
        }
    }

    /**
    * Output an u8 array representing the page data for the item
    */
    pub fn to_page_data(&self) -> Vec<u8> {
        let mut page_data = Vec::new();
        page_data.extend(&self.number_of_fields.to_be_bytes());
        page_data.extend(&self.null_map.to_bytes());
        page_data.extend(&self.field_data);
        page_data
    }

    /**
    * Build a record from this item data
    */
    pub fn to_record(&self, table: &asl::Table) -> asl::Record {
        let mut values = Vec::new();
        let mut offset = 0;
        for (idx, column) in table.columns.iter().enumerate() {
            let is_null_value = if let Some(is_null) = self.null_map.get(idx) {
                is_null
            } else {
                false
            };
            if is_null_value {
                values.push(asl::Value::Null);
            } else {
                let size = match column.column_type {
                    asl::Type::Str => {
                        let size = usize::from_be_bytes(self.field_data[offset..offset + USIZE_SIZE].try_into().unwrap());
                        offset += USIZE_SIZE;
                        Some(size)
                    },
                    asl::Type::Int => Some(mem::size_of::<i32>()),
                    asl::Type::Float => Some(mem::size_of::<f32>()),
                    asl::Type::Bool => Some(mem::size_of::<u8>()),
                    _ => None,
                };
                if let Some(size) = size {
                    let next_offset = offset + size;
                    let bytes = &self.field_data[offset..next_offset];
                    values.push(asl::Value::from_be_bytes(bytes.to_vec(),
                                                          &column.column_type));
                    offset = next_offset;
                }
            }

        }
        asl::Record { values }
    }
}