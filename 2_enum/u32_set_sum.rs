fn main() {
    let set1: [u32; 4] = [2, 3, 5, 7];
    println!("Sum is {}", sum(&set1[..]).unwrap());
    
    let set2: [u32; 4] = [2, 3, 5, u32::MAX];
    assert_eq!(sum(&set2[..]), None);
}

fn sum(num_set: &[u32]) -> Option<u32> {
    let mut sum: u32 = 0;
    for i in num_set {
        match sum.checked_add(*i) {
            Some(v) => {
                sum = v;
            },
            None => {
                return None;
            },
        }
    }
    Some(sum)
}

