use core::slice;
use std::alloc::{alloc, dealloc, Layout};
use std::cmp::min;

const DEFAULT_UNIT_SIZE: usize = 128;

#[derive(Debug)]
pub enum Seek {
    Start,
    Cur,
    End,
}

#[derive(Debug)]
pub struct AutoBuffer {
    pos: usize,
    capacity: usize,
    len: usize,
    buf_ptr: *mut u8,
    alloc_unit_size: usize,
}

impl AutoBuffer {
    pub fn new(alloc_unit_size: usize) -> Self {
        AutoBuffer {
            pos: 0,
            capacity: 0,
            len: 0,
            buf_ptr: std::ptr::null_mut(),
            alloc_unit_size: alloc_unit_size,
        }
    }

    pub fn new_from(src: &[u8]) -> Self {
        let mut buf_ptr = std::ptr::null_mut();
        if !src.is_empty() {
            let layout = Layout::array::<u8>(src.len()).unwrap();
            buf_ptr = unsafe { alloc(layout) };
        }

        AutoBuffer {
            pos: 0,
            capacity: src.len(),
            len: src.len(),
            buf_ptr: buf_ptr,
            alloc_unit_size: DEFAULT_UNIT_SIZE,
        }
    }
}

impl Drop for AutoBuffer {
    fn drop(&mut self) {
        if self.buf_ptr.is_null() {
            return;
        }

        println!("AutoBuffer<{:p}> drop capacity {}", self, self.capacity);
        unsafe {
            let layout = Layout::array::<u8>(self.capacity).unwrap();
            dealloc(self.buf_ptr, layout);
            self.buf_ptr = std::ptr::null_mut();
        }
    }
}

impl AutoBuffer {
    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn add_capacity(&mut self, size: usize) {
        self.grow(self.pos + size);
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn seek(&mut self, offset: usize, seek: Seek) {
        match seek {
            Seek::Start => self.pos = offset,
            Seek::Cur => self.pos += offset,
            Seek::End => self.pos = self.len + offset,
        }

        self.pos = min(self.pos, self.len);
    }

    pub fn write(&mut self, src: &[u8]) {
        self.write_at_pos(self.pos, src);
        self.seek(src.len(), Seek::Cur);
    }

    pub fn write_at_pos(&mut self, pos: usize, src: &[u8]) {
        if src.len() == 0 {
            return;
        }
        // 扩容
        let new_cap = pos + src.len();
        self.grow(new_cap);
        unsafe {
            let dst = self.buf_ptr.offset(pos as isize);
            std::ptr::copy(src.as_ptr(), dst, src.len());
        };
        self.len += src.len();
    }

    pub fn read(&mut self, len: usize) -> Vec<u8> {
        let (read_len, out) = self.read_at_pos(self.pos, len);
        self.seek(read_len, Seek::Cur);
        out
    }

    pub fn read_at_pos(&self, pos: usize, len: usize) -> (usize, Vec<u8>) {
        assert!(!self.buf_ptr.is_null());
        assert!(pos <= self.len);

        let mut read_len = self.len - pos;
        read_len = min(read_len, len);

        let mut out = Vec::<u8>::with_capacity(read_len);

        if read_len == 0 {
            return (read_len, out);
        }

        unsafe {
            let src = self.buf_ptr.offset(self.pos as isize);
            std::ptr::copy(src, out.as_mut_ptr(), read_len);

            out.set_len(read_len);
        }

        (read_len, out)
    }

    pub fn get_conetnt(&self, offset: usize) -> &[u8] {
        unsafe {
            let src = self.buf_ptr.offset(offset as isize) as *const u8;
            let len = self.len - offset;
            slice::from_raw_parts(src, len)
        }
    }

    pub fn get_pos_content(&self) -> &[u8] {
        unsafe {
            let src = self.buf_ptr.offset(self.pos as isize) as *const u8;
            let len = self.len - self.pos;
            slice::from_raw_parts(src, len)
        }
    }
}

impl From<&[u8]> for AutoBuffer {
    fn from(value: &[u8]) -> Self {
        let mut buf = AutoBuffer::default();
        buf.write(value);
        buf
    }
}

impl Default for AutoBuffer {
    fn default() -> Self {
        AutoBuffer::new(DEFAULT_UNIT_SIZE)
    }
}

impl AutoBuffer {
    fn grow(&mut self, size: usize) {
        let old_capacity = self.capacity;
        if size < old_capacity {
            return;
        }

        let new_capacity =
            ((size + self.alloc_unit_size - 1) / self.alloc_unit_size) * self.alloc_unit_size;
        unsafe {
            // 分配新空间
            let new_layout = Layout::array::<u8>(new_capacity).unwrap();
            let new_ptr = alloc(new_layout);
            if !new_ptr.is_null() && !self.buf_ptr.is_null() {
                // 拷贝旧值
                std::ptr::copy(self.buf_ptr, new_ptr, old_capacity);
            }

            // 释放旧空间
            if !self.buf_ptr.is_null() {
                let old_layout = Layout::array::<u8>(old_capacity).unwrap();
                dealloc(self.buf_ptr, old_layout);
            }

            self.buf_ptr = new_ptr;
            self.capacity = new_capacity;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn case_into() {
        let data: Vec<u8> = vec![12, 18, 19];
        let buf: AutoBuffer = (&data[..]).into();
        println!("{:?}", buf);
        println!("{:?}", data);
    }

    #[test]
    fn case_new_from() {
        let data: Vec<u8> = vec![12, 18, 19];
        let buf = AutoBuffer::new_from(&data);
        println!("{:?}", buf);
        println!("{:?}", data);
    }

    #[test]
    fn case_write() {
        let mut buf = AutoBuffer::default();
        let id: i32 = 10;
        let bytes = id.to_le_bytes();
        buf.write(&bytes);
        println!("case_write: {:?}", buf);

        let inner_buffer = buf.read(bytes.len());
        assert_eq!(&[0u8; 0], &inner_buffer[..]);

        buf.seek(0, Seek::Start);
        let inner_buffer = buf.read(bytes.len());
        assert_eq!(bytes, &inner_buffer[..]);
    }
}
