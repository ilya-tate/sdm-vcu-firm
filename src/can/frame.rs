pub const MAX_ID: u16 = 0x7ff; // 0000 0111 1111 1111
pub const MAX_LEN: usize = 8; // u4

// can2.0a
// 0-8 byte payload
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    id: u16, // Contains 5 leading 0s
    payload: [u8; MAX_LEN],
    len: u8,
}

impl Frame {
    pub fn new(id: u16, payload: &[u8]) -> Result<Self, FrameErr> {
        if id > MAX_ID {
            // Id out of range
            return Err(FrameErr::InvalidIdBound(id));
        }
        if payload.len() > MAX_LEN {
            // Payload too big
            return Err(FrameErr::InvalidPayloadBound(payload.len()));
        }

        let mut data = [0; MAX_LEN];
        data[..payload.len()].copy_from_slice(payload);

        Ok(Self {
            id,
            payload: data,
            len: payload.len() as u8,
        })
    }

    pub fn id(&self) -> u16 {
        self.id
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload[..self.len()]
    }
    pub fn len(&self) -> usize {
        usize::from(self.len)
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(super) fn mailbox_words(&self) -> [u32; 2] {
        [
            u32::from_be_bytes(self.payload[..4].try_into().unwrap()),
            u32::from_be_bytes(self.payload[4..].try_into().unwrap()),
        ]
    }

    pub(super) fn from_mailbox(id: u32, dlc: u32, words: [u32; 2]) -> Self {
        let mut payload = [0u8; MAX_LEN];
        payload[..4].copy_from_slice(&words[0].to_be_bytes());
        payload[4..].copy_from_slice(&words[1].to_be_bytes());
        // Classical CAN DLC 9..15 denotes eight bytes too.
        Self::new(
            ((id >> 18) & 0x7ff) as u16,
            &payload[..((dlc & 0xf) as usize).min(8)],
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mailbox_byte_order_and_padding() {
        let frame = Frame::new(0x123, &[1, 2, 3, 4, 5]).unwrap();
        assert_eq!(frame.mailbox_words(), [0x01020304, 0x05000000]);
        assert_eq!(
            Frame::from_mailbox(0x123 << 18, 5, frame.mailbox_words()),
            frame
        );
    }

    #[test]
    fn all_lengths_round_trip() {
        for len in 0..=8 {
            let frame = Frame::new(
                0x7ff,
                &[0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xff][..len],
            )
            .unwrap();
            assert_eq!(
                Frame::from_mailbox(0x7ff << 18, len as u32, frame.mailbox_words()),
                frame
            );
            assert_eq!(frame.is_empty(), len == 0);
        }
    }

    #[test]
    fn bounds_and_large_classical_dlc() {
        assert_eq!(Frame::new(0x800, &[]), Err(FrameErr::InvalidIdBound(0x800)));
        assert_eq!(
            Frame::new(0, &[0; 9]),
            Err(FrameErr::InvalidPayloadBound(9))
        );
        for dlc in 8..=15 {
            assert_eq!(
                Frame::from_mailbox(0, dlc, [u32::MAX; 2]).payload(),
                &[0xff; 8]
            );
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameErr {
    InvalidIdBound(u16),
    InvalidPayloadBound(usize),
}
