use std::{io::{IoSlice, IoSliceMut}, mem::MaybeUninit, os::{fd::AsFd, unix::net::UnixStream as StdUnixStream}, time::Duration};

use chrono::Local;
use libc::fork;
use rustix::net::{recvmsg, sendmsg, RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags, SendAncillaryBuffer, SendAncillaryMessage, SendFlags};
use tokio::{io::{AsyncReadExt as _, AsyncWriteExt, Interest}, net::UnixStream, time::sleep};

async fn parent_main(rights: UnixStream) -> anyhow::Result<()> {
    let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
    let mut cmsg_buffer = RecvAncillaryBuffer::new(&mut space);
    let mut msg = String::new();
    let mut dummy = vec![0; 1024];
    loop {
        rights.async_io(Interest::READABLE, || Ok(recvmsg(&rights, &mut [IoSliceMut::new(&mut dummy)], &mut cmsg_buffer, RecvFlags::empty())?)).await?;
        for cmsg in cmsg_buffer.drain() {
            match cmsg {
                RecvAncillaryMessage::ScmRights(fds) => {
                    for fd in fds {
                        let mut con = UnixStream::from_std(StdUnixStream::from(fd))?;
                        msg.clear();
                        con.read_to_string(&mut msg).await?;
                        println!("[{:?}] message: {msg}", Local::now().time());
                    }
                },
                _ => ()
            }
        }
    }
}

async fn child_main(rights: UnixStream) -> anyhow::Result<()> {
    let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
    loop {
        let (mut here, there) = UnixStream::pair()?;
        let fds = [there.as_fd()];
        let mut cmsg_buffer = SendAncillaryBuffer::new(&mut space);
        cmsg_buffer.clear();
        cmsg_buffer.push(SendAncillaryMessage::ScmRights(&fds));
        rights.async_io(Interest::WRITABLE, || Ok(sendmsg(&rights, &[IoSlice::new(&[])], &mut cmsg_buffer, SendFlags::empty())?)).await?;
        drop(there);
        here.write_all(b"patrick star").await?;
        here.flush().await?;
        drop(here);

        sleep(Duration::from_secs(1)).await;
    }
}

fn main() -> anyhow::Result<()> {
    let (parent, child) = StdUnixStream::pair()?;
    match unsafe { fork() } {
        -1 => Err(anyhow::Error::msg("fork error")),
        0 => {
            drop(parent);
            child.set_nonblocking(true)?;
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(async {
                    let res = child_main(UnixStream::from_std(child)?).await;
                    println!("child died");
                    res
                })
        },
        _ => {
            drop(child);
            parent.set_nonblocking(true)?;
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(async { parent_main(UnixStream::from_std(parent)?).await })
        },
    }
}
