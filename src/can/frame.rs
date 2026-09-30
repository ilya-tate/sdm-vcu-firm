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
            return Err(FrameErr::InvalidId(id));
        }
        if payload.len() > MAX_LEN {
            // Payload too big
            return Err(FrameErr::InvalidPayload(payload.len()));
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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameErr {
    InvalidIdBound(u16),
    InvalidPayloadBound(usize),
}
