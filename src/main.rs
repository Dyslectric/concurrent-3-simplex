use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

fn main() {
    let worker_threads = 4;

    let mut pyramid: VecDeque<Vec<Vec<u32>>> = VecDeque::new();

    //let mut layer = 1;

    // Compute one layer at a time
    // Compute as many rows as threads given

    for layer in 0..100 {
        let mut proc_queue: VecDeque<Box<dyn Fn() -> ()>> = VecDeque::new();

        let last_layer = Arc::from(pyramid.front());

        // Each row is asynchronously accessible
        let mut this_layer: Vec<Arc<Mutex<Vec<u32>>>> = Vec::new();

        //if layer >= 4 {
        //    pyramid.pop_back();
        //}

        //pyramid.push_front((0..layer).map(|row| {
        //    (0..row).map(|column| None).collect() as Vec<Vec<Option<u32>>
        //}).collect());

        for row_index in 0..layer {

            let row = Arc::new(Mutex::new(Vec::new()));
            this_layer.push(row.clone());

            let last_layer = last_layer.clone();

            //pyramid.push_front(this_layer);

            // Queue calculations
            proc_queue.push_front(Box::new(move || match *last_layer {
                None => {
                    row.lock().unwrap().push(1);
                }
                Some(last_layer) => {
                    for column_index in 0..row_index {
                        let last_layer_same_row = last_layer.get(row_index);

                        let dep1: u32 = match last_layer_same_row {
                            None => 0,
                            Some(row) => *row.get(column_index).unwrap(),
                        };

                        let last_layer_last_row = last_layer.get(row_index - 1).unwrap();

                        let dep2 = match column_index {
                            0 => 0,
                            _ => *last_layer_last_row.get(column_index - 1).unwrap(),
                        };

                        let dep3 = match column_index {
                            row_index => 0,
                            _ => *last_layer_last_row.get(column_index).unwrap(),
                        };

                        row.lock().unwrap().push(dep1 + dep2 + dep3);
                    }
                }
            }));

            // Wait for calculations to finish
            

        }

        //(1..=layer).map()
    }

    //let pyramid: Box<dyn Fn(PyramidKey) -> Arc<Mutex<Option<u32>>>> = Box::new(|key: PyramidKey| {
    //    return Arc::new(Mutex::new(Some(3)));
    //});
}

//fn main() {
//    let (layer, column) = {
//      let args: Vec<String> = std::env::args().collect();
//
//      let lastRow = args.get(1).unwrap_or_else(|| {
//          println!("Need a natural number argument");
//          std::process::exit(1);
//      }).parse::<u32>().unwrap_or_else(|error| {
//          println!("Need a natural number argument: {error}");
//          std::process::exit(1);
//      });
//    };
//}

//fn main() {
//    let args: Vec<String> = std::env::args().collect();
//    let mut queue: VecDeque<Vec<Box<dyn Fn() -> ()>>> = VecDeque::new();
//    let mut results: HashMap<(u32, u32), Arc<Mutex<Option<u32>>>> = HashMap::new();
//    let lastRow = args.get(1).unwrap_or_else(|| {
//        println!("Need a natural number argument");
//        std::process::exit(1);
//    }).parse::<u32>().unwrap_or_else(|error| {
//        println!("Need a natural number argument: {error}");
//        std::process::exit(1);
//    });
//
//    // populate results map for editing
//    for row in 1..=lastRow {
//        for column in 1..=row {
//            results.insert(
//                (row, column),
//                Arc::new(
//                    Mutex::new(
//                        match (row, column) {
//                            (1, _) => Some(1),
//                            (_, 1) => Some(1),
//                            (_, _) => None
//                        }
//                    )
//                )
//            );
//        }
//    }
//
//    for row in 2..=lastRow {
//        queue.push_back((1..=row / 2).map(
//            |column: u32| {
//                let top_left = results.get(&(row - 1, column - 1)).cloned();
//                let top_right = results.get(&(row - 1, column)).cloned();
//                let result = results.get(&(row, column)).cloned().unwrap();
//
//                Box::new(move || {
//                    let mut result = result.lock().unwrap();
//                    let top_left = top_left.as_ref().unwrap().lock().unwrap().unwrap();
//                    let top_right = match top_right.is_some() {
//                        true => top_right.as_ref().unwrap().lock().unwrap().unwrap(),
//                        false => 0
//                    };
//                    *result = Some(top_left + top_right);
//                }) as Box<dyn Fn()>
//            }
//        ).collect());
//    }
//
//    println!("Hello world.");
//}
