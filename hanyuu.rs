// SPDX-License-Identifier: GPL-2.0
//! Simple Block Device module for rust-linux v6.11
use core::ops::Deref;

use gen_disk::GenDisk;
use kernel::alloc::flags;
use kernel::block::mq::{gen_disk, Operations, Request, TagSet};
use kernel::prelude::*;
use kernel::sync::{new_mutex, Arc, Mutex};
use kernel::types::ARef;

module! {
    type: HanyuuKernelModule,
    name: "hanyuu",
    author: "Hanyuu Furude",
    description: "Rust block Hanyuu",
    license: "GPL v2",
}

struct HanyuuBlkDevice;

#[vtable]
impl Operations for HanyuuBlkDevice {
    fn queue_rq(rq: ARef<Request<Self>>, _is_last: bool) -> Result {
        pr_info!("queue_rq");
        let _ = Request::end_ok(rq);
        Ok(())
    }

    fn commit_rqs() {}
}

#[pin_data]
struct Disk {
    #[pin]
    blk_device: Mutex<GenDisk<HanyuuBlkDevice>>,
}

impl Disk {
    fn new(blk_device: GenDisk<HanyuuBlkDevice>) -> impl PinInit<Self> {
        pin_init!(Self {
            blk_device <- new_mutex!(blk_device),
        })
    }
}

struct HanyuuKernelModule {
    #[allow(dead_code)]
    disk: Arc<Disk>,
}

impl kernel::Module for HanyuuKernelModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hanyuu loaded\n");
        let tagset: Arc<TagSet<HanyuuBlkDevice>> =
            Arc::pin_init(TagSet::new(1, 256, 1), flags::GFP_KERNEL)?;

        let mut gen_disk_builder = gen_disk::GenDiskBuilder::new();

        gen_disk_builder = gen_disk_builder.capacity_sectors(512);
        gen_disk_builder = gen_disk_builder.logical_block_size(512)?;
        gen_disk_builder = gen_disk_builder.physical_block_size(512)?;

        let gen_disk = gen_disk_builder.build(format_args!("hau"), tagset)?;

        Ok(HanyuuKernelModule {
            disk: Arc::pin_init(Disk::new(gen_disk), GFP_KERNEL)?,
        })
    }
}
