#[cfg(test)]
mod tests {
    use arrow::array::*;
    use flarrow_message::prelude::*;

    #[test]
    fn test_zero_copy() {
        #[derive(ArrowMessage)]
        struct Message {
            buffer: UInt32Array,
        }

        let buffer: Vec<u32> = vec![0, 1, 2, 3, 4, 5];
        let ptr_1 = buffer.as_ptr();

        let message = Message {
            buffer: UInt32Array::from(buffer),
        };
        let arrow = ArrayData::try_from(message).unwrap();
        let message = Message::try_from(arrow).unwrap();
        let ptr_2 = message.buffer.values().as_ptr();

        assert_eq!(ptr_1, ptr_2);
    }
}
