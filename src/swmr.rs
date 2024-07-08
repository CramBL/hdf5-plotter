use hdf5::globals::H5P_FILE_ACCESS;
use hdf5::h5call;
use hdf5::Result;
use hdf5_sys::h5d::H5Dread;
use hdf5_sys::h5f::{H5Fclose, H5Fopen, H5F_ACC_RDONLY, H5F_ACC_SWMR_READ, H5F_LIBVER_LATEST};
use hdf5_sys::h5s::H5S_ALL;
use hdf5_sys::h5t::H5T_NATIVE_LLONG;

use hdf5_sys::h5p::{H5Pclose, H5Pcreate, H5Pset_libver_bounds, H5P_DEFAULT};
use std::ffi::CString;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::my_hdf5::dataset::H5Dataset;

pub fn multiple_reader() -> hdf5::Result<()> {
    let read_fapl = SwmrReadFapl::new()?;
    let fname = "data.hdf5";
    assert!(PathBuf::from(fname).exists());
    let filename = CString::new(fname).expect("CString::new failed");
    let open_file = OpenFileMrRead::new_swmr_reader(filename, read_fapl.id())?;

    // Initialize buffer
    let mut buffer: Vec<i64> = Vec::new();
    let dataset = H5Dataset::new(CString::new("data").unwrap(), open_file.file_id)?;

    // Read and print data in a loop
    for _ in 0..2 {
        // Refresh 5 times

        // Get the dataspace and its dimensions
        let dims = dataset.refresh()?;

        eprintln!("ndims: {dims:?}");

        let required_size = dims[0] as usize;
        if buffer.len() < required_size {
            buffer.resize(required_size, 0);
        }

        //Read the dataset
        h5call!(H5Dread(
            dataset.id(),
            *H5T_NATIVE_LLONG,
            H5S_ALL,
            H5S_ALL,
            H5P_DEFAULT,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
        ))
        .unwrap();

        let got = dataset.get_newest_n(10)?;
        println!("Data: {:?}", got);
        println!("Data count: {}", got.len());
        println!("Data: {buffer:?}");
        println!("Data count: {}", buffer.len());

        // Wait for a short time before next refresh
        thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}

pub struct SwmrReadFapl {
    fapl_id: i64,
}

impl SwmrReadFapl {
    pub fn new() -> Result<Self> {
        let fapl_id = h5call!(H5Pcreate(*H5P_FILE_ACCESS))?;

        // Set the property list to use the latest library version
        h5call!(H5Pset_libver_bounds(
            fapl_id,
            H5F_LIBVER_LATEST,
            H5F_LIBVER_LATEST
        ))
        .unwrap();

        Ok(Self { fapl_id })
    }

    pub fn id(&self) -> i64 {
        self.fapl_id
    }
}

impl Drop for SwmrReadFapl {
    fn drop(&mut self) {
        h5call!(H5Pclose(self.fapl_id)).unwrap();
    }
}

pub struct OpenFileMrRead {
    file_id: i64,
}

impl OpenFileMrRead {
    pub fn new_swmr_reader(fname: CString, access_plist: i64) -> Result<Self> {
        let file_id = h5call!(H5Fopen(
            fname.as_ptr(),
            H5F_ACC_RDONLY | H5F_ACC_SWMR_READ,
            access_plist,
        ))?;

        Ok(Self { file_id })
    }

    pub fn id(&self) -> i64 {
        self.file_id
    }
}

impl Drop for OpenFileMrRead {
    fn drop(&mut self) {
        h5call!(H5Fclose(self.file_id)).expect("Failed to close file");
    }
}
