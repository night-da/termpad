mod buffer;
mod error;

#[cfg(test)]
mod smoke {
    use crate::buffer::GapBuffer;

    #[test]
    fn gap_roundtrip() {
        let buf = GapBuffer::from_str("hello");
        assert_eq!(buf.as_text(), "hello");
    }
}

fn main() {
    println!("termpad v0.1.0 (skeleton)");
}
