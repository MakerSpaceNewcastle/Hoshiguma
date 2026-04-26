use heapless::Vec;

#[derive(Default)]
pub struct CobsFramer {
    buffer: Vec<u8, { Self::BUFFER_SIZE }>,
}

impl CobsFramer {
    pub const BUFFER_SIZE: usize = 4096;

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<(), ()> {
        self.buffer.extend_from_slice(bytes).map_err(|_| ())
    }

    pub fn next_message(&mut self) -> Option<Vec<u8, { Self::BUFFER_SIZE }>> {
        // Find the next zero byte, which indicates the end of a COBS frame
        if let Some(pos) = self.buffer.iter().position(|&b| b == 0) {
            // Extract the frame up to and including the zero byte
            let frame = self.buffer.drain(..=pos).collect();
            Some(frame)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn initial_state() {
        let mut framer = CobsFramer::default();
        assert_eq!(framer.len(), 0);
        assert!(framer.next_message().is_none());
    }

    #[test]
    fn push_when_buffer_full() {
        let mut framer = CobsFramer::default();
        framer.push(&[1; CobsFramer::BUFFER_SIZE]).unwrap();
        assert_eq!(framer.len(), CobsFramer::BUFFER_SIZE);
        assert!(framer.push(&[1]).is_err());
    }

    #[test]
    fn simple() {
        let mut framer = CobsFramer::default();

        framer.push(&[1, 2, 3, 0]).unwrap();
        assert_eq!(framer.len(), 4);
        assert_eq!(&framer.next_message().unwrap(), &[1, 2, 3, 0]);

        assert_eq!(framer.len(), 0);
        assert!(framer.next_message().is_none());
    }

    #[test]
    fn single_frame_partial() {
        let mut framer = CobsFramer::default();

        framer.push(&[1, 2, 3]).unwrap();
        assert_eq!(framer.len(), 3);
        assert!(framer.next_message().is_none());

        framer.push(&[4, 5, 0]).unwrap();
        assert_eq!(framer.len(), 6);
        assert_eq!(&framer.next_message().unwrap(), &[1, 2, 3, 4, 5, 0]);

        assert_eq!(framer.len(), 0);
        assert!(framer.next_message().is_none());
    }

    #[test]
    fn multi_frame_partial() {
        let mut framer = CobsFramer::default();

        framer.push(&[1, 2, 3]).unwrap();
        assert_eq!(framer.len(), 3);
        assert!(framer.next_message().is_none());

        framer.push(&[4, 5, 0, 6, 7]).unwrap();
        assert_eq!(framer.len(), 8);
        assert_eq!(&framer.next_message().unwrap(), &[1, 2, 3, 4, 5, 0]);

        assert_eq!(framer.len(), 2);
        assert!(framer.next_message().is_none());

        framer.push(&[0, 8]).unwrap();
        assert_eq!(framer.len(), 4);
        assert_eq!(&framer.next_message().unwrap(), &[6, 7, 0]);

        assert_eq!(framer.len(), 1);
        assert!(framer.next_message().is_none());
    }
}
