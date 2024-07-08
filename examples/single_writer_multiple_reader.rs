use std::ffi::CString;
use std::time::Duration;
use std::{ptr, thread};

use hdf5::h5call;
use hdf5_sys::h5::hsize_t;
use hdf5_sys::h5d::{H5Dclose, H5Dflush, H5Dset_extent};
use hdf5_sys::h5f::H5F_scope_t::H5F_SCOPE_GLOBAL;
use hdf5_sys::h5f::{H5Fclose, H5Fflush};
use hdf5_sys::h5p::{H5Pclose, H5Pcreate, H5Pset_chunk};
use hdf5_sys::h5s::H5S_seloper_t::H5S_SELECT_SET;
use hdf5_sys::h5s::{H5Sclose, H5Sselect_hyperslab, H5S_UNLIMITED};
use hdf5_sys::h5t::H5T_NATIVE_LLONG;
use hdf5_sys::{
    h5d::{H5D_create, H5Dwrite},
    h5f::{H5Fcreate, H5F_ACC_SWMR_WRITE, H5F_ACC_TRUNC},
    h5p::H5P_DEFAULT,
    h5s::H5Screate_simple,
};

unsafe fn writer_thread(file_name: CString, dataset_name: CString) {
    // Create file with SWMR write access
    let file_id = h5call!(H5Fcreate(
        file_name.as_ptr(),
        H5F_ACC_TRUNC | H5F_ACC_SWMR_WRITE,
        H5P_DEFAULT,
        H5P_DEFAULT,
    ))
    .unwrap();

    // Create dataspace with unlimited size
    let max_dims: [hsize_t; 1] = [H5S_UNLIMITED];
    let initial_dims: [hsize_t; 1] = [0];
    let dataspace_id = h5call!(H5Screate_simple(
        1,
        initial_dims.as_ptr(),
        max_dims.as_ptr()
    ))
    .unwrap();

    // Create dataset with chunking
    let chunk_dims: [hsize_t; 1] = [100];
    use hdf5::globals::H5P_DATASET_CREATE;
    let plist_id = h5call!(H5Pcreate(*H5P_DATASET_CREATE)).unwrap();
    h5call!(H5Pset_chunk(plist_id, 1, chunk_dims.as_ptr())).unwrap();

    let dataset_id = h5call!(H5D_create(
        file_id,
        dataset_name.as_ptr(),
        *H5T_NATIVE_LLONG,
        dataspace_id,
        H5P_DEFAULT,
        plist_id,
        H5P_DEFAULT,
    ))
    .unwrap();

    h5call!(H5Pclose(plist_id)).unwrap();
    // Close the initial dataspace as it's no longer needed
    h5call!(H5Sclose(dataspace_id)).unwrap();

    // Write data
    for i in 0..40 {
        thread::sleep(Duration::from_millis(100));
        let value = i as i64;
        let start = i as hsize_t;
        let count = 1 as hsize_t;

        // Extend the dataset
        let new_size: [hsize_t; 1] = [i + 1];
        h5call!(H5Dset_extent(dataset_id, new_size.as_ptr())).unwrap();

        // Re-create the file dataspace after extending
        let file_dataspace_id =
            h5call!(H5Screate_simple(1, new_size.as_ptr(), max_dims.as_ptr())).unwrap();

        let mem_space = h5call!(H5Screate_simple(1, &count, ptr::null())).unwrap();
        h5call!(H5Sselect_hyperslab(
            file_dataspace_id,
            H5S_SELECT_SET,
            &start,
            ptr::null(),
            &count,
            ptr::null()
        ))
        .unwrap();
        h5call!(H5Dwrite(
            dataset_id,
            *H5T_NATIVE_LLONG,
            mem_space,
            file_dataspace_id,
            H5P_DEFAULT,
            &value as *const i64 as *const _,
        ))
        .unwrap();
        h5call!(H5Sclose(file_dataspace_id)).unwrap();
        h5call!(H5Sclose(mem_space)).unwrap();
        h5call!(H5Dflush(dataset_id)).unwrap();
        h5call!(H5Fflush(file_id, H5F_SCOPE_GLOBAL)).unwrap();
    }

    // Close everything
    h5call!(H5Dclose(dataset_id)).unwrap();
    h5call!(H5Fclose(file_id)).unwrap();
}

unsafe fn reader_thread(file_name: CString, dataset_name: CString) {
    use hdf5_test::my_hdf5::dataset::H5Dataset;
    use hdf5_test::swmr::{OpenFileMrRead, SwmrReadFapl};
    thread::sleep(Duration::from_secs(1));
    let f = SwmrReadFapl::new().unwrap();
    let reader = OpenFileMrRead::new_swmr_reader(file_name, f.id()).unwrap();

    let dataset = H5Dataset::new(dataset_name, reader.id()).unwrap();

    for _ in 0..5 {
        let data = dataset.get_newest_n(20).unwrap();

        println!("Read: {data:?}");
        thread::sleep(Duration::from_secs(1));
    }
    let mut buf = Vec::new();
    dataset.read_all(&mut buf).unwrap();
    println!("All data: {buf:?}");
}

fn main() {
    let file_name = CString::new("test.h5").unwrap();
    let dataset_name = CString::new("data").unwrap();

    let writer = thread::spawn({
        let fname = file_name.clone();
        let dataset = dataset_name.clone();
        move || unsafe { writer_thread(fname, dataset) }
    });
    let reader1 = thread::spawn({
        let fname = file_name.clone();
        let dataset = dataset_name.clone();
        move || unsafe { reader_thread(fname, dataset) }
    });
    let reader2 = thread::spawn(move || unsafe { reader_thread(file_name, dataset_name) });

    writer.join().expect("Writer thread panicked");
    reader1.join().expect("Reader thread panicked");
    reader2.join().expect("Reader thread panicked");
}
