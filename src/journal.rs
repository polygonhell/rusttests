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
  AppendU32s { id: u64, u32s: Vec<u32> },
  Msg{v: String},
}

#[derive(Debug)]
pub enum JournalError {
  IoError(std::io::Error),
  SerError(ser::Error),
  Error(&'static str),
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

