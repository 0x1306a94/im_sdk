use im_util::buffer::AutoBuffer;

pub trait Codec {
    fn set_version(version: u32);

    fn encode(task_id: u32, out_buffer: &mut AutoBuffer, extend_buffer: &mut AutoBuffer);
    
    fn decode(task_id: u32, in_buffer: &AutoBuffer, extend_buffer: &AutoBuffer);
}
