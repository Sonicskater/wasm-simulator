use std::collections::{ HashSet, VecDeque};
use std::hash::Hash;
use algorithm::CacheAlgorithm;
use simulator_shared_types::FileRecord;

pub struct FiFo<T> {
    queue : VecDeque<FileRecord<T>>, // Double ended queue - basically ring buffer for order items have entered queue
    cache : HashSet<T>,
    current_used : i64, // current space in cache
    size : i64, // size of cache
    event_count: i32,
    hit_count : i32
}

impl <T> FiFo<T> where T : Hash + Eq + Clone {
    fn in_cache(&self, file : &FileRecord<T>) -> bool {
        self.cache.contains(&file.label.clone())
    }
}

impl <T> CacheAlgorithm<T> for FiFo<T> where T : Hash + Eq + Clone{
    fn simulate(&mut self, file: FileRecord<T>) {
        if file.size > self.size {
            println!("FILE SIZE: {}", file.size);
            panic!("File larger than cache")
        }
        self.event_count += 1;
        if self.in_cache(&file) {
            self.hit_count += 1;
            return;
        }
        let id = file.label.clone();
        self.cache.insert(file.label.clone());
        self.current_used += file.size;
        self.queue.push_front(file);
        while self.current_used > self.size {
            let popped = self.queue.pop_back().unwrap();
            if popped.label == id {
                panic!("Popped file we just inserted")
            }
            self.cache.remove(&popped.label.clone());
            self.current_used -= popped.size;
        }
    }

    fn new(size: i64) -> Self {
        FiFo::<T> {
            queue: VecDeque::<FileRecord<T>>::new(),
            cache: Default::default(),
            current_used: 0,
            size,
            event_count: 0,
            hit_count: 0
        }
    }

    fn stats(&self) -> (i32,i32) {
        (self.event_count, self.hit_count)
    }
}