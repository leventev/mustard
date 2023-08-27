use std::{ffi::CStr, path::Iter};

#[repr(C, packed)]
#[derive(Debug)]
struct TarHeaderRaw {
    name: [u8; 100],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    size: [u8; 12],
    mtime: [u8; 12],
    chksum: [u8; 8],
    typeflag: u8,
    linkname: [u8; 100],
    magic: [u8; 6],
    version: [u8; 2],
    uname: [u8; 32],
    gname: [u8; 32],
    devmajor: [u8; 8],
    devminor: [u8; 8],
    prefix: [u8; 155],
}

#[derive(Debug)]
pub struct TarHeader<'a> {
    name: &'a str,
    uname: &'a str,
    gname: &'a str,
    uid: usize,
    gid: usize,
    mtime: usize,
    size: usize,
}

#[derive(Debug)]
pub struct TarReader<'a> {
    buff: &'a [u8],
}

const BLOCK_SIZE: usize = 512;

#[derive(Debug)]
pub struct TarHeaderReader<'a> {
    buff: &'a [u8],
    offset: usize,
}

impl<'a> TarReader<'a> {
    pub fn from_buff(buff: &'a [u8]) -> TarReader<'a> {
        TarReader { buff }
    }

    pub fn headers(&self) -> TarHeaderReader<'a> {
        TarHeaderReader {
            buff: self.buff,
            offset: 0,
        }
    }
}

fn parse_str_from_bytes(bytes: &[u8]) -> Option<&str> {
    CStr::from_bytes_until_nul(bytes).ok()?.to_str().ok()
}

fn parse_octal_from_bytes(bytes: &[u8]) -> Option<usize> {
    let str = parse_str_from_bytes(bytes)?;
    usize::from_str_radix(str, 8).ok()
}

fn calc_checksum(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .fold(0usize, |aggr, val| aggr.wrapping_add(*val as usize))
}

impl<'a> Iterator for TarHeaderReader<'a> {
    type Item = TarHeader<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        const HEADER_SIZE: usize = std::mem::size_of::<TarHeaderRaw>();
        let header_buff = &self.buff[self.offset..self.offset + HEADER_SIZE];
        let header_raw = unsafe {
            let header_ptr = header_buff.as_ptr() as *const TarHeaderRaw;
            header_ptr.as_ref()
        }?;

        let header = TarHeader {
            name: parse_str_from_bytes(&header_raw.name)?,
            uname: parse_str_from_bytes(&header_raw.uname)?,
            gname: parse_str_from_bytes(&header_raw.gname)?,
            uid: parse_octal_from_bytes(&header_raw.uid)?,
            gid: parse_octal_from_bytes(&header_raw.gid)?,
            mtime: parse_octal_from_bytes(&header_raw.mtime)?,
            size: parse_octal_from_bytes(&header_raw.size)?,
        };

        let chksum = parse_octal_from_bytes(&header_raw.chksum)?;
        // the bytes of the chksum are treated as ASCII spaces(32)
        let calculated_chksum = calc_checksum(header_buff)
            - calc_checksum(&header_raw.chksum)
            + header_raw.chksum.len() * 32;

        if chksum != calculated_chksum {
            return None;
        }

        Some(header)
    }
}
