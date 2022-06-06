use num::Complex;


fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
 
    None
}

fn main() {
    println!("Hello, world!");

    let a = escape_time(Complex{
        im: 0.1,
        re: 0.2
    }, 1000);

    match a {
        Some(x) => println!("value: {}!", x),
        None => println!("value: None!")
    }
}
