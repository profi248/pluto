use std::{
    sync::{
        Arc,
        atomic::{ AtomicU8, Ordering },
    },
    collections::{ hash_map::Entry, HashMap },
    task::{ Context, Poll, Waker },
    time::{ Duration, Instant },
    marker::PhantomData,
    cell::UnsafeCell,
    future::Future,
    pin::Pin,
};

use protobuf::{ CodedInputStream, Message as MessageTrait };
use rumqttc::Publish;
use bytes::Bytes;

use parking_lot::Mutex as BlockingMutex;
use tokio::sync::Mutex as AsyncMutex;

use crate::{
    client::Client,
    topics::Topic,
    message::{ MessageVariant, EncryptedMessage },
    protos::shared::EncryptedMessage as EncryptedMessageStruct,
};

/// A static handler object for a given topic.
///
/// [`handle`](#tymethod.handle) will be called when
/// [`IncomingHandler`] receives a message on the topic
/// returned by [`topic`](#tymethod.topic). This method
/// returns `Option<()>` to allow [`?`](std::ops::Try)
/// syntax, but does not actually handle any errros.
///
/// Note that handler objects are expected to be static,
/// so while these functions use `&self`, they should not
/// hold any form of state.
#[async_trait::async_trait]
pub trait Handler: Send + Sync + 'static {
    fn topic(&self) -> Topic;

    async fn handle(&self, message: Message, client: Client) -> Option<()>;
}

/// A wrapper around raw message bytes.
///
/// Contains methods to parse the message into given
/// types.
///
/// This is necessary as [`Handler`]s need to be
/// given messages dynamically.
pub struct Message {
    bytes: Bytes
}

impl Message {
    pub(crate) fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }

    /// Parses this message as unencrypted.
    pub fn unencrypted<M: MessageTrait>(self) -> Result<M, Self> {
        M::parse_from_tokio_bytes(&self.bytes).map_err(|_| self)
    }

    /// Parses this message as encrypted.
    pub fn encrypted<M: MessageTrait>(self) -> Result<EncryptedMessage<M>, Self> {
        EncryptedMessageStruct::parse_from_tokio_bytes(&self.bytes)
            .map(Into::into)
            .map_err(|_| self)
    }
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

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("The topic `{0:?}` is undefined.")]
    InvalidTopic(String),
    #[error("The topic `{0:?}` does not have a handler.")]
    MissingHandler(String),

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

    handlers: HashMap<Topic, Arc<dyn Handler>>,
}

impl IncomingHandler {
    /// Creates a new main handler given a hashmap of
    /// individual message handlers by topic.
    pub fn new(handlers: HashMap<Topic, Arc<dyn Handler>>) -> Self {
        Self {
            topics: AsyncMutex::new(HashMap::new()),
            handlers,
        }
    }

    /// Listens to a given topic, and returns a future for the resposne.
    ///
    /// This method can fail directly if there is already a different context
    /// listening to this topic.
    ///
    /// The future also returns a `Result` for errors with parsing or if
    /// it times out. See [`ResponseError`].
    pub(crate) async fn listen<M: MessageTrait>(&self,
        topic: String,
        expects_encrypted: bool,
        timeout: Duration
    ) -> Result<ResponseFuture<M>, AlreadyListening> {
        let future = ResponseFuture::new(expects_encrypted, timeout);
        let responder = Responder::new(future.inner.clone());

        let clone = responder.clone();
        tokio::spawn(async move {
            tokio::time::sleep(timeout).await;

            clone.wake_timeout();
        });

        let mut topics = self.topics.lock().await;

        match topics.entry(topic.clone()) {
            Entry::Occupied(mut e) => match e.get_mut() {
                Some(r) => {
                    if r.inner.wait_start_time.elapsed() <= r.inner.timeout {
                        return Err(AlreadyListening { topic });
                    }

                    *r = responder;
                },
                e => { *e = Some(responder); }
            },

            Entry::Vacant(e) => { e.insert(Some(responder)); }
        };

        Ok(future)
    }

    /// Handles the incoming message.
    ///
    /// This method forwards incoming messages to handlers or other listening contexts.
    pub async fn handle(&self, message: Publish, client: Client) -> crate::Result<()> {
        let mut topics = self.topics.lock().await;

        let responder = topics.get_mut(&message.topic)
            .map(|r| r.take()).flatten()
            .filter(|r| r.inner.wait_start_time.elapsed() <= r.inner.timeout);

        if let Some(i) = responder {
            i.wake(message.payload);
        }
        else {
            let topic = Topic::from_topic(message.topic.clone())
                .ok_or(HandlerError::InvalidTopic(message.topic.clone()))?;

            let handler = self.handlers.get(&topic)
                .ok_or(HandlerError::MissingHandler(message.topic.clone()))?
                .clone();

            tokio::spawn(async move {
                handler.handle(Message::new(message.payload), client).await;
            });
        }

        Ok(())
    }
}

const RESPONSE_STATE: u8 = 1;
const TIMED_OUT_STATE: u8 = 2;

struct Inner {
    received: AtomicU8,
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
            received: AtomicU8::new(0),
            cell: UnsafeCell::new(None),
            waker: BlockingMutex::new(None),
            wait_start_time: Instant::now(),
            timeout
        }
    }
}

#[derive(Clone)]
struct Responder {
    inner: Arc<Inner>,
}

impl Responder {
    fn new(inner: Arc<Inner>) -> Self {
        Self { inner }
    }

    fn wake(&self, bytes: Bytes) {
        unsafe { *self.inner.cell.get() = Some(bytes); }

        self.inner.received.fetch_or(RESPONSE_STATE, Ordering::Release);

        self.inner.waker.lock()
            .take().map(|w| w.wake());
    }

    fn wake_timeout(&self) {
        self.inner.received.fetch_or(TIMED_OUT_STATE, Ordering::Relaxed);

        self.inner.waker.lock()
            .take().map(|w| w.wake());
    }
}

/// A future waiting for a response message `M`.
pub struct ResponseFuture<M: MessageTrait> {
    inner: Arc<Inner>,
    expects_encrypted: bool,

    _phantom: PhantomData<MessageVariant<M>>
}

impl<M: MessageTrait> ResponseFuture<M> {
    fn new(expects_encrypted: bool, timeout: Duration) -> Self {
        Self {
            inner: Arc::new(Inner::new(timeout)),
            expects_encrypted,

            _phantom: PhantomData
        }
    }
}

impl<M: MessageTrait> Future for ResponseFuture<M> {
    type Output = Result<MessageVariant<M>, ResponseError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.inner.received.load(Ordering::Acquire);

        if state & TIMED_OUT_STATE != 0 {
            return Poll::Ready(Err(ResponseError::TimedOut));
        }

        if state & RESPONSE_STATE != 0 {
            let bytes = unsafe { self.inner.cell.get().as_mut().unwrap().take().unwrap() };

            let mut stream = CodedInputStream::from_tokio_bytes(&bytes);

            let message: Result<MessageVariant<M>, ResponseError> = if self.expects_encrypted {
                EncryptedMessageStruct::parse_from(&mut stream)
                    .map(|m| EncryptedMessage::from(m))
                    .map(Into::into)
                    .map_err(Into::into)
            } else {
                M::parse_from(&mut stream)
                    .map(Into::into)
                    .map_err(Into::into)
            };

            return Poll::Ready(message);
        }

        *self.inner.waker.lock() = Some(cx.waker().clone());

        Poll::Pending
    }
}
