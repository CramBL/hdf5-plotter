use hdf5::h5call;
use hdf5::Result;
use hdf5_sys::h5::hsize_t;
use hdf5_sys::h5d::H5Dget_space;
use hdf5_sys::h5s::{H5Sclose, H5Screate_simple};

pub struct DataSpace {
    ds_id: i64,
    ms_id: i64,
}

impl DataSpace {
    pub fn new(dataset_id: i64, size: usize) -> Result<Self> {
        let dataspace_id = h5call!(H5Dget_space(dataset_id)).unwrap();
        // Create memory dataspace
        let mem_space_id = h5call!(H5Screate_simple(
            1,
            &size as *const usize as *const hsize_t,
            std::ptr::null()
        ))?;
        Ok(Self {
            ds_id: dataspace_id,
            ms_id: mem_space_id,
        })
    }

    pub fn dataspace_id(&self) -> i64 {
        self.ds_id
    }
    pub fn memoryspace_id(&self) -> i64 {
        self.ms_id
    }
}

impl Drop for DataSpace {
    fn drop(&mut self) {
        h5call!(H5Sclose(self.ds_id)).expect("Failed to close dataspace");
        h5call!(H5Sclose(self.ms_id)).expect("Failed to close dataspace");
    }
}
