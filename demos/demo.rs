//! termpad Rust demo — idiomatic sample for manual `termpad demos/demo.rs` showcase.
//! Also used as a syntax highlighter regression fixture in `syntax/mod.rs` tests.

const MAX_ITEMS: usize = 12;
const GREETING: &str = "Hello from Rust demo";

#[derive(Debug, Clone, PartialEq)]
struct Item {
    id: u32,
    score: f64,
    label: String,
}

impl Item {
    fn new(id: u32, score: f64, label: impl Into<String>) -> Self {
        Self {
            id,
            score,
            label: label.into(),
        }
    }

    fn describe(&self) -> String {
        format!("id={} score={:.2} label={}", self.id, self.score, self.label)
    }
}

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn sort_by_score(items: &mut [Item]) {
    items.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn main() {
    let mut items: Vec<Item> = (0..MAX_ITEMS)
        .map(|i| Item::new(i as u32 + 1, (i as f64) * 1.618, format!("item_{i}")))
        .collect();

    sort_by_score(&mut items);

    println!("{GREETING}");
    for item in &items {
        println!("{}", item.describe());
    }

    let n = 10u32;
    println!("fib({n}) = {}", fibonacci(n));

    // Option / Result showcase
    let parsed = "42".parse::<i32>();
    match parsed {
        Ok(value) => println!("parsed number: {value}"),
        Err(err) => eprintln!("parse failed: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fibonacci_base_cases() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(6), 8);
    }
}
