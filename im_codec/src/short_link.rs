use im_util::buffer::AutoBuffer;

pub trait Codec: Send {
    fn encode(&self, task_id: u32, out_buffer: &mut AutoBuffer, extend_buffer: &mut AutoBuffer);

    fn decode(&self, task_id: u32, in_buffer: &AutoBuffer, extend_buffer: &mut AutoBuffer);
}

pub const DEFAULT_VERSION: u32 = 100;
pub struct DefaultCodec {
    version: u32,
}

impl DefaultCodec {
    pub fn new(version: u32) -> Self {
        DefaultCodec { version: version }
    }
}

impl Codec for DefaultCodec {
    fn encode(&self, task_id: u32, out_buffer: &mut AutoBuffer, extend_buffer: &mut AutoBuffer) {}

    fn decode(&self, task_id: u32, in_buffer: &AutoBuffer, extend_buffer: &mut AutoBuffer) {}
}
