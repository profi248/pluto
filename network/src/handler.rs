use std::{
    sync::{
        Arc,
        atomic::{ AtomicBool, Ordering },
    },
    collections::{ hash_map::Entry, HashMap },
    task::{ Context, Poll, Waker },
    time::{ Duration, Instant },
    any::{ Any, TypeId },
    borrow::Cow,
    marker::PhantomData,
    cell::UnsafeCell,
    future::Future,
    pin::Pin,
};

use protobuf::{ CodedInputStream, Message };
use rumqttc::Publish;
use bytes::Bytes;

use parking_lot::Mutex as BlockingMutex;
use tokio::sync::Mutex as AsyncMutex;

use crate::{ client::Client, topics::Topic };

pub trait Handler: Send + Sync {
    fn topic(&self) -> &Topic;

    fn handle(&self, message: Bytes, client: &Client) -> crate::Result<()>;
}

/// Error returned when there is already a different
/// listener on the requested topic.
#[derive(Debug)]
pub struct AlreadyListening {
    topic: String,
}

impl std::fmt::Display for AlreadyListening {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
           "The requested topic `{}` is already being listened to by another handler.",
           self.topic
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResponseError {
    #[error("Timed out.")]
    TimedOut,
    #[error("{0}")]
    Protobuf(#[from] protobuf::Error)
}

impl From<ResponseError> for crate::Error {
    fn from(e: ResponseError) -> Self {
        match e {
            ResponseError::TimedOut => Self::TimedOut,
            ResponseError::Protobuf(e) => Self::Protobuf(e),
        }
    }
}

#[derive(Debug)]
pub struct MissingHandler {
    topic: Topic,
}

impl std::fmt::Display for MissingHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
           "The topic `{:?}` does not have a handler.",
            self.topic
        )
    }
}

/// Handler for forwarding incoming messages to the correct destination.
///
/// This includes chained requests, which will receive the output directly,
/// as well as calling new handlers.
pub struct IncomingHandler {
    topics: AsyncMutex<HashMap<String, Option<Responder>>>,

    // TODO: implement default handlers.
    handlers: HashMap<Topic, Box<dyn Handler>>,
}

impl IncomingHandler {
    pub fn new(handlers: HashMap<Topic, Box<dyn Handler>>) -> Self {
        Self {
            topics: AsyncMutex::new(HashMap::new()),
            handlers,
        }
    }

    pub async fn listen<M: Message>(&self, topic: String, timeout: Duration) -> Result<ResponseFuture<M>, AlreadyListening> {
        let future = ResponseFuture::new(timeout);
        let responder = Responder::new(future.inner.clone());

        let mut topics = self.topics.lock().await;

        match topics.entry(topic.clone()) {
            Entry::Occupied(mut e) => match e.get_mut() {
                Some(_) => return Err(AlreadyListening { topic }),
                e => { *e = Some(responder); }
            },

            Entry::Vacant(e) => { e.insert(Some(responder)); }
        };

        Ok(future)
    }

    pub async fn handle(&self, message: Publish) {
        let mut topics = self.topics.lock().await;

        let responder = topics.get_mut(&message.topic)
            .map(|r| r.take()).flatten()
            .filter(|r| r.inner.wait_start_time.elapsed() <= r.inner.timeout);

        if let Some(i) = responder {
            i.wake(message.payload);
        }
        else {
            // self.handlers.get()
        }
    }
}

struct Inner {
    received: AtomicBool,
    cell: UnsafeCell<Option<Bytes>>,

    waker: BlockingMutex<Option<Waker>>,

    wait_start_time: Instant,
    timeout: Duration,
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

impl Inner {
    fn new(timeout: Duration) -> Self {
        Self {
            received: AtomicBool::new(false),
            cell: UnsafeCell::new(None),
            waker: BlockingMutex::new(None),
            wait_start_time: Instant::now(),
            timeout
        }
    }
}

struct Responder {
    inner: Arc<Inner>,
}

impl Responder {
    pub fn new(inner: Arc<Inner>) -> Self {
        Self { inner }
    }

    fn wake(self, bytes: Bytes) {
        unsafe { *self.inner.cell.get() = Some(bytes); }

        self.inner.received.store(true, Ordering::Release);

        self.inner.waker.lock()
            .take().map(|w| w.wake());
    }
}

pub struct ResponseFuture<M: Message> {
    inner: Arc<Inner>,

    _phantom: PhantomData<M>
}

impl<M: Message> ResponseFuture<M> {
    fn new(timeout: Duration) -> Self {
        Self {
            inner: Arc::new(Inner::new(timeout)),

            _phantom: PhantomData
        }
    }
}

impl<M: Message> Future for ResponseFuture<M> {
    type Output = Result<M, ResponseError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.inner.received.load(Ordering::Acquire) {
            *self.inner.waker.lock() = Some(cx.waker().clone());

            return Poll::Pending;
        }

        let bytes = unsafe { self.inner.cell.get().as_mut().unwrap().take().unwrap() };

        let mut stream = CodedInputStream::from_tokio_bytes(&bytes);

        Poll::Ready(M::parse_from(&mut stream).map_err(Into::into))
    }
}
