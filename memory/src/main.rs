use std::mem::size_of;
// "copy on write" - a smart pointer that reads from its ptr location without copy
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_char;

static B: [u8; 10] = [99, 97, 114, 114, 121, 116, 111, 119, 101, 108];
static C: [u8; 11] = [116, 104, 97, 110, 107, 115, 102, 105, 115, 104, 0];

/*
 * Improving usability of functions that accept String or &str.
 * While String lives on the heap, &str lives on the stack. Its annoying
 * to write functions that only accept one of the two when only read-access
 * is required. As a workaround, use meta programming.
 */
fn is_strong<T: AsRef<str>>(password: T) -> bool {
    // this function can be called with &str and String
    return password.as_ref().len() > 5;
}

// Graphics application ---------------------------------------
use graphics::math::{add, mul_scalar, Vec2d};
use piston_window::*;
use rand::prelude::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::time::Instant;

#[global_allocator]
static ALLOCATOR: ReportingAllocator = ReportingAllocator;
struct ReportingAllocator; // ?

unsafe impl GlobalAlloc for ReportingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let start = Instant::now();
        let ptr = System.alloc(layout);
        let end = Instant::now();
        let time_taken = end - start;
        let bytes_requested = layout.size();

        eprintln!("{}\t{}", bytes_requested, time_taken.as_nanos());
        return ptr;
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

struct World {
    current_turn: u64,
    particles: Vec<Box<Particle>>,
    height: f64,
    width: f64,
    rng: ThreadRng,
}

struct Particle {
    height: f64,
    width: f64,
    position: Vec2d<f64>,
    velocity: Vec2d<f64>,
    acceleration: Vec2d<f64>,
    color: [f32; 4],
}

impl Particle {
    fn new(world: &World) -> Particle {
        let mut rng = thread_rng();
        let x = rng.gen_range(0.0..=world.width); // = -> right inclusive range
        let y = world.height;
        let x_velocity = 0.0;
        let y_velocity = rng.gen_range(-2.0..0.0);
        let x_acceleration = 0.0;
        let y_acceleration = rng.gen_range(0.0..0.15);

        return Particle {
            height: 4.0,
            width: 4.0,
            position: [x, y].into(),
            velocity: [x_velocity, y_velocity].into(),
            acceleration: [x_acceleration, y_acceleration].into(),
            color: [1.0, 1.0, 1.0, 0.99],
        };
    }
    fn update(&mut self) {
        self.velocity = add(self.velocity, self.acceleration);
        self.position = add(self.position, self.velocity);
        self.acceleration = mul_scalar(self.acceleration, 0.7);
        self.color[3] *= 0.995;
    }
}

impl World {
    fn new(width: f64, height: f64) -> World {
        return World {
            current_turn: 0,
            particles: Vec::<Box<Particle>>::new(),
            height: height,
            width: width,
            rng: thread_rng(),
        };
    }
    fn add_shapes(&mut self, n: i32) {
        for _ in 0..n.abs() {
            let particle = Particle::new(&self);
            let boxed_particle = Box::new(particle);
            self.particles.push(boxed_particle);
        }
    }

    fn remove_shapes(&mut self, n: i32) {
        for _ in 0..n.abs() {
            let mut to_delete = None;
            let particle_iter = self.particles.iter().enumerate();
            for (i, particle) in particle_iter {
                if particle.color[3] < 0.02 {
                    to_delete = Some(i);
                }
                break;
            }
            if let Some(i) = to_delete {
                self.particles.remove(i);
            } else {
                self.particles.remove(0);
            }
        }
    }
    fn update(&mut self) {
        let n = self.rng.gen_range(-3..=3);

        if n > 0 {
            self.add_shapes(n);
        } else {
            self.remove_shapes(n);
        }

        self.particles.shrink_to_fit();
        for shape in &mut self.particles {
            shape.update();
        }
        self.current_turn += 1;
    }
}
fn main() {
    let a: usize = 42; // memory address size for the CPU
    let b: &[u8; 10] = &B;
    let c: Box<[u8]> = Box::new(C);
    println!("a (u int)");
    println!("  location: {:p}", &a); // pointer formatting
    println!("  size: {:?} bytes", size_of::<usize>());
    println!("  value: {:?}", a);

    println!("b (ref to B)");
    println!("  location: {:p}", &b);
    println!("  size: {:?} bytes", size_of::<&[u8; 10]>());
    println!("  value: {:?}", b);

    println!("c (smart ptr to C)");
    println!("  location: {:p}", &c);
    println!("  size: {:?} bytes", size_of::<Box<[u8]>>());
    println!("  value: {:?}", c);

    println!("B (array of 10 bytes)");
    println!("  location: {:p}", &B);
    println!("  size: {:?} bytes", size_of::<[u8; 10]>());
    println!("  value: {:?}", c);

    println!("C (array of 11 bytes)");
    println!("  location: {:p}", &C);
    println!("  size: {:?} bytes", size_of::<[u8; 11]>());
    println!("  value: {:?}", C);

    // ------------------------------------------------------------------------
    let b_: String;
    let c_: Cow<str>;

    unsafe {
        let b_ptr = &B as *const u8 as *mut u8;
        b_ = String::from_raw_parts(b_ptr, 10, 10);
        let c_ptr = &C as *const u8 as *const c_char;
        c_ = CStr::from_ptr(c_ptr).to_string_lossy();
    }
    println!("a: {}, b_: {}, c_: {}", a, b_, c_);
    // ------------------------------------------------------------------------
    let z: i64 = 42;
    let z_ptr = &z as *const i64;
    let z_addr: usize = unsafe { std::mem::transmute(z_ptr) };
    println!("z: {} ({:p} ...0x{:x})", z, z_ptr, z_addr + 7);

    // Graphics application ---------------------------------------
    let (width, height) = (1280.0, 960.0);
    let mut window: PistonWindow = WindowSettings::new(
        "particles", [width, height]
    ).exit_on_esc(true).build().expect("Could not create window");

    let mut world = World::new(width, height);
    world.add_shapes(1000);

    while let Some(event) = window.next() {
        world.update();

        window.draw_2d(&event, |ctx, renderer, _device| {
            clear([0.15, 0.17, 0.17, 0.9], renderer);
            for s in &mut world.particles {
                let size = [s.position[0], s.position[1], s.width, s.height];
                rectangle(s.color, size, ctx.transform, renderer);
            }
        });
    }
}
