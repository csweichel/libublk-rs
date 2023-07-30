use libublk::io::{UblkCQE, UblkDev, UblkIO, UblkQueueCtx, UblkTgtImpl};
use libublk::{ctrl::UblkCtrl, UblkError};

pub struct NullTgt {}

// setup null target
impl UblkTgtImpl for NullTgt {
    fn init_tgt(&self, dev: &UblkDev) -> Result<serde_json::Value, UblkError> {
        let dev_size = 250_u64 << 30;
        dev.set_default_params(dev_size);
        Ok(serde_json::json!({}))
    }
    fn tgt_type(&self) -> &'static str {
        "null"
    }
}

fn handle_io(
    _r: &mut io_uring::IoUring<io_uring::squeue::Entry>,
    ctx: &UblkQueueCtx,
    io: &mut UblkIO,
    e: &UblkCQE,
) -> Result<i32, UblkError> {
    let tag = e.get_tag();
    let iod = ctx.get_iod(tag);
    let bytes = unsafe { (*iod).nr_sectors << 9 } as i32;

    io.complete(bytes);
    Ok(0)
}

fn test_add() {
    let s = std::env::args().nth(2).unwrap_or_else(|| "-1".to_string());
    let dev_id = s.parse::<i32>().unwrap();
    let _pid = unsafe { libc::fork() };
    if _pid == 0 {
        libublk::ublk_tgt_worker(
            dev_id,
            2,
            64,
            512_u32 * 1024,
            0,
            true,
            |_| Box::new(NullTgt {}),
            handle_io,
            |dev_id| {
                let mut ctrl = UblkCtrl::new(dev_id, 0, 0, 0, 0, false).unwrap();

                ctrl.dump();
            },
        )
        .unwrap()
        .join()
        .unwrap();
    }
}

fn test_del() {
    let s = std::env::args().nth(2).unwrap_or_else(|| "0".to_string());
    let dev_id = s.parse::<i32>().unwrap();
    let mut ctrl = UblkCtrl::new(dev_id as i32, 0, 0, 0, 0, false).unwrap();

    ctrl.del().unwrap();
}

fn main() {
    if let Some(cmd) = std::env::args().nth(1) {
        match cmd.as_str() {
            "add" => test_add(),
            "del" => test_del(),
            _ => todo!(),
        }
    }
}
