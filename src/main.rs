use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
};

use rand::Rng;

fn print_top_layer(pyramid: &Arc<Mutex<VecDeque<Vec<Vec<u64>>>>>) {
    {
        let pyramid = pyramid.lock().unwrap();
        let top_layer = pyramid.front().unwrap();
        dbg!(&top_layer);
    }
}

fn push_value(pyramid: &Arc<Mutex<VecDeque<Vec<Vec<u64>>>>>, row: usize, value: u64) {
    pyramid
        .lock()
        .unwrap()
        .front_mut()
        .unwrap()
        .get_mut(row)
        .unwrap()
        .push(value);
    pyramid.clear_poison();
}

fn calculate_row(
    last_layer: &Option<Arc<Vec<Vec<u64>>>>,
    pyramid: &Arc<Mutex<VecDeque<Vec<Vec<u64>>>>>,
    row: usize,
) {
    match &last_layer {
        None => {
            push_value(&pyramid, row, 1);
        }
        Some(last_layer) => {
            for column_index in 0..=row {
                let last_layer_same_row = last_layer.get(row);

                let above = match last_layer.get(row) {
                    None => 0,
                    Some(row) => match row.get(column_index) {
                        None => 0,
                        Some(column) => *column,
                    },
                };

                let last_layer_last_row = match row {
                    0 => None,
                    _ => last_layer.get(row - 1),
                };

                let front_left = match column_index {
                    0 => 0,
                    _ => *last_layer_last_row
                        .unwrap()
                        .get(column_index - 1)
                        .unwrap_or_else(|| {
                            std::process::exit(1);
                        }),
                };

                let front_right = match last_layer_last_row {
                    None => 0,
                    Some(row) => match row.get(column_index) {
                        None => 0,
                        Some(column) => *column,
                    },
                };

                push_value(&pyramid, row, above + front_left + front_right);

                print_top_layer(&pyramid);

                // Artificial random wait time
                //let mut rand = rand::rng();
                //rand.reseed().unwrap();
                //let millis: u64 = rand.random_range(100..300);

                //thread::sleep(std::time::Duration::from_millis(millis));

            }
        }
    };

    return;
}

fn run_layer_calculations(
    //proc_queue: &mut VecDeque<Arc<Box<dyn Fn() -> ()>>>,
    layer: usize,
    n_worker_threads: usize,
    last_layer: Option<Arc<Vec<Vec<u64>>>>,
    pyramid: &Arc<Mutex<VecDeque<Vec<Vec<u64>>>>>,
) {
    {
        let mut pyramid = pyramid.lock().unwrap();

        // create rows to populate
        for _ in 0..=layer {
            pyramid.front_mut().unwrap().push(Vec::new());
        }
    }

    // Loop through calculations

    let pyramid = pyramid.clone();
    let last_layer = last_layer.clone();

    // Won't always need 4 workers on a layer
    if n_worker_threads > layer {
        let mut workers: Vec<thread::JoinHandle<()>> = Vec::new();

        for row in 0..=layer {
            let pyramid = pyramid.clone();
            let last_layer = last_layer.clone();
            workers.push(thread::spawn(move || {
                calculate_row(&last_layer, &pyramid, row)
            }));
        }

        workers.into_iter().for_each(move |worker| {
            worker.join().unwrap();
        });
    } else {
        let mut workers: Vec<thread::JoinHandle<()>> = Vec::new();

        (0..n_worker_threads).for_each(|row| {
            let last_layer = last_layer.clone();
            let pyramid = pyramid.clone();
            workers.push(thread::spawn(move || {
                calculate_row(&last_layer, &pyramid, row)
            }));
        });

        for row in n_worker_threads..=layer {
            let last_layer = last_layer.clone();
            let pyramid = pyramid.clone();

            'rowloop: loop {
                for (index, worker) in workers.iter().enumerate() {
                    if worker.is_finished() {
                        workers.push(thread::spawn(move || {
                            calculate_row(&last_layer, &pyramid, row)
                        }));
                        workers.remove(index);
                        break 'rowloop;
                    } else {
                        continue;
                    }
                }
            }
        }

        // Wait for remaining computations to finish
        workers.into_iter().for_each(|worker| {
            worker.join().unwrap();
        });
    }

}

struct PyramidGenerator {
    pyramid: Arc<Mutex<VecDeque<Vec<Vec<u64>>>>>,
}

impl PyramidGenerator {
    pub fn new() -> Self {
        return Self {
            pyramid: Arc::new(Mutex::new(VecDeque::new())),
        };
    }
    fn run(&self, n_worker_threads: usize) {
        for layer in 0..72 {
            //let mut proc_queue: VecDeque<Arc<Box<dyn Fn() -> ()>>> = VecDeque::new();

            // Clone the last layer to avoid locking the pyramid to observe
            let last_layer = match Arc::clone(&self.pyramid).lock().unwrap().front() {
                Some(layer) => Some(Arc::new(layer.clone())),
                None => None,
            };

            // Create the new layer
            self.pyramid.lock().unwrap().push_front(Vec::new());
            self.pyramid.clear_poison();

            // Create the new row vectors for the layer
            run_layer_calculations(layer, n_worker_threads, last_layer, &self.pyramid);
            //for row_index in 0..layer {}
        }
    }
}

fn sdl_run(pyramid: Arc<Mutex<VecDeque<Vec<Vec<u64>>>>>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    //let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    //let texture = texture_creator.load_texture(png)?;

    //canvas.copy(&texture, None, None)?;
    canvas.present();

    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() {
    let n_worker_threads = 10;
    //let mut pyramid: VecDeque<Vec<Vec<u32>>> = VecDeque::new();

    let pyramid_gen = PyramidGenerator::new();
    let sdl_pyramid_ref = Arc::clone(&pyramid_gen.pyramid);

    let pyramid_gen_thread = thread::spawn(move || pyramid_gen.run(n_worker_threads));
    pyramid_gen_thread.join().unwrap();
    //let sdl_thread = thread::spawn(move || sdl_run(sdl_pyramid_ref));

    //sdl_thread.join().unwrap();
}
