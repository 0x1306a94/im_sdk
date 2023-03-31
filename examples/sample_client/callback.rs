use im_codec::long_link::CmdId;
use im_net::transport::callback;
use im_net::transport::identify_mode;
use im_net::transport::identify_step;
use im_util::buffer::AutoBuffer;

pub(crate) struct TCB {}

impl callback::Callback for TCB {
    fn get_long_link_identify_check_buffer(
        &mut self,
        identify_buffer: &mut AutoBuffer,
    ) -> (identify_mode::Mode, CmdId) {
        let buf = vec![1_u8, 2_u8, 3_u8, 4_u8];
        println!("填充鉴权数据...");
        identify_buffer.write(&buf);
        (identify_mode::Mode::Now, 1)
    }

    fn verify_long_link_identify(&mut self, response_buffer: &AutoBuffer) -> identify_step::Step {
        identify_step::Step::Ok
    }
}
