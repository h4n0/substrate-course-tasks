fn main() {
    let rect = Rectangle{ length:10.8, width:2.0 };
    print_area(&rect);
    let cir = Circle{ radius:2.0 };
    print_area(&cir);
    let tri = Triangle{ height:2.4, base: 0.5 };
    print_area(&tri);
}

fn print_area<T: Area>(shape: &T) {
    println!("The area of given shape is {}", shape.calc_area());
}

trait Area {
    fn calc_area(&self) -> f64;
}

struct Rectangle {
    length: f64,
    width: f64,
}

impl Area for Rectangle {
    fn calc_area(&self) -> f64 {
        self.length * self.width
    }
}

struct Circle {
    radius: f64,
}

impl Area for Circle {
    fn calc_area(&self) -> f64 {
        self.radius * self.radius * 3.14
    }
}

struct Triangle {
    height: f64,
    base: f64,
}

impl Area for Triangle {
    fn calc_area(&self) -> f64 {
        self.height * self.base * 0.5
    }
}
