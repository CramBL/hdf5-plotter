use hdf5::h5call;
use hdf5::Result;
use hdf5_sys::h5::hsize_t;
use hdf5_sys::h5d::{H5Dclose, H5Dget_space, H5Dopen, H5Dread, H5Drefresh};

use hdf5_sys::h5s::H5S_seloper_t::H5S_SELECT_SET;
use hdf5_sys::h5s::{H5Sget_simple_extent_dims, H5Sget_simple_extent_ndims, H5Sselect_hyperslab};
use hdf5_sys::h5t::H5T_NATIVE_LLONG;

use hdf5_sys::h5p::H5P_DEFAULT;
use std::ffi::CString;
use std::vec;

use crate::my_hdf5::dataspace::DataSpace;
use crate::my_hdf5::htype::H5Datatype;

pub struct H5Dataset {
    dataset_id: i64,
}

impl H5Dataset {
    pub fn new(name: CString, file_id: i64) -> Result<Self> {
        let dataset_id = h5call!(H5Dopen(file_id, name.as_ptr(), H5P_DEFAULT)).unwrap();

        Ok(Self { dataset_id })
    }

    pub fn ndims(&self) -> Vec<u64> {
        // Get the dataspace and its dimensions
        let dataspace_id = h5call!(H5Dget_space(self.dataset_id)).unwrap();
        let ndims = h5call!(H5Sget_simple_extent_ndims(dataspace_id)).unwrap();

        let mut dims: Vec<hsize_t> = vec![0; ndims as usize];
        h5call!(H5Sget_simple_extent_dims(
            dataspace_id,
            dims.as_mut_ptr(),
            std::ptr::null_mut()
        ))
        .unwrap();

        dims.into_iter().collect()
    }

    pub fn refresh(&self) -> Result<Vec<u64>> {
        eprintln!("Current dims: {:?}", self.ndims());
        h5call!(H5Drefresh(self.id()))?;
        eprintln!("After refresh dims: {:?}", self.ndims());
        Ok(self.ndims())
    }

    pub fn id(&self) -> i64 {
        self.dataset_id
    }

    pub fn read_all(&self, buf: &mut Vec<i64>) -> Result<()> {
        let required_size = self.ndims()[0] as usize;
        if buf.len() < required_size {
            buf.resize(required_size, 0);
        }
        h5call!(H5Dread(
            self.id(),
            *H5T_NATIVE_LLONG,
            hdf5_sys::h5s::H5S_ALL,
            hdf5_sys::h5s::H5S_ALL,
            H5P_DEFAULT,
            buf.as_mut_ptr() as *mut std::ffi::c_void,
        ))?;
        Ok(())
    }

    /// Get newest up to N samples
    pub fn get_newest_n(&self, n: usize) -> Result<Vec<i64>> {
        let ndims = self.refresh()?;
        let total_size = ndims[0] as usize;
        eprintln!("Total size = {total_size}");
        let start = if total_size > n { total_size - n } else { 0 };
        let count = total_size.min(n);
        eprintln!("count={count}");
        let mut buf = vec![0i64; count];
        let dataspace = DataSpace::new(self.id(), count)?;
        // Create a hyperslab selection for the last WINDOW_SIZE elements (or fewer)
        h5call!(H5Sselect_hyperslab(
            dataspace.dataspace_id(),
            H5S_SELECT_SET,
            &start as *const usize as *const hsize_t,
            std::ptr::null(),
            &count as *const usize as *const hsize_t,
            std::ptr::null()
        ))?;

        // Read the dataset
        h5call!(H5Dread(
            self.id(),
            *H5T_NATIVE_LLONG,
            dataspace.memoryspace_id(),
            dataspace.dataspace_id(),
            H5P_DEFAULT,
            buf.as_mut_ptr() as *mut std::ffi::c_void,
        ))?;

        eprintln!("Buf len={}", buf.len());

        eprintln!("{buf:?}");

        Ok(buf)
    }

    pub fn get_datatype(&self) -> Result<H5Datatype> {
        H5Datatype::new(self.id())
    }
}

impl Drop for H5Dataset {
    fn drop(&mut self) {
        h5call!(H5Dclose(self.dataset_id)).expect("Failed to close dataset");
    }
}
