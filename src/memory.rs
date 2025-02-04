use crate::error::Exception;
use std::fmt;

pub const MEMORY_LEN: usize = 10 * 1024;
pub const STACK_LEN: usize = 1024;

pub trait Addressable: fmt::Debug {
    fn read(&self, addr: u32) -> Result<u8, Exception>;
    fn write(&mut self, addr: u32, value: u8) -> Result<(), Exception>;

    fn read_u16(&self, addr: u32) -> Result<u16, Exception> {
        Ok(u16::from_le_bytes([self.read(addr)?, self.read(addr + 1)?]))
    }

    fn write_u16(&mut self, addr: u32, value: u16) -> Result<(), Exception> {
        value
            .to_le_bytes()
            .into_iter()
            .enumerate()
            .try_for_each(|(i, byte)| self.write(addr + i as u32, byte))
    }

    fn read_u32(&self, addr: u32) -> Result<u32, Exception> {
        Ok(u32::from_le_bytes([
            self.read(addr)?,
            self.read(addr + 1)?,
            self.read(addr + 2)?,
            self.read(addr + 3)?,
        ]))
    }

    fn write_u32(&mut self, addr: u32, value: u32) -> Result<(), Exception> {
        value
            .to_le_bytes()
            .into_iter()
            .enumerate()
            .try_for_each(|(i, byte)| self.write(addr + i as u32, byte))
    }

    fn copy(&mut self, from: u32, to: u32, n: usize) -> Result<(), Exception> {
        (0..n).try_for_each(|idx| {
            let idx = idx as u32;
            self.write(to + idx, self.read(from + idx)?)
        })
    }
}

impl<const N: usize> Addressable for [u8; N] {
    fn read(&self, addr: u32) -> Result<u8, Exception> {
        self.get(addr as usize)
            .copied()
            .ok_or(Exception::InvalidMemoryAccess(addr))
    }

    fn write(&mut self, addr: u32, value: u8) -> Result<(), Exception> {
        if self.len() > MEMORY_LEN {
            return Err(Exception::InvalidMemoryAccess(addr));
        }

        self[addr as usize] = value;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Stack<T: Copy + fmt::Debug + Default, const N: usize> {
    data: [T; N],
    pub len: usize,
}

impl<T, const N: usize> fmt::Debug for Stack<T, N>
where
    T: Copy + fmt::Debug + Default,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.len == self.data.len() {
            f.debug_list()
                .entries(self.data[..self.len].iter())
                .finish()
        } else {
            f.debug_list()
                .entries(self.data[..self.len].iter())
                .finish_non_exhaustive()
        }
    }
}

impl<T, const N: usize> Stack<T, N>
where
    T: Copy + fmt::Debug + Default,
{
    pub fn new() -> Self {
        Self {
            data: [T::default(); N],
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), Exception> {
        if self.data.len() <= self.len {
            return Err(Exception::StackOverflow);
        }

        self.data[self.len] = value;
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<T, Exception> {
        if self.len == 0 {
            return Err(Exception::StackUnderflow);
        }

        self.len -= 1;
        Ok(self.data[self.len])
    }

    pub fn peek(&self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        Some(self.data[self.len - 1])
    }

    pub fn peek_at(&self, at: usize) -> Option<T> {
        if at >= self.len {
            return None;
        }

        Some(self.data[self.len - 1 - at])
    }
}

impl<T, const N: usize> Default for Stack<T, N>
where
    T: Copy + fmt::Debug + Default,
{
    fn default() -> Self {
        Self::new()
    }
}
