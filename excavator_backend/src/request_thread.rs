use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use std::time::Duration;

pub trait ThreadRequest: Send + 'static {
	type Output: Send;
	fn execute(self) -> Self::Output;
}

pub trait Waker: Clone + Send + 'static {
	fn wake(&self);
}

impl Waker for () {
	fn wake(&self) {}
}

pub struct ThreadRequester<T: ThreadRequest, W: Waker = ()> {
	thread: (Sender<T>, Receiver<T::Output>),
	waker: W,
}

impl<T: ThreadRequest> ThreadRequester<T, ()> {
	pub fn new() -> Self {
		let (sender, _) = channel();
		let (_, receiver) = channel();
		Self { thread: (sender, receiver), waker: () }
	}
	
	pub fn new_with_request(request: T) -> Self {
		Self { thread: Self::spawn_thread(request, ()), waker: () }
	}
}

impl<T: ThreadRequest, W: Waker> ThreadRequester<T, W> {
	pub fn new_with_waker(waker: W) -> Self {
		let (sender, _) = channel();
		let (_, receiver) = channel();
		Self { thread: (sender, receiver), waker }
	}
	
	pub fn new_with_request_and_waker(request: T, waker: W) -> Self {
		Self { thread: Self::spawn_thread(request, waker.clone()), waker }
	}
	
	pub fn replace_waker(&mut self, waker: W) {
		self.waker = waker;
	}
	
	pub fn make_request(&mut self, request: T) {
		if let Err(SendError(request)) = self.thread.0.send(request) {
			self.thread = Self::spawn_thread(request, self.waker.clone());
		}
	}
	
	pub fn take_results(&mut self) -> impl Iterator<Item = T::Output> {
		self.thread.1.try_iter()
	}
	
	fn spawn_thread(request: T, waker: W) -> (Sender<T>, Receiver<T::Output>) {
		let (request_sender, request_receiver) = channel();
		let (result_sender, result_receiver) = channel();
		
		std::thread::spawn(move || {
			let (receiver, sender) = (request_receiver, result_sender);
			let mut next_request = Some(request);
			
			while let Some(request) = next_request.take() {
				let result = request.execute();
				if sender.send(result).is_err() {
					// Stop processing queued requests when the ThreadRequester is dropped
					break;
				}
				
				waker.wake();
				
				let recv = receiver.recv_timeout(Duration::from_secs(1));
				next_request = match recv {
					Ok(request) => Some(request), // continue the thread
					Err(_) => None, // end the thread, because there are no more queued requests
				}
			}
		});
		
		(request_sender, result_receiver)
	}
}
