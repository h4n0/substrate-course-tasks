fn main() {
    // Test if green light keeps 20 second
    let light = TrafficLight::Green;
    println!("This light keeps {} second.", light.time());
}

enum TrafficLight {
    Red,
    Green,
    Yellow,
}

trait LightTime {
    fn time(&self) -> u8;
}

impl LightTime for TrafficLight {
    // Match light colors with different durations
    // Red - 30s
    // Green - 20s
    // Yellow - 2s
    fn time(&self) -> u8 {
        match self {
            TrafficLight::Red => 30,
            TrafficLight::Green => 20,
            TrafficLight::Yellow => 2,

        }
    }
}
