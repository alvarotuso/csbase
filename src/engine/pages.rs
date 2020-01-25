use bit_vec::BitVec;
use std::convert::TryInto;
use std::mem;

use crate::engine::asl;

const PAGE_SIZE: usize = 8 * 1024;
const USIZE_SIZE: usize = mem::size_of::<usize>();
const PAGE_DATA_SIZE: usize = PAGE_SIZE - mem::size_of::<u32>() - USIZE_SIZE*2;

#[derive(Clone)]
pub struct Page {
    id: u32,
    free_space_start: usize,
    free_space_end: usize,
    data: [u8; PAGE_DATA_SIZE],
}

impl Page {
    pub fn new(id: u32) -> Page {
        Page { id, data: [0; PAGE_DATA_SIZE], free_space_start: 0, free_space_end: PAGE_DATA_SIZE }
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

    fn get_items(&self) -> Vec<Item> {
        self.get_item_offset_and_sizes().iter().map(
            |(offset, size)| {
                Item::from_page_data(&self.data[*offset..*size])
            }
        ).collect()
    }
}

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
        let mut null_map = BitVec::new();
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
            null_map.set(idx, if let asl::Value::Null = value { true } else { false });
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
                    asl::Type::Str => Some(usize::from_be_bytes(self.field_data[offset..USIZE_SIZE].try_into().unwrap())),
                    asl::Type::Int => Some(4usize),
                    asl::Type::Float => Some(4usize),
                    asl::Type::Bool => Some(1usize),
                    _ => None,
                };
                if let Some(size) = size {
                    let next_offset = offset + size;
                    let bytes = &self.field_data[offset..next_offset];
                    values.push(asl::Value::from_be_bytes(bytes.to_vec(),
                                                          &column.column_type));
                }
            }

        }
        asl::Record { values }
    }
}