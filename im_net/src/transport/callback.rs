use super::identify_mode;
use super::identify_step;

use im_codec::long_link::CmdId;
use im_util::buffer::AutoBuffer;

pub trait Callback: Send {
    /// 连接鉴权, 例如建立连接后首先需要和服务端进行鉴权通信
    /// identify_buffer: 鉴权数据,写入到这里
    fn get_long_link_identify_check_buffer(
        &mut self,
        identify_buffer: &mut AutoBuffer,
    ) -> (identify_mode::Mode, CmdId);

    /// 验证鉴权是否成功
    fn verify_long_link_identify(&mut self, response_buffer: &AutoBuffer) -> identify_step::Step;
}
