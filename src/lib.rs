use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[allow(dead_code)]
fn insert_key(map: Arc<Mutex<HashMap<String, String>>>, key: String, value: String) -> bool {
    
    let result = map.lock().unwrap().insert(key, value);
    match result{
        Some(_) => {return true;},
        None => {return false;}
    }
}
#[allow(dead_code)]
fn insert_with_expiry(map: Arc<Mutex<HashMap<String, String>>>, expiry: Arc<Mutex<HashMap<String, Instant>>>, timeout_seconds: u64, key: String, value: String) -> bool {
    let instant_key = key.to_owned();
    let result = map.lock().unwrap().insert(key, value);
    let current_timestamp = Instant::now();
    let future = current_timestamp + Duration::from_secs(timeout_seconds);

    expiry.lock().unwrap().insert(instant_key, future);
    match result{
        Some(_) => {return true;},
        None => {return false;}
    }

}

#[allow(dead_code)]
fn get_key(map: Arc<Mutex<HashMap<String, String>>>,expiry: Arc<Mutex<HashMap<String, Instant>>>, key: String) -> Option<String> {
    let expiry_key = key.to_owned();

    let now = Instant::now();
    let expiration= expiry.lock().unwrap().get(&expiry_key).cloned();
    match expiration{
        Some(expiry_time) => {
            if expiry_time < now {
                expiry.lock().unwrap().remove(&expiry_key.to_owned());
                map.lock().unwrap().remove(&expiry_key.to_owned());
                return None
            }
        },
        None => {}
    }
    let result = map.lock().unwrap().get(&key).cloned();
    return result;
}


#[allow(dead_code)]
fn evict_key(map: Arc<Mutex<HashMap<String, String>>>, key: String) {

    map.lock().unwrap().remove(&key);
    
}
pub mod test{
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use std::thread;
#[test]
fn test_multithreaded_create() {
    // size is the first base 2 greater than 10 million as per the requirements.  this means the hashmap is reaosnable
    const ELEMENTS: usize = 14680064;
    let arcmap = Arc::new(Mutex::new(HashMap::<String, String>::with_capacity(ELEMENTS)));
    let expiry = Arc::new(Mutex::new(HashMap::<String, Instant>::with_capacity(ELEMENTS)));
    
    
    
    let guard = arcmap.lock().unwrap();
    assert_eq!(guard.capacity(), 14680064);

    // lets drop it so that we can check the original
    std::mem::drop(guard);
  
   
    for j in 0..10000 {
        let key = format!("{}", j);
           insert_key(arcmap.clone(),key.to_owned(), "1".to_owned());
        
    }

    let secondary = arcmap.clone();
    for j in 0..10000{
        let start = Instant::now();
        let key = format!("{}", j);
        get_key(secondary.clone(),expiry.clone(), key.to_owned());
    let duration = start.elapsed();

    println!("Time elapsed in expensive_function() is: {:?}", duration);
    }

   
}   

#[test]
fn test_insert_key() {
    // Create a new hash map and store it in an Arc wrapper.
    let map: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let expiry = Arc::new(Mutex::new(HashMap::<String, Instant>::with_capacity(14680064)));
    

    
    insert_key(map.clone(), "hello".to_owned(), "world".to_owned());

    let result = get_key(map.clone(),expiry.clone(), "hello".to_owned());
    // Check that the key-value pair was successfully inserted.
    assert_eq!("world".to_owned(), result.unwrap());
}

#[test]
fn test_get_key() {
    // Create a new hash map and store it in an Arc wrapper.
    let map: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let expiry = Arc::new(Mutex::new(HashMap::<String, Instant>::with_capacity(14680064)));
    
    
    // Insert a key-value pair into the map.
    insert_key(map.clone(), "hello".to_owned(), "world".to_owned());

    // Test getting the value associated with the key from the map.
    assert_eq!(get_key(map.clone(),expiry.clone(), "hello".to_owned()), Some("world".to_owned()));

    // Test getting a value for a key that does not exist in the map.
    assert_eq!(get_key(map.clone(),expiry.clone(), "foo".to_owned()), None);
}

#[test]
fn test_evict_key() {
    // Create a new hash map and store it in an Arc wrapper.
    let map: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let expiry = Arc::new(Mutex::new(HashMap::<String, Instant>::with_capacity(14680064)));
    
    
    // Insert a key-value pair into the map.
    insert_key(map.clone(), "hello".to_owned(), "world".to_owned());

    // Test removing the key-value pair from the map.
    evict_key(map.clone(), "hello".to_owned());

    // Check that the key-value pair was successfully removed.
    assert_eq!(get_key(map.clone(), expiry.clone(),"hello".to_owned()), None);



    insert_with_expiry(map.clone(), expiry.clone(), 4, "expires".to_owned(), "shortly".to_owned());
    let item = get_key(map.clone(), expiry.clone(), "expires".to_owned());
        match item{
            // will exist and do nothing
            Some(_) => {},
            None => panic!("should exist for now"),
        }

    let seconds_1 = Duration::from_secs(1);
    std::thread::sleep(seconds_1);

     let item = get_key(map.clone(), expiry.clone(), "expires".to_owned());
        match item{
            // will exist and do nothing
            Some(_) => {},
            None => panic!("should exist for now"),
        }
    let seconds_4 = Duration::from_secs(4);
    std::thread::sleep(seconds_4);
    let item = get_key(map.clone(), expiry.clone(), "expires".to_owned());
    match item{
        // will exist and do nothing
        Some(_) => panic!("should not exist now"),
        None => {},
    }

}

#[test]
fn test_multiple_threads() {
    // Create a new HashMap and add a key/value pair to it.
    let map = Arc::new(Mutex::new(HashMap::new()));
    let expiry = Arc::new(Mutex::new(HashMap::new()));
    let mut threads = Vec::new();

    // Create ten threads that insert values into the map.
    for i in 0..10 {
        let map = map.clone();
        let handle = thread::spawn(move || {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            insert_key(map, key, value);
        });
        threads.push(handle);
    }

    // Join the threads to ensure that all inserts have completed.
    for handle in threads {
        handle.join().unwrap();
    }

    // Check that all 10 values have been inserted into the map.
    for i in 0..10 {
        let key = format!("key{}", i);
        let expected_value = format!("value{}", i);
        let actual_value = get_key(map.clone(), expiry.clone(), key.clone()).unwrap();
        assert_eq!(actual_value, expected_value);
    }
}

}