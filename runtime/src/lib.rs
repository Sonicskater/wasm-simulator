#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use wasmer::Memory;

pub fn write_bincode_to_wasm_memory<T : serde::Serialize>(data: T, memory: &Memory, ptr: usize, len: usize){
    let serialized_array = bincode::serialize(&data).expect("Failed to serialize type");
    write_bytes_to_wasm_memory(&*serialized_array, memory, ptr, len)
}

pub fn write_bytemuck_to_wasm_memory<T : bytemuck::Pod >(data: T, memory: &Memory, ptr: usize, len: usize){
    let bytes = bytemuck::bytes_of(&data);
    //println!("Writing {} bytes using Bytemuck",bytes.len());
    write_bytes_to_wasm_memory(bytes, memory, ptr, len)
}

pub fn write_bytes_to_wasm_memory(bytes: &[u8], memory: &Memory, ptr: usize, len: usize){
    modularis::write_bytes_to_wasm_memory(bytes,memory,ptr,len)
}