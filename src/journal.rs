use serde::{Deserialize, Serialize};
use serde_json::{Value};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufWriter;


use serde_json as ser;

pub struct DiskJournal<'a> {
  fname: &'a str,
  writer: BufWriter<File>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Entry {
  Write {
    page: u32,
    offset: u16,
    bytes: Vec<u8>,
  },
  Msg{v: String},
}

#[derive(Debug)]
pub enum JournalError {
  IoError(std::io::Error),
  SerError(ser::Error),
  Error(&'static str),
}


impl Entry {
    fn write_slice<T>(page: u32, offset: u16, vs: &[T]) -> Entry {
    let ptr = vs.as_ptr() as *const u8;
    let length = vs.len() * std::mem::size_of::<T>();
    let new_slice = unsafe { std::slice::from_raw_parts(ptr, length) };
    // Copy from the slice
    let bytes = new_slice.to_vec();

    Entry::Write {
      page: page,
      offset: offset,
      bytes: bytes,
    }
  }

  fn write<T>(page: u32, offset: u16, vs: &mut Vec<T>) -> Entry {
    // Hand off the vector in here
    let ptr = vs.as_mut_ptr() as *mut u8;
    let length = vs.len() * std::mem::size_of::<T>();
    let capacity = vs.capacity() * std::mem::size_of::<T>();
    let bytes = unsafe { Vec::from_raw_parts(ptr, length, capacity) };
    std::mem::forget(vs);
    // Memory now owned by returned struct

    Entry::Write {
      page: page,
      offset: offset,
      bytes: bytes,
    }
  }

}


pub trait Journal {
  fn add(&mut self, entry: &Entry) -> Result<(), JournalError>;
}



impl<'a> Journal for DiskJournal<'a> {
  fn add(&mut self, entry: &Entry) -> Result<(), JournalError> {
    ser::to_writer(self.writer.by_ref(), entry).map_err(JournalError::SerError)?;
    Ok(())
  }


}


impl<'a> DiskJournal<'a> {
  pub fn new(file_name: &str) -> Result<DiskJournal, JournalError> {
    let file = OpenOptions::new()
      .create(true)
      .append(true)
      .write(true)
      .open(file_name).map_err(JournalError::IoError)?;

    Ok(DiskJournal {
      fname: file_name,
      writer: std::io::BufWriter::new(file),
    })
  }

  pub fn flush(self) -> Result<DiskJournal<'a>, JournalError> {
    let file = match std::io::BufWriter::into_inner(self.writer) {
      Ok(x) => Ok(x),
      Err(_) => Err(JournalError::Error("Failed to get file from writer"))
    }?;
    
    file.sync_all().map_err(JournalError::IoError)?;
    Ok(DiskJournal {
      fname: self.fname,
      writer: BufWriter::new(file),
    })
  }

  pub fn read(&self) -> Result<Vec<Entry>, JournalError> {
    // New file here so we're not competing with the writer
    let file = OpenOptions::new().read(true).open(self.fname).map_err(JournalError::IoError)?;
    let mut reader = std::io::BufReader::new(file);
    reader.seek(std::io::SeekFrom::Start(0)).map_err(JournalError::IoError)?;
    let mut des = ser::Deserializer::from_reader(reader).into_iter::<Entry>();
    let out = Vec::new();
    let res = des.try_fold(out, |mut acc, r| { 
        match r {
          Err(e) => Err(JournalError::SerError(e)),
          Ok(v) => { acc.push(v); Ok(acc) },
        }
      }
    );
    res
  }
}

