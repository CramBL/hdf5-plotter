use hdf5::h5call;
use hdf5_sys::h5::hsize_t;
use hdf5_sys::h5d::H5Dget_type;
use hdf5_sys::h5t::{
    id_t, H5T_class_t, H5T_sign_t, H5Tclose, H5Tget_class, H5Tget_sign, H5Tget_size,
};

#[derive(Debug)]
pub struct H5Datatype {
    type_id: id_t,
}

impl H5Datatype {
    pub fn new(dataset_id: id_t) -> hdf5::Result<Self> {
        let type_id = h5call!(H5Dget_type(dataset_id))?;
        Ok(Self { type_id })
    }

    pub fn class(&self) -> H5T_class_t {
        unsafe { H5Tget_class(self.type_id) }
    }

    pub fn size(&self) -> hsize_t {
        unsafe { H5Tget_size(self.type_id) as u64 }
    }

    pub fn sign(&self) -> H5T_sign_t {
        unsafe { H5Tget_sign(self.type_id) }
    }
}

impl Drop for H5Datatype {
    fn drop(&mut self) {
        h5call!(H5Tclose(self.type_id)).expect("Failed to close datatype");
    }
}
