use rand::distributions::Uniform;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::time::Instant;

#[derive(enum_map::Enum)]
enum Distribution {
    Frame,
    Square,
    Circle,
}

// sorts by x first, then y
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
struct Point {
    tangent: f64,
    x:       f64,
    y:       f64,
}

impl Point {
    fn dot(&self, other: Point) -> f64 {
        -self.x * other.y + self.y * other.x
    }
}

impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, other: Self) -> Point {
        Point { x: self.x - other.x, y: self.y - other.y, tangent: 0. }
    }
}

fn gen_frame(rng: &mut ThreadRng, uniform: &Uniform<f64>) -> (f64, f64) {
    let x: f64 = rng.sample(uniform);
    let y: u8 = rng.gen_range(0, 2);
    let y = y as f64;

    if rng.gen_bool(0.5) {
        (x, y)
    } else {
        (y, x)
    }
}

fn gen_square(rng: &mut ThreadRng, uniform: &Uniform<f64>) -> (f64, f64) {
    (rng.sample(uniform), rng.sample(uniform))
}

fn gen_circle(rng: &mut ThreadRng, uniform: &Uniform<f64>) -> (f64, f64) {
    //smell
    let angle = rng.sample(uniform) * std::f64::consts::PI * 2.;

    let x = angle.cos();
    let y = angle.sin();

    (x, y)
}

fn main() {
    let n = 10_000_000;

    let mut rng = thread_rng();
    let uniform = Uniform::new(0.0, 1.0);

    let generators = enum_map::enum_map! {
        Distribution::Square => gen_square,
        Distribution::Frame => gen_frame,
        Distribution::Circle => gen_circle
    };

    const DISTRIBUTION: Distribution = Distribution::Frame;

    let mut points: Vec<Point> = (0..n)
        .map(|_| {
            let (x, y) = generators[DISTRIBUTION](&mut rng, &uniform);

            Point { x, y, tangent: 0. }
        })
        .collect();
    // println!("points: {:?}\n", points);

    let now = Instant::now();

    //partial because NaN != NaN,
    //first unwrap is because partial_cmp yields None if values are incomparable
    //last unwrap is because iterator may be empty
    let origin_id =
        points.iter().enumerate().min_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
    let origin = points.swap_remove(origin_id);
    println!("origin: {:?}\n", origin);

    for p in points.iter_mut() {
        p.tangent = (p.y - origin.y) / (p.x - origin.x);
    }

    //I think it sorts from left to right
    //clockwise 100%
    points.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    // println!("sorted by angle: {:?}\n", points);

    let mut hull = vec![origin, *points.first().unwrap()];

    //here should be O(n) pass with checks
    for &point in points.iter().skip(1) {
        //if points[i] is on the left of vector from hull.pre_last() to hull.last()
        //then we should replace hull.last() with points[i] ang keep checking, otherwise we push
        while let [.., prelast, last] = hull[..] {
            // here I could draw hull[..len-1] in red and old in blue & new in green
            let old = last - prelast;
            let new = point - prelast;

            //if old is on the left of new
            if old.dot(new) < 0. {
                break;
            } else {
                hull.pop();
            }
        }

        hull.push(point)
    }

    println!("time: {}", now.elapsed().as_secs_f64());

    println!("Hull lengh: {}", hull.len());
    // println!("Hull: {:?}\n", hull);
}
