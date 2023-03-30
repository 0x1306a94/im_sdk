use super::endpoint;

pub(crate) struct NetSource {
    long: Option<endpoint::Endpoint>,
    short: Option<endpoint::Endpoint>,
}

impl NetSource {
    pub(crate) fn new() -> Self {
        NetSource {
            long: None,
            short: None,
        }
    }
}

unsafe impl Send for NetSource {}

impl NetSource {
    pub(crate) fn set_long_link_endpoint(&mut self, endpoint: endpoint::Endpoint) {
        self.long = Some(endpoint);
    }

    pub(crate) fn get_long_link_endpoint(&self) -> Option<endpoint::Endpoint> {
        self.long.clone()
    }

    pub(crate) fn set_short_link_endpoint(&mut self, endpoint: endpoint::Endpoint) {
        self.long = Some(endpoint);
    }

    pub(crate) fn get_short_link_endpoint(&self) -> Option<endpoint::Endpoint> {
        self.short.clone()
    }
}
