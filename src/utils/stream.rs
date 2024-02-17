use std::{pin::{pin, Pin}, task::{Context, Poll}};

use futures_util::Stream;
use ntex::util::{Bytes, BytesMut};
use tokio::io::AsyncRead;
use pin_project::pin_project;
use tokio_util::io::poll_read_buf;

#[derive(Debug)]
#[pin_project]
pub struct ReaderChunkedStream<R> {
        #[pin]
        reader: Option<R>,
        buf: BytesMut,
        chunk_size: usize
}
pub const DEFAULT_CHUNK_SIZE: usize = 65535 - 40;
impl<R: AsyncRead + Unpin> ReaderChunkedStream<R> {
    pub fn new(reader: R) -> Self {
        Self::with_chunk_size(reader, DEFAULT_CHUNK_SIZE)
    }
    pub fn with_chunk_size(reader: R, chunk_size: usize) -> Self {
        let mut buf = BytesMut::with_capacity(chunk_size);
        buf.reserve(chunk_size);
        ReaderChunkedStream {
            reader: Some(reader),
            buf,
            chunk_size
        }
    }
}
impl<R: AsyncRead + Unpin> Stream for ReaderChunkedStream<R> {
    type Item = std::io::Result<Bytes>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let r = match this.reader.as_mut().as_pin_mut(){
            None => {
                return Poll::Ready(None)
            },
            Some(reader) => reader
        };
        
        match poll_read_buf(r, cx, this.buf){
            Poll::Ready(Ok(_)) => {},
            Poll::Ready(Err(e)) =>{ 
                return Poll::Ready(Some(Err(e)))
            },
            Poll::Pending => {
                return Poll::Pending
            },
        };
        
        if this.buf.len() == 0 {
            this.reader.set(None);
            return Poll::Ready(Some(Ok(this.buf.clone().freeze())));
        }

        let mut old_buf = BytesMut::with_capacity(*this.chunk_size);
        old_buf.reserve(*this.chunk_size);
        std::mem::swap(this.buf, &mut old_buf);
        return Poll::Ready(Some(Ok(old_buf.freeze())));
    }
}
#[pin_project]
pub struct AsyncReadMerger<R, E>{
    #[pin]
    r1: R,
    #[pin]
    r2: E,
    switched: bool
}

impl<R: AsyncRead + Unpin, E: AsyncRead + Unpin> AsyncReadMerger<R, E>{
    pub fn new(r1: R, r2: E) -> Self{
        Self{
            r1,
            r2,
            switched: false
        }
    }
}

impl<R: AsyncRead + Unpin, E: AsyncRead + Unpin> AsyncRead for AsyncReadMerger<R, E>{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let mut this = self.project();
        let old_filled_len = buf.filled().len();

        let cur: Pin<&mut (dyn AsyncRead + Unpin)> = if *this.switched{
            this.r2.as_mut()
        } else {
            this.r1.as_mut()
        };
        match cur.poll_read(cx, buf){
            Poll::Ready(Ok(_)) => {},
            Poll::Ready(Err(e)) =>{ 
                return Poll::Ready(Err(e))
            },
            Poll::Pending => {
                return Poll::Pending
            },
        };
        if old_filled_len == buf.filled().len() && !*this.switched{
            *this.switched = true;
            match this.r2.poll_read(cx, buf){
                Poll::Ready(Ok(_)) => return Poll::Ready(Ok(())),
                Poll::Ready(Err(e)) =>{ 
                    return Poll::Ready(Err(e))
                },
                Poll::Pending => {
                    return Poll::Pending
                },
            };
        } else {
            Poll::Ready(Ok(()))
        }
    }
}