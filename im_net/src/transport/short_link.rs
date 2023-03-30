use std::cell::RefCell;
use std::sync::Arc;

use super::net_source;
use crate::codec::short_link;

pub(crate) struct ShortLink {
    source: Arc<RefCell<net_source::NetSource>>,
    codec: Box<dyn short_link::Codec>,
}

impl ShortLink {
    pub(crate) fn new(
        source: Arc<RefCell<net_source::NetSource>>,
        codec: Box<dyn short_link::Codec>,
    ) -> Self {
        ShortLink {
            source: source,
            codec: codec,
        }
    }
}
