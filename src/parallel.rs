use core::slice;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::marker;
use std::sync::atomic::{AtomicU32, Ordering};

use heed::types::Bytes;
use heed::{BytesDecode, BytesEncode, RoTxn};
use memmap2::Mmap;
use rand::seq::index;
use rand::Rng;
use roaring::RoaringBitmap;

use crate::internals::{KeyCodec, Leaf, NodeCodec};
use crate::key::{Prefix, PrefixCodec};
use crate::{Database, Distance, ItemId, Result};

/// A structure to store the tree nodes out of the heed database.
pub struct TmpNodes {
    file: BufWriter<File>,
    ids: Vec<u32>,
    bounds: Vec<usize>,
}

impl TmpNodes {
    /// Creates an empty `TmpNodes`.
    pub fn new() -> heed::Result<TmpNodes> {
        let file = tempfile::tempfile().map(BufWriter::new)?;
        Ok(TmpNodes { file, ids: Vec::new(), bounds: vec![0] })
    }

    /// Append a new node in the file.
    pub fn put<'a, DE: BytesEncode<'a>>(
        &mut self,
        item: u32,
        data: &'a DE::EItem,
    ) -> heed::Result<()> {
        let bytes = DE::bytes_encode(data).map_err(heed::Error::Encoding)?;
        self.file.write_all(&bytes)?;
        let last_bound = self.bounds.last().unwrap();
        self.bounds.push(last_bound + bytes.len());
        self.ids.push(item);
        Ok(())
    }

    /// Converts it into a readers to be able to read the nodes.
    pub fn into_reader(self) -> Result<TmpNodesReader> {
        let file = self.file.into_inner().map_err(|iie| iie.into_error())?;
        Ok(TmpNodesReader {
            mmap: unsafe { Mmap::map(&file)? },
            ids: self.ids,
            bounds: self.bounds,
        })
    }
}

/// A reader of nodes stored in a file.
pub struct TmpNodesReader {
    mmap: Mmap,
    ids: Vec<u32>,
    bounds: Vec<usize>,
}

impl TmpNodesReader {
    /// The number of nodes stored in this file.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Returns an forward iterator over the nodes.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &[u8])> {
        self.ids.iter().zip(self.bounds.windows(2)).map(|(&id, bounds)| {
            let [start, end] = [bounds[0], bounds[1]];
            (id, &self.mmap[start..end])
        })
    }
}

/// A concurrent ID generate that will never return the same ID twice.
#[derive(Debug)]
#[repr(transparent)]
pub struct ConcurrentNodeIds(AtomicU32);

impl ConcurrentNodeIds {
    /// Creates the ID generator starting at the given number.
    pub fn new(v: u32) -> ConcurrentNodeIds {
        ConcurrentNodeIds(AtomicU32::new(v))
    }

    /// Returns and increment the ID you can use as a NodeId.
    pub fn next(&self) -> u32 {
        self.0.fetch_add(1, Ordering::SeqCst)
    }

    /// Returns the current id.
    pub fn current(&self) -> u32 {
        self.0.load(Ordering::SeqCst)
    }
}

/// A struture used to keep a list of the leaf nodes in the tree.
///
/// It is safe to share between threads as the pointer are pointing
/// in the mmapped file and the transaction is kept here and therefore
/// no longer touches the database.
pub struct ImmutableLeafs<'t, D> {
    leaf_ids: RoaringBitmap,
    constant_length: Option<usize>,
    offsets: Vec<*const u8>,
    _marker: marker::PhantomData<(&'t (), D)>,
}

impl<'t, D: Distance> ImmutableLeafs<'t, D> {
    /// Creates the structure by fetching all the leaf pointers
    /// and keeping the transaction making the pointers valid.
    pub fn new(rtxn: &'t RoTxn, database: Database<D>, index: u16) -> heed::Result<Self> {
        let mut leaf_ids = RoaringBitmap::new();
        let mut constant_length = None;
        let mut offsets = Vec::new();

        let iter = database
            .remap_types::<PrefixCodec, Bytes>()
            .prefix_iter(rtxn, &Prefix::item(index))?
            .remap_key_type::<KeyCodec>();

        for result in iter {
            let (key, bytes) = result?;
            let item_id = key.node.unwrap_item();
            assert_eq!(*constant_length.get_or_insert(bytes.len()), bytes.len());
            assert!(leaf_ids.push(item_id));
            offsets.push(bytes.as_ptr());
        }

        Ok(ImmutableLeafs { leaf_ids, constant_length, offsets, _marker: marker::PhantomData })
    }

    /// Returns the leafs identified by the given ID.
    pub fn get(&self, item_id: ItemId) -> heed::Result<Option<Leaf<'t, D>>> {
        let len = match self.constant_length {
            Some(len) => len,
            None => return Ok(None),
        };
        let ptr = match self
            .leaf_ids
            .rank(item_id)
            .checked_sub(1)
            .and_then(|offset| self.offsets.get(offset as usize))
        {
            Some(ptr) => *ptr,
            None => return Ok(None),
        };
        let bytes = unsafe { slice::from_raw_parts(ptr, len) };
        NodeCodec::bytes_decode(bytes).map_err(heed::Error::Decoding).map(|node| node.leaf())
    }
}

unsafe impl<D> Sync for ImmutableLeafs<'_, D> {}

pub struct ImmutableSubsetLeafs<'t, D> {
    subset: RoaringBitmap,
    leafs: &'t ImmutableLeafs<'t, D>,
}

impl<'t, D: Distance> ImmutableSubsetLeafs<'t, D> {
    pub fn from_item_ids(
        leafs: &'t ImmutableLeafs<'t, D>,
        iterator: impl IntoIterator<Item = ItemId>,
    ) -> Self {
        ImmutableSubsetLeafs { subset: RoaringBitmap::from_iter(iterator), leafs }
    }

    pub fn get(&self, item_id: ItemId) -> heed::Result<Option<Leaf<'t, D>>> {
        if self.subset.contains(item_id) {
            self.leafs.get(item_id)
        } else {
            Ok(None)
        }
    }

    pub fn choose_two<R: Rng>(&self, rng: &mut R) -> heed::Result<Option<[Leaf<'t, D>; 2]>> {
        let indexes = index::sample(rng, self.subset.len() as usize, 2);
        let first = match self.subset.select(indexes.index(0) as u32) {
            Some(item_id) => self.leafs.get(item_id)?,
            None => None,
        };
        let second = match self.subset.select(indexes.index(1) as u32) {
            Some(item_id) => self.leafs.get(item_id)?,
            None => None,
        };
        Ok(first.zip(second).map(|(a, b)| [a, b]))
    }

    pub fn choose<R: Rng>(&self, rng: &mut R) -> heed::Result<Option<Leaf<'t, D>>> {
        if self.subset.is_empty() {
            Ok(None)
        } else {
            let index = rng.gen_range(0..self.subset.len() as u32);
            match self.subset.select(index) {
                Some(item_id) => self.leafs.get(item_id),
                None => Ok(None),
            }
        }
    }
}
