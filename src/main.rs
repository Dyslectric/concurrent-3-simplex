use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
};

use rand::Rng;
use sdl2::{rect::Rect, render::TextureQuery};

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
                pyramid.clear_poison();

                //print_top_layer(&pyramid);

                // Artificial random wait time
                let mut rand = rand::rng();
                rand.reseed().unwrap();
                let millis: u64 = rand.random_range(16..32);

                thread::sleep(std::time::Duration::from_millis(millis));
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
        for layer in 0..40 {
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
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    //let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .resizable()
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let (window_width, window_height) = { window.size() };

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();

    let mut font_size: u16 = 32;
    let mut font = ttf_context.load_font("./assets/inter.ttf", 32)?;

    //canvas.copy(&texture, None, None)?;
    canvas.present();

    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use sdl2::pixels::Color;

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

        let (window_width, window_height) = { canvas.window().size() };

        let pyramid_top_layer = {
            let pyramid = pyramid.lock().unwrap();

            match pyramid.get(0) {
                Some(layer) => Some(layer.clone()),
                None => None,
            }
        };

        let top_layer_text = match pyramid_top_layer {
            Some(layer) => Some({
                let mut layer_string: String = "".to_string();

                for row in layer {
                    for column in &row {
                        layer_string.push_str(column.to_string().as_str());
                        layer_string.push_str("   ");
                    }
                    if row.len() != 0 {
                        layer_string.push_str("\n\n");
                    }
                }

                layer_string
            }),
            None => None,
        };

        //let text

        let top_layer = font
            .render(
                match top_layer_text {
                    None => "".to_string(),
                    Some(s) => s,
                }
                .as_str(),
            )
            //.blended(Color::RGBA(255, 255, 255, 255))
            .blended_wrapped(Color::RGBA(255, 255, 255, 255), 0)
            .map_err(|e| e.to_string())?;

        //let middle_layer =
        //let bottom_layer =

        let texture = texture_creator
            .create_texture_from_surface(&top_layer)
            .map_err(|e| e.to_string())?;

        let TextureQuery {
            width: texture_width,
            height: texture_height,
            ..
        } = texture.query();

        if window_width < texture_width || window_height < texture_height {
            font_size -= 1;
            font = ttf_context.load_font("assets/inter.ttf", font_size)?;
        }

        let target = Rect::new(0, 0, texture_width, texture_height);

        //let target = match (
        //    window_width >= texture_width,
        //    window_height >= texture_height,
        //    window_width as f32 / texture_width as f32,
        //    window_height as f32 / texture_height as f32,
        //) {
        //    (true, true, _, _) => Rect::new(0, 0, texture_width, texture_height),
        //    (true, false, _, height_ratio) => Rect::new(
        //        0,
        //        0,
        //        (texture_width as f32 * height_ratio) as u32,
        //        window_height,
        //    ),
        //    (false, true, width_ratio, _) => Rect::new(
        //        0,
        //        0,
        //        window_width,
        //        (texture_height as f32 * width_ratio) as u32,
        //    ),
        //    (false, false, width_ratio, height_ratio) => match width_ratio < height_ratio {
        //        true => Rect::new(
        //            0,
        //            0,
        //            (texture_width as f32 * width_ratio) as u32,
        //            (texture_height as f32 * width_ratio) as u32,
        //        ),
        //        false => Rect::new(
        //            0,
        //            0,
        //            (texture_width as f32 * height_ratio) as u32,
        //            (texture_height as f32 * height_ratio) as u32,
        //        ),
        //    },
        //};

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.copy(&texture, None, target).unwrap();
        canvas.present();

        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    Ok(())
}

fn main() {
    let n_worker_threads = 12;
    //let mut pyramid: VecDeque<Vec<Vec<u32>>> = VecDeque::new();

    let pyramid_gen = PyramidGenerator::new();
    let sdl_pyramid_ref = Arc::clone(&pyramid_gen.pyramid);

    let pyramid_gen_thread = thread::spawn(move || pyramid_gen.run(n_worker_threads));
    let sdl_thread = thread::spawn(move || sdl_run(sdl_pyramid_ref));

    pyramid_gen_thread.join().unwrap();
    sdl_thread.join().unwrap();
}
