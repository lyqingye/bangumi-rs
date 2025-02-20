use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;

use super::client::Client;
use super::errors::Pan115Error;
use super::model::FileInfo;

type PageFuture<'a> = Pin<Box<dyn Future<Output = Result<Vec<FileInfo>, Pan115Error>> + 'a>>;

pub struct FileStream<'a> {
    client: &'a Client,
    cid: &'a str,
    offset: i32,
    page_size: i32,
    current_page: Vec<FileInfo>,
    has_more: bool,
    current_future: Option<PageFuture<'a>>,
}

impl<'a> FileStream<'a> {
    pub fn new(client: &'a Client, cid: &'a str, page_size: i32) -> Self {
        Self {
            client,
            cid,
            offset: 0,
            page_size,
            current_page: Vec::new(),
            has_more: true,
            current_future: None,
        }
    }
}

impl Stream for FileStream<'_> {
    type Item = Result<FileInfo, Pan115Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // 如果当前页为空且还有更多数据，则加载下一页
        if self.current_page.is_empty() && self.has_more {
            if self.current_future.is_none() {
                // 创建一个新的 Future 来加载下一页数据
                let cid = self.cid;
                let offset = self.offset;
                let page_size = self.page_size;
                let fut = self.client.list_files(cid, Some(offset), Some(page_size));
                self.current_future = Some(Box::pin(fut));
            }

            // 检查当前的 Future 是否完成
            if let Some(fut) = &mut self.current_future {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok(files)) => {
                        if files.is_empty() || files.len() < self.page_size as usize {
                            self.has_more = false; // 没有更多数据了
                        }
                        self.current_page = files;
                        self.offset += self.page_size; // 更新偏移量
                        self.current_future = None; // 清除当前的 Future
                    }
                    Poll::Ready(Err(e)) => {
                        self.current_future = None; // 清除当前的 Future
                        return Poll::Ready(Some(Err(e))); // 返回错误
                    }
                    Poll::Pending => return Poll::Pending, // 等待异步操作完成
                }
            }
        }

        // 从当前页中取出一个文件
        if !self.current_page.is_empty() {
            let file = self.current_page.remove(0);
            Poll::Ready(Some(Ok(file)))
        } else {
            Poll::Ready(None) // 没有更多数据了
        }
    }
}
